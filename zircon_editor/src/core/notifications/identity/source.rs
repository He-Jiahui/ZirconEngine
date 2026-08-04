use std::sync::Arc;

use super::NotificationIdentityError;

pub const MAX_NOTIFICATION_SOURCE_ID_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationSourceKind {
    Builtin,
    Plugin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationSource {
    kind: NotificationSourceKind,
    id: Arc<str>,
}

impl NotificationSource {
    pub fn builtin(owner: impl Into<String>) -> Result<Self, NotificationIdentityError> {
        Self::new(NotificationSourceKind::Builtin, owner)
    }

    pub fn plugin(plugin_id: impl Into<String>) -> Result<Self, NotificationIdentityError> {
        Self::new(NotificationSourceKind::Plugin, plugin_id)
    }

    pub const fn kind(&self) -> NotificationSourceKind {
        self.kind
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn new(
        kind: NotificationSourceKind,
        id: impl Into<String>,
    ) -> Result<Self, NotificationIdentityError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(NotificationIdentityError::EmptySourceId);
        }
        if id.len() > MAX_NOTIFICATION_SOURCE_ID_BYTES {
            return Err(NotificationIdentityError::SourceIdTooLong {
                maximum: MAX_NOTIFICATION_SOURCE_ID_BYTES,
                actual: id.len(),
            });
        }
        Ok(Self {
            kind,
            id: Arc::from(id),
        })
    }
}
