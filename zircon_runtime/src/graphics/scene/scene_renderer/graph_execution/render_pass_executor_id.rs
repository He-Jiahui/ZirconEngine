use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPassExecutorId(String);

impl RenderPassExecutorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderPassExecutorId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderPassExecutorId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for RenderPassExecutorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
