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
    state: Mutex<ProgressNotificationState>,
}

#[derive(Default)]
struct ProgressNotificationState {
    entries: BTreeMap<NotificationId, ProgressNotification>,
    jobs: BTreeMap<JobId, NotificationId>,
    #[cfg(test)]
    job_lookup_probes: usize,
}

impl Default for ProgressNotificationCenter {
    fn default() -> Self {
        Self {
            state: Mutex::new(ProgressNotificationState::default()),
        }
    }
}

impl ProgressNotificationCenter {
    pub fn publish(
        &self,
        notification: ProgressNotification,
    ) -> Result<(), ProgressNotificationError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // The job-system fallback is intentionally replaceable by a source-specific producer.
        if let Some((existing_job, existing_is_automatic)) = state
            .entries
            .get(notification.id())
            .map(|existing| (existing.job(), is_automatic_binding(existing)))
        {
            if existing_job == notification.job()
                && existing_is_automatic
                && !is_automatic_binding(&notification)
            {
                state
                    .entries
                    .insert(notification.id().clone(), notification);
                return Ok(());
            }
            return Err(ProgressNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }

        #[cfg(test)]
        {
            state.job_lookup_probes = state.job_lookup_probes.saturating_add(1);
        }
        if let Some(existing_id) = state.jobs.get(&notification.job()).cloned() {
            let existing_is_automatic = state
                .entries
                .get(&existing_id)
                .map(is_automatic_binding)
                .unwrap_or(false);
            if existing_is_automatic && !is_automatic_binding(&notification) {
                state.entries.remove(&existing_id);
                state
                    .jobs
                    .insert(notification.job(), notification.id().clone());
                state
                    .entries
                    .insert(notification.id().clone(), notification);
                return Ok(());
            }
            return Err(ProgressNotificationError::DuplicateJob {
                job: notification.job(),
            });
        }
        if state.entries.len() >= MAX_PROGRESS_NOTIFICATIONS {
            return Err(ProgressNotificationError::CapacityExceeded {
                maximum: MAX_PROGRESS_NOTIFICATIONS,
            });
        }
        state
            .jobs
            .insert(notification.job(), notification.id().clone());
        state
            .entries
            .insert(notification.id().clone(), notification);
        Ok(())
    }

    pub fn snapshot(&self, jobs: &EditorJobProgressSource) -> Vec<ProgressNotificationSnapshot> {
        let captured = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.entries.is_empty() {
                return Vec::new();
            }
            let mut captured = Vec::with_capacity(state.entries.len());
            for (id, notification) in &state.entries {
                let job = notification.job();
                // Keep the job identity with the stable producer key so a replacement
                // notification using the same ID cannot be pruned as stale.
                captured.push((id.clone(), job));
            }
            captured
        };
        self.synchronize_captured(
            &captured,
            jobs.snapshot_for_unique_ids(captured.iter().map(|(_, job)| *job)),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entries
            .is_empty()
    }

    pub fn remaining_capacity(&self) -> usize {
        MAX_PROGRESS_NOTIFICATIONS.saturating_sub(
            self.state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .entries
                .len(),
        )
    }

    /// Removes an authoritative job binding after its lifecycle has terminally completed.
    pub fn retire_job(&self, job: JobId) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(notification_id) = state.jobs.remove(&job) {
            state.entries.remove(&notification_id);
        }
    }

    pub fn synchronize(
        &self,
        jobs: impl IntoIterator<Item = EditorJobProgressSnapshot>,
    ) -> Vec<ProgressNotificationSnapshot> {
        let jobs = jobs
            .into_iter()
            .map(|snapshot| (snapshot.id(), snapshot))
            .collect::<BTreeMap<JobId, _>>();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .entries
            .retain(|_, notification| jobs.contains_key(&notification.job()));
        state.jobs.retain(|job, _| jobs.contains_key(job));
        state
            .entries
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
        captured: &[(NotificationId, JobId)],
        jobs: impl IntoIterator<Item = EditorJobProgressSnapshot>,
    ) -> Vec<ProgressNotificationSnapshot> {
        let jobs = jobs
            .into_iter()
            .map(|snapshot| (snapshot.id(), snapshot))
            .collect::<BTreeMap<JobId, _>>();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for (notification_id, captured_job) in captured {
            if jobs.contains_key(captured_job) {
                continue;
            }
            let remove_captured_binding = state
                .entries
                .get(notification_id)
                .is_some_and(|notification| notification.job() == *captured_job);
            if remove_captured_binding {
                state.entries.remove(notification_id);
                state.jobs.remove(captured_job);
            }
        }
        state
            .entries
            .values()
            .filter_map(|notification| {
                jobs.get(&notification.job())
                    .cloned()
                    .map(|job| ProgressNotificationSnapshot::new(notification.clone(), job))
            })
            .collect()
    }

    #[cfg(test)]
    fn job_lookup_probe_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .job_lookup_probes
    }
}

fn is_automatic_binding(notification: &ProgressNotification) -> bool {
    notification.source().kind() == NotificationSourceKind::Builtin
        && notification.source().id() == AUTOMATIC_PROGRESS_SOURCE_ID
}

#[cfg(test)]
#[path = "tests/center.rs"]
mod tests;
