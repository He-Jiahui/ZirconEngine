use std::fmt;

use thiserror::Error;

const MAX_TOOLKIT_INSTANCE_ID_BYTES: usize = 256;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolkitInstanceId(String);

impl ToolkitInstanceId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ToolkitInstanceIdError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ToolkitInstanceIdError::Empty);
        }
        if value.len() > MAX_TOOLKIT_INSTANCE_ID_BYTES {
            return Err(ToolkitInstanceIdError::TooLong {
                len: value.len(),
                max: MAX_TOOLKIT_INSTANCE_ID_BYTES,
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ToolkitInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ToolkitInstanceId")
            .field(&self.0)
            .finish()
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ToolkitInstanceIdError {
    #[error("document toolkit instance id cannot be empty")]
    Empty,
    #[error("document toolkit instance id length {len} exceeds {max}")]
    TooLong { len: usize, max: usize },
}
