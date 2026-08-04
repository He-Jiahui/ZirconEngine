use std::sync::Arc;
use std::time::Duration;

use crate::core::notifications::{NotificationId, NotificationSource};

use super::ToastNotificationError;

const MAX_TOAST_LIFETIME_SECS: u64 = 60 * 60;
const MAX_TOAST_KEY_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastNotification {
    id: NotificationId,
    source: NotificationSource,
    severity: ToastSeverity,
    title_key: Arc<str>,
    message_key: Arc<str>,
    lifetime: Duration,
}

impl ToastNotification {
    /// Keeps producer diagnostics within the immutable notification contract.
    pub fn bounded_message(value: &str, fallback: &str) -> String {
        let value = value.trim();
        let value = if value.is_empty() { fallback } else { value };
        if value.len() <= MAX_TOAST_KEY_BYTES {
            return value.to_string();
        }

        const ELLIPSIS: &str = "...";
        let mut end = MAX_TOAST_KEY_BYTES - ELLIPSIS.len();
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}{}", &value[..end], ELLIPSIS)
    }

    pub fn new(
        id: NotificationId,
        source: NotificationSource,
        severity: ToastSeverity,
        title_key: impl Into<String>,
        message_key: impl Into<String>,
        lifetime: Duration,
    ) -> Result<Self, ToastNotificationError> {
        if lifetime.is_zero() || lifetime > Duration::from_secs(MAX_TOAST_LIFETIME_SECS) {
            return Err(ToastNotificationError::InvalidLifetime);
        }
        Ok(Self {
            id,
            source,
            severity,
            title_key: key("title key", title_key)?,
            message_key: key("message key", message_key)?,
            lifetime,
        })
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }
    pub fn source(&self) -> &NotificationSource {
        &self.source
    }
    pub const fn severity(&self) -> ToastSeverity {
        self.severity
    }
    pub fn title_key(&self) -> &str {
        &self.title_key
    }
    pub fn message_key(&self) -> &str {
        &self.message_key
    }
    pub const fn lifetime(&self) -> Duration {
        self.lifetime
    }
}

fn key(field: &'static str, value: impl Into<String>) -> Result<Arc<str>, ToastNotificationError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(ToastNotificationError::EmptyField { field });
    }
    if value.len() > MAX_TOAST_KEY_BYTES {
        return Err(ToastNotificationError::FieldTooLong {
            field,
            maximum: MAX_TOAST_KEY_BYTES,
            actual: value.len(),
        });
    }
    Ok(Arc::from(value))
}

#[cfg(test)]
mod tests {
    use super::{MAX_TOAST_KEY_BYTES, ToastNotification};

    #[test]
    fn bounded_message_preserves_utf8_boundary_and_contract_limit() {
        let message = ToastNotification::bounded_message(&"中".repeat(MAX_TOAST_KEY_BYTES), "fallback");

        assert!(message.len() <= MAX_TOAST_KEY_BYTES);
        assert!(message.is_char_boundary(message.len()));
        assert!(message.ends_with("..."));
    }
}
