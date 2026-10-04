//! 报告通知身份、重复任务绑定和容量失败，使进度生产者能够区分替换规则与拒绝准入；任务真实状态仍由任务系统拥有。
use std::fmt::{Display, Formatter};

use crate::core::jobs::JobId;
use crate::core::notifications::{NotificationId, NotificationIdentityError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressNotificationError {
    Identity(NotificationIdentityError),
    EmptyField {
        field: &'static str,
    },
    FieldTooLong {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },
    DuplicateNotification {
        notification: NotificationId,
    },
    DuplicateJob {
        job: JobId,
    },
    CapacityExceeded {
        maximum: usize,
    },
}

impl Display for ProgressNotificationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identity(error) => error.fmt(formatter),
            Self::EmptyField { field } => write!(formatter, "progress {field} must not be empty"),
            Self::FieldTooLong {
                field,
                maximum,
                actual,
            } => write!(
                formatter,
                "progress {field} is {actual} bytes; maximum is {maximum} bytes"
            ),
            Self::DuplicateNotification { notification } => {
                write!(formatter, "progress `{notification}` already exists")
            }
            Self::DuplicateJob { job } => write!(
                formatter,
                "job {} already has a progress notification",
                job.value()
            ),
            Self::CapacityExceeded { maximum } => {
                write!(formatter, "progress notification capacity is {maximum}")
            }
        }
    }
}

impl std::error::Error for ProgressNotificationError {}

impl From<NotificationIdentityError> for ProgressNotificationError {
    fn from(error: NotificationIdentityError) -> Self {
        Self::Identity(error)
    }
}
