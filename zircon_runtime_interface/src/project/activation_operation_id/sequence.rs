use serde::{Deserialize, Deserializer, Serialize};

use super::ProjectActivationOperationSequenceError;

/// A non-zero sequence number allocated monotonically by one launch instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ProjectActivationOperationSequence(u64);

impl ProjectActivationOperationSequence {
    pub const fn new(value: u64) -> Option<Self> {
        match value {
            0 => None,
            _ => Some(Self(value)),
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ProjectActivationOperationSequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value)
            .ok_or(ProjectActivationOperationSequenceError::Zero)
            .map_err(serde::de::Error::custom)
    }
}
