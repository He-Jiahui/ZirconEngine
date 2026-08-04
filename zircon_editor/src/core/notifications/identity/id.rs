use std::fmt::{Display, Formatter};
use std::sync::Arc;

use super::NotificationIdentityError;

pub const MAX_NOTIFICATION_ID_BYTES: usize = 192;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotificationId(Arc<str>);

impl NotificationId {
    pub fn parse(value: impl Into<String>) -> Result<Self, NotificationIdentityError> {
        let value = value.into();
        if value.len() > MAX_NOTIFICATION_ID_BYTES {
            return Err(NotificationIdentityError::InvalidNotificationId(value));
        }
        let mut segment_count = 0;
        let invalid_segment = value.split('.').any(|segment| {
            segment_count += 1;
            !valid_segment(segment)
        });
        if segment_count < 3 || invalid_segment {
            return Err(NotificationIdentityError::InvalidNotificationId(value));
        }
        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NotificationId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}
