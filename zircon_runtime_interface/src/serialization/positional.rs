//! Explicit positional formats used by existing asset files, wire envelopes,
//! and cache keys. Changing the codec must not change their persisted bytes.

use std::io::{self, Read, Write};

use serde::{de::DeserializeOwned, Serialize};

mod bounded_decode;

// Match the largest admitted cooked-artifact payload. Individual formats keep
// their smaller byte caps; a shared codec must not impose a new 256 MiB cutoff.
const MAX_DECODE_ALLOCATION_BYTES: usize = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionalEncoding {
    FixedLittleEndian,
    VariableLittleEndian,
}

#[derive(Debug, thiserror::Error)]
pub enum PositionalEncodeError {
    #[error("positional payload exceeds {max} bytes")]
    SizeLimit { max: u64 },
    #[error("encode positional payload: {0}")]
    Codec(#[source] bincode_next::error::EncodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum PositionalDecodeError {
    #[error("positional payload exceeds {max} bytes")]
    SizeLimit { max: u64 },
    #[error("positional payload has {remaining} trailing bytes")]
    TrailingBytes { remaining: usize },
    #[error("decode positional payload: {0}")]
    Codec(#[source] bincode_next::error::DecodeError),
}

impl PositionalEncoding {
    pub fn encode<T: Serialize + ?Sized>(
        self,
        value: &T,
    ) -> Result<Vec<u8>, PositionalEncodeError> {
        self.encode_bounded(value, u64::MAX)
    }

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
        if writer.exceeded {
            return Err(PositionalEncodeError::SizeLimit { max });
        }
        result.map_err(PositionalEncodeError::Codec)
    }

    pub fn decode<T: DeserializeOwned>(self, bytes: &[u8]) -> Result<T, PositionalDecodeError> {
        self.decode_bounded(bytes, bytes.len() as u64)
    }

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

struct LimitedWriter<'a, W> {
    writer: &'a mut W,
    remaining: u64,
    exceeded: bool,
}

impl<W: Write> Write for LimitedWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() as u64 > self.remaining {
            self.exceeded = true;
            return Err(io::Error::other("positional byte limit exceeded"));
        }
        let written = self.writer.write(bytes)?;
        self.remaining -= written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
#[path = "tests/positional.rs"]
mod tests;
