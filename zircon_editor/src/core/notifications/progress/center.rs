use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::core::jobs::{EditorJobProgressSnapshot, EditorJobProgressSource, JobId};
use crate::core::notifications::NotificationId;

use super::{ProgressNotification, ProgressNotificationError};

#[derive(Clone, Debug)]
pub struct ProgressNotificationSnapshot {
    notification: ProgressNotification,
    job: EditorJobProgressSnapshot,
}

impl ProgressNotificationSnapshot {
    fn new(notification: ProgressNotification, job: EditorJobProgressSnapshot) -> Self {
        Self { notification, job }
    }
    pub fn notification(&self) -> &ProgressNotification {
        &self.notification
    }
    pub fn job(&self) -> &EditorJobProgressSnapshot {
        &self.job
    }
}

#[derive(Default)]
pub struct ProgressNotificationCenter {
    entries: Mutex<BTreeMap<NotificationId, ProgressNotification>>,
}

impl ProgressNotificationCenter {
    pub fn publish(
        &self,
        notification: ProgressNotification,
    ) -> Result<(), ProgressNotificationError> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if entries.contains_key(notification.id()) {
            return Err(ProgressNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }
        if entries
            .values()
            .any(|entry| entry.job() == notification.job())
        {
            return Err(ProgressNotificationError::DuplicateJob {
                job: notification.job(),
            });
        }
        entries.insert(notification.id().clone(), notification);
        Ok(())
    }

    pub fn snapshot(&self, jobs: &EditorJobProgressSource) -> Vec<ProgressNotificationSnapshot> {
        self.synchronize(jobs.snapshot())
    }

    pub fn synchronize(
        &self,
        jobs: impl IntoIterator<Item = EditorJobProgressSnapshot>,
    ) -> Vec<ProgressNotificationSnapshot> {
        let jobs = jobs
            .into_iter()
            .map(|snapshot| (snapshot.id(), snapshot))
            .collect::<BTreeMap<JobId, _>>();
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.retain(|_, notification| jobs.contains_key(&notification.job()));
        entries
            .values()
            .filter_map(|notification| {
                jobs.get(&notification.job())
                    .cloned()
                    .map(|job| ProgressNotificationSnapshot::new(notification.clone(), job))
            })
            .collect()
    }
}
