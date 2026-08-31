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
        if !valid_notification_id_syntax(&value) {
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

fn valid_notification_id_syntax(value: &str) -> bool {
    let mut segment_count = 1usize;
    let mut segment_has_value = false;
    for byte in value.bytes() {
        match byte {
            b'.' if segment_has_value => {
                segment_count = segment_count.saturating_add(1);
                segment_has_value = false;
            }
            b'.' => return false,
            b'a'..=b'z' | b'0'..=b'9' | b'_' => segment_has_value = true,
            _ => return false,
        }
    }
    segment_has_value && segment_count >= 3
}

#[cfg(test)]
#[path = "id/single_pass_validation_tests.rs"]
mod single_pass_validation_tests;
