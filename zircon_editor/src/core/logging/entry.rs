use std::sync::Arc;

use super::{EditorLogError, LogJump, LogSeverity, LogSource};

const MAX_LOG_MESSAGE_BYTES: usize = 8 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    source: LogSource,
    severity: LogSeverity,
    message: Arc<str>,
    timestamp_frame: u64,
    jump: Option<LogJump>,
}

impl LogEntry {
    pub fn new(
        source: LogSource,
        severity: LogSeverity,
        message: impl Into<String>,
        timestamp_frame: u64,
        jump: Option<LogJump>,
    ) -> Result<Self, EditorLogError> {
        let message = message.into();
        if message.trim().is_empty() {
            return Err(EditorLogError::EmptyMessage);
        }
        if message.len() > MAX_LOG_MESSAGE_BYTES {
            return Err(EditorLogError::MessageTooLong {
                maximum: MAX_LOG_MESSAGE_BYTES,
                actual: message.len(),
            });
        }
        Ok(Self {
            source,
            severity,
            message: Arc::from(message),
            timestamp_frame,
            jump,
        })
    }

    pub fn source(&self) -> &LogSource {
        &self.source
    }

    pub const fn severity(&self) -> LogSeverity {
        self.severity
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn timestamp_frame(&self) -> u64 {
        self.timestamp_frame
    }

    pub fn jump(&self) -> Option<&LogJump> {
        self.jump.as_ref()
    }

    pub fn estimated_bytes(&self) -> usize {
        self.source.estimated_bytes()
            + self.message.len()
            + self.jump.as_ref().map_or(0, LogJump::estimated_bytes)
            + std::mem::size_of::<u64>()
    }
}
