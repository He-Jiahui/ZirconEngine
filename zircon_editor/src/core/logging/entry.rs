use std::sync::Arc;

use super::{EditorLogError, LogJump, LogSeverity, LogSource};

const MAX_LOG_MESSAGE_BYTES: usize = 8 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 尚未分配全局序号的结构化日志输入；来源与跳转供 Activity UI 投影。
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
        let message = validated_message(message.into())?;
        Ok(Self {
            source,
            severity,
            message,
            timestamp_frame,
            jump,
        })
    }

    /// 外部诊断消息无效时换用安全文案，同时保留来源、严重度与跳转目标。
    pub(crate) fn new_with_fallback(
        source: LogSource,
        severity: LogSeverity,
        message: String,
        fallback: &'static str,
        timestamp_frame: u64,
        jump: Option<LogJump>,
    ) -> Result<Self, EditorLogError> {
        let message =
            validated_message(message).or_else(|_| validated_message(fallback.to_owned()))?;
        Ok(Self {
            source,
            severity,
            message,
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

fn validated_message(message: String) -> Result<Arc<str>, EditorLogError> {
    if message.trim().is_empty() {
        return Err(EditorLogError::EmptyMessage);
    }
    if message.len() > MAX_LOG_MESSAGE_BYTES {
        return Err(EditorLogError::MessageTooLong {
            maximum: MAX_LOG_MESSAGE_BYTES,
            actual: message.len(),
        });
    }
    Ok(Arc::from(message))
}

#[cfg(test)]
#[path = "tests/entry_optimization_batch_hc_editor584_tests.rs"]
mod optimization_batch_hc_editor584_tests;
