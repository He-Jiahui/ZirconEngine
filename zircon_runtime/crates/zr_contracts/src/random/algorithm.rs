use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Explicit algorithm identity persisted with every random-stream snapshot.
///
/// New algorithms require a new variant instead of changing the behavior of an
/// existing stream identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum RandomAlgorithmId {
    Pcg32XshRrV1 = 1,
}

impl RandomAlgorithmId {
    /// Stable algorithm value written into a versioned persistence boundary.
    pub const fn stable_id(self) -> u16 {
        self as u16
    }

    /// Decodes an algorithm identity without treating an unknown future value
    /// as the current algorithm.
    pub const fn from_stable_id(value: u16) -> Result<Self, RandomAlgorithmIdError> {
        match value {
            1 => Ok(Self::Pcg32XshRrV1),
            _ => Err(RandomAlgorithmIdError::UnsupportedStableId { value }),
        }
    }

    pub const fn version(self) -> u16 {
        match self {
            Self::Pcg32XshRrV1 => 1,
        }
    }
}

impl Serialize for RandomAlgorithmId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.stable_id().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RandomAlgorithmId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stable_id = u16::deserialize(deserializer)?;
        Self::from_stable_id(stable_id).map_err(serde::de::Error::custom)
    }
}

/// Rejection emitted when persisted data names an algorithm this runtime does not support.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomAlgorithmIdError {
    #[error("unsupported random algorithm stable id {value}")]
    UnsupportedStableId { value: u16 },
}

/// Stable identity for one of the independent streams supported by PCG32.
///
/// PCG's 64-bit state transition encodes a stream as an odd increment. The
/// low bit is therefore reserved and exactly 63 bits remain for stream
/// identity. Keeping that limit in the type prevents two caller-supplied
/// `u64` values that differ only in the high bit from silently selecting the
/// same stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandomSequenceId(u64);

impl RandomSequenceId {
    pub const MAX_VALUE: u64 = u64::MAX >> 1;

    pub const fn new(value: u64) -> Result<Self, RandomSequenceIdError> {
        if value > Self::MAX_VALUE {
            return Err(RandomSequenceIdError::OutOfRange { value });
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    /// Reduces a uniformly distributed derivation word into the 63-bit PCG
    /// stream space without modulo bias.
    pub(crate) const fn from_uniform_u64(value: u64) -> Self {
        Self(value & Self::MAX_VALUE)
    }
}

impl Serialize for RandomSequenceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RandomSequenceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomSequenceIdError {
    #[error("PCG32 random sequence id {value} exceeds the 63-bit stream space")]
    OutOfRange { value: u64 },
}
