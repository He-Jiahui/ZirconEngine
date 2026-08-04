use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationIdentityError {
    InvalidNotificationId(String),
    EmptySourceId,
    SourceIdTooLong { maximum: usize, actual: usize },
}

impl Display for NotificationIdentityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNotificationId(id) => {
                write!(formatter, "notification id `{id}` is invalid")
            }
            Self::EmptySourceId => formatter.write_str("notification source id must not be empty"),
            Self::SourceIdTooLong { maximum, actual } => write!(
                formatter,
                "notification source id is {actual} bytes; maximum is {maximum} bytes"
            ),
        }
    }
}

impl std::error::Error for NotificationIdentityError {}
