use std::sync::Arc;

use crate::core::jobs::{JobId, JobTicket};
use crate::core::notifications::{NotificationId, NotificationSource};

use super::ProgressNotificationError;

const MAX_PROGRESS_KEY_BYTES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressNotification {
    id: NotificationId,
    source: NotificationSource,
    job: JobId,
    title_key: Arc<str>,
}

impl ProgressNotification {
    /// Binds a progress projection to a submitted job without retaining its result receiver.
    pub fn from_ticket<T>(
        id: NotificationId,
        source: NotificationSource,
        ticket: &JobTicket<T>,
        title_key: impl Into<String>,
    ) -> Result<Self, ProgressNotificationError> {
        Self::new(id, source, ticket.id(), title_key)
    }

    pub fn new(
        id: NotificationId,
        source: NotificationSource,
        job: JobId,
        title_key: impl Into<String>,
    ) -> Result<Self, ProgressNotificationError> {
        let title_key = title_key.into();
        if title_key.trim().is_empty() {
            return Err(ProgressNotificationError::EmptyField { field: "title key" });
        }
        if title_key.len() > MAX_PROGRESS_KEY_BYTES {
            return Err(ProgressNotificationError::FieldTooLong {
                field: "title key",
                maximum: MAX_PROGRESS_KEY_BYTES,
                actual: title_key.len(),
            });
        }
        Ok(Self {
            id,
            source,
            job,
            title_key: Arc::from(title_key),
        })
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }
    pub fn source(&self) -> &NotificationSource {
        &self.source
    }
    pub const fn job(&self) -> JobId {
        self.job
    }
    pub fn title_key(&self) -> &str {
        &self.title_key
    }
}
