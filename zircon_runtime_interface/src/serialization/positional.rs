//! Explicit positional formats used by existing asset files, wire envelopes,
//! and cache keys. Changing the codec must not change their persisted bytes.
//!
//! 此模块按调用方选定的布局编解码单个值，不读取统一的 schema、版本或格式前缀。
//! 文件封套、身份校验、格式版本和尾随数据策略由各调用方负责；带 schema 的通用
//! 文档入口见 [`VersionedSchema`](super::VersionedSchema) 与 [`PayloadHeader`](super::PayloadHeader)。
//!
//! # 约束
//! 两种编码均固定既有 bincode 配置。持久化调用方不能仅为本地实现便利而切换变体，
//! 否则已有资产、缓存键或二进制封套的字节布局会改变。

use std::io::{self, Read, Write};

use serde::{de::DeserializeOwned, Serialize};

#[path = "positional/bounded_decode.rs"]
mod bounded_decode;

// Match the largest admitted cooked-artifact payload. Individual formats keep
// their smaller byte caps; a shared codec must not impose a new 256 MiB cutoff.
// 各格式仍须传入自己的 wire 字节预算；此常量是共享 decoder 的历史硬上限，
// 不是每次调用实际可用的分配额度。变长整数格式还会单独放宽 decoder claim 预算。
const MAX_DECODE_ALLOCATION_BYTES: usize = 2 * 1024 * 1024 * 1024;

/// 选择持久化位置格式所用的 bincode 整数布局。
///
/// 该选择不携带 schema 或版本信息；封套、schema 或版本的校验责任与时机均由具体格式决定。使用
/// 同一持久格式的读写双方应保持同一变体，以免历史字节失去兼容性。
///
/// # 参考
/// schema 版本化载荷由 [`VersionedSchema`](super::VersionedSchema) 与
/// [`PayloadHeader`](super::PayloadHeader) 描述；本枚举只负责值的 positional 编码。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionalEncoding {
    /// 小端序固定宽度整数；用于已经以该布局持久化的值。
    FixedLittleEndian,
    /// 小端序变长整数；其整数宽度策略也是持久化字节契约的一部分。
    VariableLittleEndian,
}

/// 将值写为 positional 格式时可能遇到的错误。
///
/// 大小上限针对编码后的 wire 字节。流式写入失败不会撤销底层 writer 已接受的前缀。
#[derive(Debug, thiserror::Error)]
pub enum PositionalEncodeError {
    #[error("positional payload exceeds {max} bytes")]
    /// 编码结果将超过调用方提供的字节上限。
    /// 携带本次编码允许的最大 wire 字节数 `max`。
    SizeLimit { max: u64 },
    #[error("encode positional payload: {0}")]
    /// bincode 编码器或底层 writer 返回错误。
    /// 携带原始编码或 I/O 错误，供错误链追溯。
    Codec(#[source] bincode_next::error::EncodeError),
}

/// 从 positional 字节恢复值时可能遇到的错误。
///
/// 切片完整解码会报告尾随字节；前缀和流式入口则把剩余输入的处理留给调用方。
#[derive(Debug, thiserror::Error)]
pub enum PositionalDecodeError {
    #[error("positional payload exceeds {max} bytes")]
    /// 切片长度超过入口的 wire 字节上限。
    /// 携带调用方提供的最大 wire 字节数 `max`。
    SizeLimit { max: u64 },
    #[error("positional payload has {remaining} trailing bytes")]
    /// 完整切片解码一个值后仍有未消费字节。
    /// `remaining` 是输入切片中未被该值消费的字节数。
    TrailingBytes { remaining: usize },
    #[error("decode positional payload: {0}")]
    /// bincode 解码失败，包括输入不完整或 decoder claim 超限。
    /// 携带原始解码错误，供错误链追溯。
    Codec(#[source] bincode_next::error::DecodeError),
}

impl PositionalEncoding {
    /// 将值编码为完整的 positional 字节向量，不附加调用方字节上限。
    ///
    /// # 参数
    /// - `value`：由 serde 序列化的值。
    ///
    /// # 返回值
    /// 编码后的完整字节向量。
    ///
    /// # 错误
    /// 自定义序列化逻辑失败时返回 [`PositionalEncodeError::Codec`]。
    pub fn encode<T: Serialize + ?Sized>(
        self,
        value: &T,
    ) -> Result<Vec<u8>, PositionalEncodeError> {
        self.encode_bounded(value, u64::MAX)
    }

    /// 将值编码为完整字节向量，并在输出超过 `max` 字节时失败。
    ///
    /// # 参数
    /// - `value`：由 serde 序列化的值。
    /// - `max`：允许的编码后 wire 字节数。
    ///
    /// # 返回值
    /// 不超过上限的完整编码。
    ///
    /// # 错误
    /// 超限时返回 [`PositionalEncodeError::SizeLimit`]；序列化失败时返回
    /// [`PositionalEncodeError::Codec`]。
    pub fn encode_bounded<T: Serialize + ?Sized>(
        self,
        value: &T,
        max: u64,
    ) -> Result<Vec<u8>, PositionalEncodeError> {
        let mut bytes = Vec::new();
        self.encode_into(&mut bytes, value, max)?;
        Ok(bytes)
    }

    /// Streams exactly once and rejects a write before it crosses the byte cap.
    ///
    /// 此入口不先构造完整的临时向量。若编码或限额检查失败，writer 中已接受的前缀不会
    /// 回滚；需要原子替换时由调用方提供事务性 writer。
    ///
    /// # 参数
    /// - `writer`：接收 positional 字节的目标。
    /// - `value`：由 serde 序列化的值。
    /// - `max`：整个输出可写入的 wire 字节上限。
    ///
    /// # 返回值
    /// 成功写出的字节数。
    ///
    /// # 错误
    /// 超限返回 [`PositionalEncodeError::SizeLimit`]；serde 或 writer 失败返回
    /// [`PositionalEncodeError::Codec`]。
    pub fn encode_into<T: Serialize + ?Sized>(
        self,
        writer: &mut impl Write,
        value: &T,
        max: u64,
    ) -> Result<usize, PositionalEncodeError> {
        let mut writer = LimitedWriter {
            writer,
            remaining: max,
            exceeded: false,
        };
        let result = match self {
            Self::FixedLittleEndian => bincode_next::serde::encode_into_std_write(
                value,
                &mut writer,
                bincode_next::config::legacy(),
            ),
            Self::VariableLittleEndian => bincode_next::serde::encode_into_std_write(
                value,
                &mut writer,
                bincode_next::config::legacy().with_variable_int_encoding(),
            ),
        };
        // 保留独立标记，使 cap 拒绝映射为稳定的 SizeLimit，而非包装成任意 I/O 错误。
        if writer.exceeded {
            return Err(PositionalEncodeError::SizeLimit { max });
        }
        result.map_err(PositionalEncodeError::Codec)
    }

    /// 从完整切片解码一个值，并拒绝任何未消费的尾随字节。
    ///
    /// 输入切片长度本身作为 wire 上限；decoder 仍受共享的硬 claim 上限约束。
    ///
    /// # 参数
    /// - `bytes`：只包含一个完整值的 positional 字节。
    ///
    /// # 返回值
    /// 解码后的拥有型值。
    ///
    /// # 错误
    /// 编码无效或 decoder claim 超限时返回 [`PositionalDecodeError::Codec`]；此入口不会
    /// 产生切片大小错误，但完整解码仍会拒绝尾随数据。
    pub fn decode<T: DeserializeOwned>(self, bytes: &[u8]) -> Result<T, PositionalDecodeError> {
        self.decode_bounded(bytes, bytes.len() as u64)
    }

    /// 在 `max` wire 字节上限内解码完整切片中的一个值。
    ///
    /// 与 [`decode_prefix_bounded`](Self::decode_prefix_bounded) 不同，此入口要求值恰好
    /// 消耗整个切片。`max` 限制输入字节；变长整数格式的 decoder claim 另有倍率和共享
    /// 硬上限，不能把 `max` 当作逐字节等值的内存分配承诺。
    ///
    /// # 参数
    /// - `bytes`：完整 positional 输入。
    /// - `max`：允许的 wire 字节数。
    ///
    /// # 返回值
    /// 解码后的拥有型值。
    ///
    /// # 错误
    /// 超过输入上限返回 [`PositionalDecodeError::SizeLimit`]；存在尾随数据返回
    /// [`PositionalDecodeError::TrailingBytes`]；格式或 claim 失败返回
    /// [`PositionalDecodeError::Codec`]。
    pub fn decode_bounded<T: DeserializeOwned>(
        self,
        bytes: &[u8],
        max: u64,
    ) -> Result<T, PositionalDecodeError> {
        let (value, used) = self.decode_prefix_bounded(bytes, max)?;
        if used != bytes.len() {
            return Err(PositionalDecodeError::TrailingBytes {
                remaining: bytes.len() - used,
            });
        }
        Ok(value)
    }

    /// Used for version headers followed by a separately decoded payload.
    ///
    /// 此入口允许切片中还有后续字段，并返回第一个值实际消费的字节数；它不验证剩余内容。
    /// 需要完整载荷语义的调用方必须自行解码尾部，或改用完整切片入口。
    ///
    /// # 参数
    /// - `bytes`：包含待解前缀及可选后续内容的切片。
    /// - `max`：整个输入切片允许的 wire 字节上限。
    ///
    /// # 返回值
    /// `(value, used)`，其中 `used` 是前缀解码消费的字节数。
    ///
    /// # 错误
    /// 输入切片超过上限返回 [`PositionalDecodeError::SizeLimit`]；格式或 claim 失败返回
    /// [`PositionalDecodeError::Codec`]。
    pub fn decode_prefix_bounded<T: DeserializeOwned>(
        self,
        bytes: &[u8],
        max: u64,
    ) -> Result<(T, usize), PositionalDecodeError> {
        if bytes.len() as u64 > max {
            return Err(PositionalDecodeError::SizeLimit { max });
        }
        let result = match self {
            Self::FixedLittleEndian => bounded_decode::slice(
                bytes,
                max,
                bincode_next::config::legacy().with_limit::<MAX_DECODE_ALLOCATION_BYTES>(),
            ),
            Self::VariableLittleEndian => bounded_decode::slice(
                bytes,
                max,
                bincode_next::config::legacy()
                    .with_variable_int_encoding()
                    .with_limit::<MAX_DECODE_ALLOCATION_BYTES>(),
            ),
        };
        result.map_err(PositionalDecodeError::Codec)
    }

    /// The reader cap also bounds decompressed artifact data independently of
    /// compressed input size; the codec additionally bounds container claims.
    ///
    /// 只解出一个值，不要求 reader 到达 EOF，也不检查其后是否仍有字节；若格式要求整个
    /// 解压载荷恰好只有该值，调用方还须检查剩余 reader（例如 artifact store 的尾随字节检查）。
    /// `max` 限制最多可读的 wire 字节，decoder claim 另受该格式及共享上限约束。
    ///
    /// # 参数
    /// - `reader`：提供 positional 字节的流。
    /// - `max`：最多从流中读取的 wire 字节数。
    ///
    /// # 返回值
    /// 从流前缀解码出的拥有型值。
    ///
    /// # 错误
    /// 输入截断、格式错误或 claim 超限返回 [`PositionalDecodeError::Codec`]。
    pub fn decode_from<T: DeserializeOwned>(
        self,
        reader: impl Read,
        max: u64,
    ) -> Result<T, PositionalDecodeError> {
        let result = match self {
            Self::FixedLittleEndian => bounded_decode::stream(
                reader.take(max),
                max,
                bincode_next::config::legacy().with_limit::<MAX_DECODE_ALLOCATION_BYTES>(),
            ),
            Self::VariableLittleEndian => bounded_decode::stream(
                reader.take(max),
                max,
                bincode_next::config::legacy()
                    .with_variable_int_encoding()
                    .with_limit::<MAX_DECODE_ALLOCATION_BYTES>(),
            ),
        };
        result.map_err(PositionalDecodeError::Codec)
    }
}

/// 在调用方 writer 前统计剩余 wire 配额，并区分本地超限与底层写入失败。
struct LimitedWriter<'a, W> {
    // 限额内的字节最终写入此目标；已经写出的部分不会因后续错误而撤销。
    writer: &'a mut W,
    // 成功转交给底层 writer 后才递减，表示尚可接受的输出字节数。
    remaining: u64,
    // 记录本地拒写，供外层优先映射为 PositionalEncodeError::SizeLimit。
    exceeded: bool,
}

impl<W: Write> Write for LimitedWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // 先拒绝会越过总配额的整次请求，避免把部分超限内容交给底层 writer。
        if bytes.len() as u64 > self.remaining {
            self.exceeded = true;
            return Err(io::Error::other("positional byte limit exceeded"));
        }
        let written = self.writer.write(bytes)?;
        self.remaining -= written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        // 刷新仍由底层 writer 决定；限额只约束已经写入的 positional 字节。
        self.writer.flush()
    }
}

#[cfg(test)]
#[path = "tests/positional.rs"]
mod tests;
