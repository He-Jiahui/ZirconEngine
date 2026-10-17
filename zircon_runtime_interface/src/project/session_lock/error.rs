use std::fmt;

/// Strict validation failure for the versioned project-session admission record.
/// 会话记录解码或跨字段状态校验失败；消息用于诊断，不带平台锁句柄或可恢复动作。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionAdmissionRecordError {
    message: String,
}

impl ProjectSessionAdmissionRecordError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ProjectSessionAdmissionRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ProjectSessionAdmissionRecordError {}
