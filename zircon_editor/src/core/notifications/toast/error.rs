//! 区分短时提示内容、有效期、重复身份及容量拒绝，供宿主决定是否重试；失效提示不可借错误恢复继续占用中心容量。
use std::fmt::{Display, Formatter};

use crate::core::notifications::{NotificationId, NotificationIdentityError};

use super::model::TOAST_LIFETIME_POLICY;

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
    PresentationSequenceExhausted,
    PresentationClockOverflow,
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
            Self::InvalidLifetime => write!(
                formatter,
                "toast lifetime must be {TOAST_LIFETIME_POLICY}; publication time plus lifetime must not overflow"
            ),
            Self::InvalidCapacity => {
                formatter.write_str("toast capacity must be greater than zero")
            }
            Self::PresentationSequenceExhausted => formatter.write_str("toast presentation sequence is exhausted"),
            Self::PresentationClockOverflow => formatter.write_str("toast first-presentation time plus lifetime must not overflow"),
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
