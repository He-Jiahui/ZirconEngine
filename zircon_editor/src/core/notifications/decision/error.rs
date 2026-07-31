use std::fmt::{Display, Formatter};

use super::{
    DecisionCenterInstanceId, DecisionOptionId, DecisionReceiptCursor, DecisionReceiptSequence,
    NotificationId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecisionNotificationError {
    InvalidNotificationId(String),
    InvalidOptionId(String),
    EmptyField {
        field: &'static str,
    },
    FieldTooLong {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },
    AtLeastTwoOptionsRequired,
    TooManyOptions {
        maximum: usize,
        actual: usize,
    },
    DuplicateOption {
        option: DecisionOptionId,
    },
    OptionNotFound {
        notification: NotificationId,
        option: DecisionOptionId,
    },
    InvalidCapacity {
        field: &'static str,
    },
    DuplicateNotification {
        notification: NotificationId,
    },
    PendingCapacityReached {
        capacity: usize,
    },
    NotificationNotFound {
        notification: NotificationId,
    },
    StaleTicket {
        notification: NotificationId,
        expected_incarnation: u64,
        received_incarnation: u64,
    },
    ForeignTicket {
        notification: NotificationId,
        expected_center: DecisionCenterInstanceId,
        received_center: DecisionCenterInstanceId,
    },
    ForeignCursor {
        expected_center: DecisionCenterInstanceId,
        received_center: DecisionCenterInstanceId,
    },
    AlreadyResolved {
        notification: NotificationId,
        selected: DecisionOptionId,
        requested: DecisionOptionId,
    },
    CancellationNotAllowed {
        notification: NotificationId,
    },
    ReceiptSequenceExhausted,
    TicketSequenceExhausted,
    CenterInstanceExhausted,
    CursorExpired {
        requested: u64,
        oldest_available: DecisionReceiptSequence,
        resume_cursor: DecisionReceiptCursor,
    },
}

impl Display for DecisionNotificationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNotificationId(id) => {
                write!(formatter, "notification id `{id}` is invalid")
            }
            Self::InvalidOptionId(id) => write!(formatter, "decision option id `{id}` is invalid"),
            Self::EmptyField { field } => write!(formatter, "decision {field} must not be empty"),
            Self::FieldTooLong {
                field,
                maximum,
                actual,
            } => write!(
                formatter,
                "decision {field} is {actual} bytes; maximum is {maximum} bytes"
            ),
            Self::AtLeastTwoOptionsRequired => {
                formatter.write_str("decision notifications require at least two options")
            }
            Self::TooManyOptions { maximum, actual } => write!(
                formatter,
                "decision notification has {actual} options; maximum is {maximum}"
            ),
            Self::DuplicateOption { option } => {
                write!(formatter, "decision option `{option}` is duplicated")
            }
            Self::OptionNotFound {
                notification,
                option,
            } => write!(
                formatter,
                "decision `{notification}` has no option `{option}`"
            ),
            Self::InvalidCapacity { field } => {
                write!(
                    formatter,
                    "decision center {field} must be greater than zero"
                )
            }
            Self::DuplicateNotification { notification } => {
                write!(
                    formatter,
                    "decision notification `{notification}` already exists"
                )
            }
            Self::PendingCapacityReached { capacity } => write!(
                formatter,
                "decision notification pending capacity {capacity} is full"
            ),
            Self::NotificationNotFound { notification } => {
                write!(
                    formatter,
                    "decision notification `{notification}` was not found"
                )
            }
            Self::StaleTicket {
                notification,
                expected_incarnation,
                received_incarnation,
            } => write!(
                formatter,
                "decision ticket for `{notification}` is stale: expected incarnation {expected_incarnation}, received {received_incarnation}"
            ),
            Self::ForeignTicket {
                notification,
                expected_center,
                received_center,
            } => write!(
                formatter,
                "decision ticket for `{notification}` belongs to center {}, not center {}",
                received_center.value(),
                expected_center.value()
            ),
            Self::ForeignCursor {
                expected_center,
                received_center,
            } => write!(
                formatter,
                "decision receipt cursor belongs to center {}, not center {}",
                received_center.value(),
                expected_center.value()
            ),
            Self::AlreadyResolved {
                notification,
                selected,
                requested,
            } => write!(
                formatter,
                "decision `{notification}` is already resolved as `{selected}`, not `{requested}`"
            ),
            Self::CancellationNotAllowed { notification } => write!(
                formatter,
                "decision notification `{notification}` has no cancellation option"
            ),
            Self::ReceiptSequenceExhausted => {
                formatter.write_str("decision receipt sequence is exhausted")
            }
            Self::TicketSequenceExhausted => {
                formatter.write_str("decision ticket sequence is exhausted")
            }
            Self::CenterInstanceExhausted => {
                formatter.write_str("decision center instance sequence is exhausted")
            }
            Self::CursorExpired {
                requested,
                oldest_available,
                resume_cursor: _,
            } => write!(
                formatter,
                "decision receipt cursor {requested} expired; oldest available sequence is {}",
                oldest_available.value()
            ),
        }
    }
}

impl std::error::Error for DecisionNotificationError {}
