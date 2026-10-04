//! 区分短时提示内容、有效期、重复身份及容量拒绝，供宿主决定是否重试；失效提示不可借错误恢复继续占用中心容量。
use std::fmt::{Display, Formatter};

use crate::core::notifications::{NotificationId, NotificationIdentityError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastNotificationError {
    Identity(NotificationIdentityError),
    EmptyField {
        field: &'static str,
    },
    FieldTooLong {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },
    InvalidLifetime,
    InvalidCapacity,
    DuplicateNotification {
        notification: NotificationId,
    },
    CapacityReached {
        capacity: usize,
    },
}

impl Display for ToastNotificationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identity(error) => error.fmt(formatter),
            Self::EmptyField { field } => write!(formatter, "toast {field} must not be empty"),
            Self::FieldTooLong {
                field,
                maximum,
                actual,
            } => write!(
                formatter,
                "toast {field} is {actual} bytes; maximum is {maximum} bytes"
            ),
            Self::InvalidLifetime => {
                formatter.write_str("toast lifetime must be between one second and one hour")
            }
            Self::InvalidCapacity => {
                formatter.write_str("toast capacity must be greater than zero")
            }
            Self::DuplicateNotification { notification } => {
                write!(formatter, "toast `{notification}` already exists")
            }
            Self::CapacityReached { capacity } => {
                write!(formatter, "toast capacity {capacity} is full")
            }
        }
    }
}

impl std::error::Error for ToastNotificationError {}

impl From<NotificationIdentityError> for ToastNotificationError {
    fn from(error: NotificationIdentityError) -> Self {
        Self::Identity(error)
    }
}
