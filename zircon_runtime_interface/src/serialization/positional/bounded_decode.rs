//! Apply each format's runtime byte limit before the codec allocates a
//! length-prefixed container. A bounded Read alone checks too late.
//!
//! 这里把实际 wire 消耗与 decoder 的字节申领分别计数：变长整数可能以很短的编码占用
//! 输入，却按原生整数宽度申领；长度前缀容器则可能在读取内容前先申请存储。

use std::io::Read;

use bincode_next::config::{Config, IntEncoding};
use bincode_next::de::read::{BorrowReader, Reader, SliceReader};
use bincode_next::de::{BorrowDecoder, Decoder, DecoderImpl};
use bincode_next::error::DecodeError;
use bincode_next::serde::{BorrowCompat, Compat};
use bincode_next::{BorrowDecode, Decode, IoReader};
use serde::de::DeserializeOwned;

use super::MAX_DECODE_ALLOCATION_BYTES;

/// 从切片前缀解出一个拥有型 serde 值，并返回 decoder 实际消费的位置。
///
/// 上层先限制完整切片长度；此处额外限制 bincode 对标量和容器作出的累计字节申领。
/// 前缀后的字节会留给上层解析，不在此函数内视为尾随错误。
pub(super) fn slice<T: DeserializeOwned, C: Config>(
    bytes: &[u8],
    max: u64,
    config: C,
) -> Result<(T, usize), DecodeError> {
    let reader = CountedSlice {
        inner: SliceReader::new(bytes),
        consumed: 0,
    };
    let mut decoder = BoundedDecoder::new(reader, config, max);
    let value = BorrowCompat::<T>::borrow_decode(&mut decoder)?.0;
    Ok((value, decoder.reader().consumed))
}

/// 从受限 reader 中解出一个拥有型 serde 值。
///
/// 只消费一个值所需的输入，不负责确认流已经到达 EOF；完整性要求由调用方决定。
pub(super) fn stream<T: DeserializeOwned, C: Config>(
    reader: impl Read,
    max: u64,
    config: C,
) -> Result<T, DecodeError> {
    let mut decoder = BoundedDecoder::new(
        CountedStream {
            inner: IoReader::new(reader),
            consumed: 0,
        },
        config,
        max,
    );
    Compat::<T>::decode(&mut decoder).map(|value| value.0)
}

/// 在 bincode decoder 外分别约束累计 decoder claim 与可用于字节容器的 wire 预算。
struct BoundedDecoder<D> {
    // 保留 bincode 配置、上下文和其自身的硬限额。
    inner: D,
    // 已获准的累计 claim；递归容器申领和标量读取共用此计数。
    claimed: usize,
    // 该编码模式下允许的累计 claim 上限；变长整数会按配置放宽，仍受全局硬限额约束。
    max: usize,
    // 与 claim 分开的 wire 上限，用于在读取容器内容前拦截字符串和字节缓冲区。
    wire_max: usize,
}

impl<R: ReadProgress, C: Config> BoundedDecoder<DecoderImpl<R, C, ()>> {
    /// 创建两种预算计数器；变长整数的 claim 计数按最多 16 个原生字节放宽，wire 上限不变。
    fn new(reader: R, config: C, max: u64) -> Self {
        // Variable integers can occupy one wire byte while the decoder claims
        // their full native width (up to 16 bytes). Track wire reads separately.
        let factor = if config.int_encoding() == IntEncoding::Variable {
            16
        } else {
            1
        };
        Self {
            inner: DecoderImpl::new(reader, config, ()),
            claimed: 0,
            max: max
                .saturating_mul(factor)
                .min(MAX_DECODE_ALLOCATION_BYTES as u64) as usize,
            wire_max: max.min(MAX_DECODE_ALLOCATION_BYTES as u64) as usize,
        }
    }
}

// Forward the public codec extension markers and coverage assertion along
// with Decoder. The underlying decoder retains its configured hard ceiling.
// 该包装器仍须满足 bincode 的 sealed/error-path 约束；静态覆盖断言必须传递到底层 decoder。
impl<D: Decoder> bincode_next::utils::Sealed for BoundedDecoder<D> {}
impl<D: Decoder> bincode_next::error_path::BincodeErrorPathCovered<0> for BoundedDecoder<D> {
    const ASSERT: () = <D as bincode_next::error_path::BincodeErrorPathCovered<0>>::ASSERT;
}

/// 拦截 claim 操作，同时把读取器、配置及上下文交给原始 bincode decoder。
impl<D: Decoder> Decoder for BoundedDecoder<D>
where
    D::R: ReadProgress,
{
    type C = D::C;
    type R = D::R;
    type Context = D::Context;

    fn config(&self) -> &Self::C {
        self.inner.config()
    }
    fn reader(&mut self) -> &mut Self::R {
        self.inner.reader()
    }
    fn context(&mut self) -> &mut Self::Context {
        self.inner.context()
    }

    fn claim_bytes_read(&mut self, n: usize) -> Result<(), DecodeError> {
        // 先检查包装层预算，再向内层登记；只有两层均接受后才提交本地累计值。
        let claimed = self
            .claimed
            .checked_add(n)
            .filter(|claimed| *claimed <= self.max)
            .ok_or(DecodeError::LimitExceeded)?;
        self.inner.claim_bytes_read(n)?;
        self.claimed = claimed;
        Ok(())
    }

    fn unclaim_bytes_read(&mut self, n: usize) {
        // 只回退本层并向内层转发字节预算记账，不代表容器存储已释放。
        self.claimed -= n;
        self.inner.unclaim_bytes_read(n);
    }

    fn claim_container_read<T>(&mut self, len: usize) -> Result<(), DecodeError> {
        // 长度前缀容器可能在其 payload 尚未读取时分配存储。字节容器须先满足剩余 wire
        // 预算；其他元素类型则由累计 decoder claim 与内层硬上限约束。
        let bytes = len
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(DecodeError::LimitExceeded)?;
        // Legacy strings and byte buffers allocate before reading. Their byte
        // count must fit the caller's remaining wire budget in either encoding.
        if std::mem::size_of::<T>() == 1
            && bytes > self.wire_max.saturating_sub(self.inner.reader().consumed())
        {
            return Err(DecodeError::LimitExceeded);
        }
        self.claim_bytes_read(bytes)
    }
}

/// 将借用读取器访问转发到底层 decoder；上层类型仍要求 DeserializeOwned。
impl<'de, D: BorrowDecoder<'de>> BorrowDecoder<'de> for BoundedDecoder<D>
where
    D::R: ReadProgress,
{
    type BR = D::BR;
    fn borrow_reader(&mut self) -> &mut Self::BR {
        self.inner.borrow_reader()
    }
}

/// 为切片 reader 增加消费计数，供前缀解码返回准确偏移。
struct CountedSlice<'de> {
    // 保持原切片的有界读取与借用访问语义。
    inner: SliceReader<'de>,
    // Reader 消费、显式 consume 和借用 take_bytes 的累计值；peek 不在此计入。
    consumed: usize,
}

/// 提供已消费 wire 字节数的读取器能力，用来计算字节容器尚可使用的输入预算。
trait ReadProgress: Reader {
    /// 返回已提交消费的 wire 字节；窥看但未提交的字节不计入。
    fn consumed(&self) -> usize;
}

impl ReadProgress for CountedSlice<'_> {
    fn consumed(&self) -> usize {
        self.consumed
    }
}

// 只在读取成功或消费提交后累计；peek_read 本身不改变偏移。
impl Reader for CountedSlice<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        // SliceReader 成功填满请求后才提交计数；失败不会虚增消费偏移。
        self.inner.read(bytes)?;
        self.consumed += bytes.len();
        Ok(())
    }

    fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
        self.inner.peek_read(n)
    }

    fn consume(&mut self, n: usize) {
        // peek_read 仅借看；只有 decoder 调用 consume 时才将字节计入偏移。
        self.inner.consume(n);
        self.consumed += n;
    }
}

impl<'de> BorrowReader<'de> for CountedSlice<'de> {
    fn take_bytes(&mut self, n: usize) -> Result<&'de [u8], DecodeError> {
        // 借用式读取同样推进 wire 偏移，否则前缀返回值和剩余字节预算都会偏小。
        let bytes = self.inner.take_bytes(n)?;
        self.consumed += n;
        Ok(bytes)
    }
}

/// 为流式 reader 计数成功读取的字节；调用方另行决定是否检查流尾。
struct CountedStream<R> {
    // 统一 bincode 的流式 Read 实现。
    inner: IoReader<R>,
    // 成功读取的实际 wire 字节数，用于长度前缀容器的剩余预算。
    consumed: usize,
}

// IoReader::read 完整满足请求后才累计，错误路径不会虚增已消费字节数。
impl<R: Read> Reader for CountedStream<R> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.inner.read(bytes)?;
        self.consumed += bytes.len();
        Ok(())
    }
}

impl<R: Read> ReadProgress for CountedStream<R> {
    fn consumed(&self) -> usize {
        self.consumed
    }
}
