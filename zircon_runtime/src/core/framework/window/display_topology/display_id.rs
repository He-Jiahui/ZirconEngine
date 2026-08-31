use std::error::Error;
use std::fmt;
use std::sync::Arc;

const MAX_DISPLAY_KEY_BYTES: usize = 512;

/// A backend-provided stable identity for one physical display, logical screen,
/// or render output. It is not a topology-local index and is deliberately not
/// serialized as a live host handle.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DisplayId {
    kind: DisplayKind,
    stable_key: Arc<str>,
}

impl DisplayId {
    pub fn new(
        kind: DisplayKind,
        stable_key: impl Into<Arc<str>>,
    ) -> Result<Self, DisplayIdentityError> {
        let stable_key = stable_key.into();
        if stable_key.trim().is_empty() {
            return Err(DisplayIdentityError::Empty);
        }
        if stable_key.len() > MAX_DISPLAY_KEY_BYTES {
            return Err(DisplayIdentityError::TooLong {
                maximum_bytes: MAX_DISPLAY_KEY_BYTES,
                actual_bytes: stable_key.len(),
            });
        }
        Ok(Self { kind, stable_key })
    }

    pub const fn kind(&self) -> DisplayKind {
        self.kind
    }

    pub fn as_str(&self) -> &str {
        &self.stable_key
    }
}

impl fmt::Display for DisplayId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.kind, self.as_str())
    }
}

/// The display domain addressed by a stable backend key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplayKind {
    PhysicalOutput,
    LogicalScreen,
    RenderOutput,
}

impl fmt::Display for DisplayKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::PhysicalOutput => "physical_output",
            Self::LogicalScreen => "logical_screen",
            Self::RenderOutput => "render_output",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayIdentityError {
    Empty,
    TooLong {
        maximum_bytes: usize,
        actual_bytes: usize,
    },
}

impl fmt::Display for DisplayIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("display identity must not be empty"),
            Self::TooLong {
                maximum_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "display identity is {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit"
            ),
        }
    }
}

impl Error for DisplayIdentityError {}
