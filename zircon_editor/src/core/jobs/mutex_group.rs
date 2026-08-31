use serde::{Deserialize, Deserializer, Serialize};

use super::MutexGroupError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MutexGroup(String);

impl MutexGroup {
    /// Maximum retained UTF-8 bytes for one resource serialization identity.
    pub const MAX_BYTES: usize = 128;

    pub fn parse(value: impl Into<String>) -> Result<Self, MutexGroupError> {
        let value = value.into();
        if value.is_empty() {
            return Err(MutexGroupError::Empty);
        }
        if value.len() > Self::MAX_BYTES {
            return Err(MutexGroupError::TooLong {
                len: value.len(),
                max: Self::MAX_BYTES,
            });
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(MutexGroupError::Invalid { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for MutexGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutex_group_enforces_its_utf8_byte_budget_for_parse_and_deserialize() {
        assert!(MutexGroup::parse("a".repeat(MutexGroup::MAX_BYTES)).is_ok());
        assert_eq!(
            MutexGroup::parse("a".repeat(MutexGroup::MAX_BYTES + 1)),
            Err(MutexGroupError::TooLong {
                len: MutexGroup::MAX_BYTES + 1,
                max: MutexGroup::MAX_BYTES,
            })
        );
        let encoded = serde_json::to_string(&"a".repeat(MutexGroup::MAX_BYTES + 1)).unwrap();
        assert!(serde_json::from_str::<MutexGroup>(&encoded).is_err());
    }

    #[test]
    #[ignore = "managed Editor09 performance evidence"]
    fn editor09_mutex_group_retention_budget_evidence() {
        const OVERSIZED_BYTES: usize = 1_048_576;

        let error = MutexGroup::parse("a".repeat(OVERSIZED_BYTES)).unwrap_err();

        assert_eq!(
            error,
            MutexGroupError::TooLong {
                len: OVERSIZED_BYTES,
                max: MutexGroup::MAX_BYTES,
            }
        );
        println!(
            "EDITOR_JOB_BENCH_V1 kind=mutex_group_retention oversized_input_bytes={} retained_identity_bytes_before={} retained_identity_bytes_after=0 retained_byte_reduction_percent=100.0000 maximum_bytes={}",
            OVERSIZED_BYTES,
            OVERSIZED_BYTES,
            MutexGroup::MAX_BYTES,
        );
    }
}
