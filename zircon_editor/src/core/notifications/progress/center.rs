use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::core::jobs::{EditorJobProgressSnapshot, EditorJobProgressSource, JobId};
use crate::core::notifications::{NotificationId, NotificationSourceKind};

use super::{ProgressNotification, ProgressNotificationError};

pub const MAX_PROGRESS_NOTIFICATIONS: usize = 64;
pub(crate) const AUTOMATIC_PROGRESS_SOURCE_ID: &str = "editor.jobs";

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

pub struct ProgressNotificationCenter {
    entries: Mutex<BTreeMap<NotificationId, ProgressNotification>>,
}

impl Default for ProgressNotificationCenter {
    fn default() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }
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
        // The job-system fallback is intentionally replaceable by a source-specific producer.
        if let Some(existing) = entries.get(notification.id()) {
            if existing.job() == notification.job()
                && is_automatic_binding(existing)
                && !is_automatic_binding(&notification)
            {
                entries.insert(notification.id().clone(), notification);
                return Ok(());
            }
            return Err(ProgressNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }
        if let Some((existing_id, automatic)) = entries
            .iter()
            .find(|(_, entry)| entry.job() == notification.job())
            .map(|(id, entry)| (id.clone(), is_automatic_binding(entry)))
        {
            if automatic && !is_automatic_binding(&notification) {
                entries.remove(&existing_id);
                entries.insert(notification.id().clone(), notification);
                return Ok(());
            }
            return Err(ProgressNotificationError::DuplicateJob {
                job: notification.job(),
            });
        }
        if entries.len() >= MAX_PROGRESS_NOTIFICATIONS {
            return Err(ProgressNotificationError::CapacityExceeded {
                maximum: MAX_PROGRESS_NOTIFICATIONS,
            });
        }
        entries.insert(notification.id().clone(), notification);
        Ok(())
    }

    pub fn snapshot(&self, jobs: &EditorJobProgressSource) -> Vec<ProgressNotificationSnapshot> {
        let (captured, ids) = {
            let entries = self
                .entries
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if entries.is_empty() {
                return Vec::new();
            }
            (
                // Keep the job identity with the stable producer key so a replacement
                // notification using the same ID cannot be pruned as stale.
                entries
                    .iter()
                    .map(|(id, notification)| (id.clone(), notification.job()))
                    .collect::<BTreeMap<_, _>>(),
                entries
                    .values()
                    .map(ProgressNotification::job)
                    .collect::<Vec<_>>(),
            )
        };
        self.synchronize_captured(&captured, jobs.snapshot_for_ids(ids))
    }

    pub fn is_empty(&self) -> bool {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    }

    pub fn remaining_capacity(&self) -> usize {
        MAX_PROGRESS_NOTIFICATIONS.saturating_sub(
            self.entries
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .len(),
        )
    }

    /// Removes an authoritative job binding after its lifecycle has terminally completed.
    pub fn retire_job(&self, job: JobId) {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|_, notification| notification.job() != job);
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

    fn synchronize_captured(
        &self,
        captured: &BTreeMap<NotificationId, JobId>,
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
        entries.retain(|id, notification| {
            let Some(captured_job) = captured.get(id) else {
                return true;
            };
            notification.job() != *captured_job || jobs.contains_key(&notification.job())
        });
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

fn is_automatic_binding(notification: &ProgressNotification) -> bool {
    notification.source().kind() == NotificationSourceKind::Builtin
        && notification.source().id() == AUTOMATIC_PROGRESS_SOURCE_ID
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::core::jobs::{EditorJobProgressSnapshot, JobCategory, JobId};
    use crate::core::notifications::{NotificationId, NotificationSource};

    use super::{ProgressNotification, ProgressNotificationCenter};

    fn notification(id: &str, job: JobId) -> ProgressNotification {
        ProgressNotification::new(
            NotificationId::parse(id).unwrap(),
            NotificationSource::builtin("editor.progress.test").unwrap(),
            job,
            "editor.progress.title",
        )
        .unwrap()
    }

    fn job(id: JobId) -> EditorJobProgressSnapshot {
        EditorJobProgressSnapshot::new(id, "job", JobCategory::Import, None, true)
    }

    #[test]
    fn captured_synchronization_does_not_remove_bindings_added_after_capture() {
        let center = ProgressNotificationCenter::default();
        let captured_id = NotificationId::parse("editor.progress.captured").unwrap();
        let captured_job = JobId::new(1);
        let later_job = JobId::new(2);
        center
            .publish(notification(captured_id.as_str(), captured_job))
            .unwrap();
        let captured = BTreeMap::from([(captured_id, captured_job)]);
        center
            .publish(notification("editor.progress.later", later_job))
            .unwrap();

        let projected = center.synchronize_captured(&captured, [job(captured_job)]);

        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].job().id(), captured_job);
        assert_eq!(center.synchronize([job(later_job)]).len(), 1);
    }

    #[test]
    fn captured_synchronization_preserves_a_reused_id_bound_to_a_new_job() {
        let center = ProgressNotificationCenter::default();
        let id = NotificationId::parse("editor.progress.reused").unwrap();
        let retired_job = JobId::new(1);
        let replacement_job = JobId::new(2);
        center
            .publish(notification(id.as_str(), retired_job))
            .unwrap();
        let captured = BTreeMap::from([(id.clone(), retired_job)]);
        center.retire_job(retired_job);
        center
            .publish(notification(id.as_str(), replacement_job))
            .unwrap();

        assert!(center
            .synchronize_captured(&captured, std::iter::empty::<EditorJobProgressSnapshot>(),)
            .is_empty());
        assert_eq!(center.synchronize([job(replacement_job)]).len(), 1);
    }
}
