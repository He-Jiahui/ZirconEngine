use std::fmt::{Display, Formatter};
use std::sync::Arc;

use super::EditorLogError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogJump(LogJumpTarget);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogJumpTarget {
    Asset(Arc<str>),
    ScriptLocation {
        path: Arc<str>,
        line: u32,
        column: u32,
    },
}

impl LogJump {
    pub fn asset(path: impl Into<String>) -> Result<Self, EditorLogError> {
        Ok(Self(LogJumpTarget::Asset(required_target(path)?)))
    }

    pub fn script_location(
        path: impl Into<String>,
        line: u32,
        column: u32,
    ) -> Result<Self, EditorLogError> {
        Ok(Self(LogJumpTarget::ScriptLocation {
            path: required_target(path)?,
            line,
            column,
        }))
    }

    pub fn target(&self) -> &LogJumpTarget {
        &self.0
    }

    pub(super) fn estimated_bytes(&self) -> usize {
        match &self.0 {
            LogJumpTarget::Asset(path) => path.len(),
            LogJumpTarget::ScriptLocation { path, .. } => {
                path.len() + 2 * std::mem::size_of::<u32>()
            }
        }
    }
}

impl Display for LogJump {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            LogJumpTarget::Asset(path) => write!(formatter, "asset:{path}"),
            LogJumpTarget::ScriptLocation { path, line, column } => {
                write!(formatter, "script:{path}:{line}:{column}")
            }
        }
    }
}

fn required_target(value: impl Into<String>) -> Result<Arc<str>, EditorLogError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(EditorLogError::EmptyJumpTarget);
    }
    Ok(Arc::from(value))
}
