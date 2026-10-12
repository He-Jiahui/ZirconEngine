//! Apply each format's runtime byte limit before the codec allocates a
//! length-prefixed container. A bounded Read alone checks too late.

use std::io::Read;

use bincode_next::config::{Config, IntEncoding};
use bincode_next::de::read::{BorrowReader, Reader, SliceReader};
use bincode_next::de::{BorrowDecoder, Decoder, DecoderImpl};
use bincode_next::error::DecodeError;
use bincode_next::serde::{BorrowCompat, Compat};
use bincode_next::{BorrowDecode, Decode, IoReader};
use serde::de::DeserializeOwned;

use super::MAX_DECODE_ALLOCATION_BYTES;

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

struct BoundedDecoder<D> {
    inner: D,
    claimed: usize,
    max: usize,
    wire_max: usize,
}

impl<R: ReadProgress, C: Config> BoundedDecoder<DecoderImpl<R, C, ()>> {
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
impl<D: Decoder> bincode_next::utils::Sealed for BoundedDecoder<D> {}
impl<D: Decoder> bincode_next::error_path::BincodeErrorPathCovered<0> for BoundedDecoder<D> {
    const ASSERT: () = <D as bincode_next::error_path::BincodeErrorPathCovered<0>>::ASSERT;
}

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
        self.claimed -= n;
        self.inner.unclaim_bytes_read(n);
    }

    fn claim_container_read<T>(&mut self, len: usize) -> Result<(), DecodeError> {
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

impl<'de, D: BorrowDecoder<'de>> BorrowDecoder<'de> for BoundedDecoder<D>
where
    D::R: ReadProgress,
{
    type BR = D::BR;
    fn borrow_reader(&mut self) -> &mut Self::BR {
        self.inner.borrow_reader()
    }
}

struct CountedSlice<'de> {
    inner: SliceReader<'de>,
    consumed: usize,
}

trait ReadProgress: Reader {
    fn consumed(&self) -> usize;
}

impl ReadProgress for CountedSlice<'_> {
    fn consumed(&self) -> usize {
        self.consumed
    }
}

impl Reader for CountedSlice<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.inner.read(bytes)?;
        self.consumed += bytes.len();
        Ok(())
    }

    fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
        self.inner.peek_read(n)
    }

    fn consume(&mut self, n: usize) {
        self.inner.consume(n);
        self.consumed += n;
    }
}

impl<'de> BorrowReader<'de> for CountedSlice<'de> {
    fn take_bytes(&mut self, n: usize) -> Result<&'de [u8], DecodeError> {
        let bytes = self.inner.take_bytes(n)?;
        self.consumed += n;
        Ok(bytes)
    }
}

struct CountedStream<R> {
    inner: IoReader<R>,
    consumed: usize,
}

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
