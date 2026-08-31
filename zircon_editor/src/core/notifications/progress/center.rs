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
        let (captured, ids) = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.entries.is_empty() {
                return Vec::new();
            }
            (
                // Keep the job identity with the stable producer key so a replacement
                // notification using the same ID cannot be pruned as stale.
                state
                    .entries
                    .iter()
                    .map(|(id, notification)| (id.clone(), notification.job()))
                    .collect::<BTreeMap<_, _>>(),
                state
                    .entries
                    .values()
                    .map(ProgressNotification::job)
                    .collect::<Vec<_>>(),
            )
        };
        self.synchronize_captured(&captured, jobs.snapshot_for_ids(ids))
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
        captured: &BTreeMap<NotificationId, JobId>,
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
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::jobs::{EditorJobProgressSnapshot, JobCategory, JobId};
    use crate::core::notifications::{NotificationId, NotificationSource};

    use super::{
        AUTOMATIC_PROGRESS_SOURCE_ID, MAX_PROGRESS_NOTIFICATIONS, ProgressNotification,
        ProgressNotificationCenter,
    };

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

        assert!(
            center
                .synchronize_captured(&captured, std::iter::empty::<EditorJobProgressSnapshot>(),)
                .is_empty()
        );
        assert_eq!(center.synchronize([job(replacement_job)]).len(), 1);
    }

    #[test]
    fn retiring_a_manual_replacement_releases_its_job_index_entry() {
        let center = ProgressNotificationCenter::default();
        let job_id = JobId::new(7);
        let automatic = ProgressNotification::new(
            NotificationId::parse("editor.progress.automatic").unwrap(),
            NotificationSource::builtin(AUTOMATIC_PROGRESS_SOURCE_ID).unwrap(),
            job_id,
            "editor.progress.title",
        )
        .unwrap();
        center.publish(automatic).unwrap();
        center
            .publish(notification("editor.progress.manual", job_id))
            .unwrap();

        center.retire_job(job_id);

        assert!(
            center
                .publish(notification("editor.progress.reused", job_id))
                .is_ok()
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn optimization_wave_20260825_editor10_progress_job_index_evidence() {
        const LOOKUPS: usize = 100_000;
        const MAX_ELAPSED_NS: u128 = 3_000_000_000;

        let center = ProgressNotificationCenter::default();
        for index in 0..MAX_PROGRESS_NOTIFICATIONS {
            center
                .publish(notification(
                    &format!("editor.progress.bench.{index:02}"),
                    JobId::new(index as u64),
                ))
                .unwrap();
        }
        let candidate = notification(
            "editor.progress.bench.duplicate",
            JobId::new((MAX_PROGRESS_NOTIFICATIONS - 1) as u64),
        );
        let probes_before = center.job_lookup_probe_count();
        let started = Instant::now();
        for _ in 0..LOOKUPS {
            black_box(center.publish(candidate.clone()).unwrap_err());
        }
        let elapsed_ns = started.elapsed().as_nanos();
        let indexed_job_probes = center
            .job_lookup_probe_count()
            .saturating_sub(probes_before);
        let legacy_candidate_checks = LOOKUPS * MAX_PROGRESS_NOTIFICATIONS;
        let probe_reduction_bps = legacy_candidate_checks
            .saturating_sub(indexed_job_probes)
            .saturating_mul(10_000)
            / legacy_candidate_checks;

        println!(
            "EDITOR_PROGRESS_JOB_INDEX_BENCH_V1 entries={MAX_PROGRESS_NOTIFICATIONS} lookups={LOOKUPS} legacy_candidate_checks={legacy_candidate_checks} indexed_job_probes={indexed_job_probes} probe_reduction_bps={probe_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
        );

        assert_eq!(indexed_job_probes, LOOKUPS);
        assert_eq!(probe_reduction_bps, 9_843);
        assert!(elapsed_ns <= MAX_ELAPSED_NS);
    }
}
