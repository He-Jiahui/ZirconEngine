use std::collections::BTreeMap;
use std::time::Duration;

use super::JobCategory;

const DEFAULT_THUMBNAIL_LIMIT: usize = 2;
const DEFAULT_EXPORT_LIMIT: usize = 1;
const DEFAULT_INTERACTIVE_SAVE_LIMIT: usize = 1;
const DEFAULT_PLAY_LIMIT: usize = 1;
const DEFAULT_PENDING_ADMISSION_ENTRIES: usize = 16_384;
const DEFAULT_PENDING_ADMISSION_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_PENDING_ADMISSION_AGE: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorJobAdmissionLimits {
    pub(super) max_pending_entries: usize,
    pub(super) max_pending_estimated_bytes: usize,
    pub(super) max_oldest_pending_age: Duration,
}

impl EditorJobAdmissionLimits {
    pub const fn new(
        max_pending_entries: usize,
        max_pending_estimated_bytes: usize,
        max_oldest_pending_age: Duration,
    ) -> Self {
        Self {
            max_pending_entries,
            max_pending_estimated_bytes,
            max_oldest_pending_age,
        }
    }
}

impl Default for EditorJobAdmissionLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_PENDING_ADMISSION_ENTRIES,
            DEFAULT_PENDING_ADMISSION_BYTES,
            DEFAULT_PENDING_ADMISSION_AGE,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorJobAdmissionSnapshot {
    pending_entries: usize,
    pending_estimated_bytes: usize,
    oldest_pending_age: Option<Duration>,
    merged_submissions: u64,
    cancelled_pending: u64,
    started_pending: u64,
}

impl EditorJobAdmissionSnapshot {
    pub(super) const fn new(
        pending_entries: usize,
        pending_estimated_bytes: usize,
        oldest_pending_age: Option<Duration>,
        merged_submissions: u64,
        cancelled_pending: u64,
        started_pending: u64,
    ) -> Self {
        Self {
            pending_entries,
            pending_estimated_bytes,
            oldest_pending_age,
            merged_submissions,
            cancelled_pending,
            started_pending,
        }
    }

    pub const fn pending_entries(self) -> usize {
        self.pending_entries
    }

    pub const fn pending_estimated_bytes(self) -> usize {
        self.pending_estimated_bytes
    }

    pub const fn oldest_pending_age(self) -> Option<Duration> {
        self.oldest_pending_age
    }

    pub const fn merged_submissions(self) -> u64 {
        self.merged_submissions
    }

    pub const fn cancelled_pending(self) -> u64 {
        self.cancelled_pending
    }

    pub const fn started_pending(self) -> u64 {
        self.started_pending
    }
}

#[derive(Clone, Debug)]
pub struct EditorJobLimits {
    limits: BTreeMap<JobCategory, usize>,
    admission: EditorJobAdmissionLimits,
}

impl EditorJobLimits {
    pub(crate) fn resolved(
        worker_parallelism: usize,
        configured_limits: impl IntoIterator<Item = (JobCategory, usize)>,
    ) -> Self {
        let worker_limit = worker_parallelism.max(1);
        let mut limits = BTreeMap::new();
        for category in JobCategory::ALL {
            limits.insert(
                category,
                user_configurable_default_limit(category).unwrap_or(worker_limit),
            );
        }
        for (category, limit) in configured_limits {
            limits.insert(category, limit.max(1));
        }
        Self {
            limits,
            admission: EditorJobAdmissionLimits::default(),
        }
    }

    pub fn with_limit(mut self, category: JobCategory, limit: usize) -> Self {
        self.limits.insert(category, limit.max(1));
        self
    }

    pub fn with_admission_limits(mut self, admission: EditorJobAdmissionLimits) -> Self {
        self.admission = admission;
        self
    }

    pub fn limit(&self, category: JobCategory) -> usize {
        self.limits
            .get(&category)
            .copied()
            .expect("resolved editor job limits contain every category")
    }

    pub(super) const fn admission_limits(&self) -> EditorJobAdmissionLimits {
        self.admission
    }
}

impl Default for EditorJobLimits {
    fn default() -> Self {
        Self::resolved(1, [])
    }
}

pub(crate) const fn user_configurable_default_limit(category: JobCategory) -> Option<usize> {
    match category {
        JobCategory::Thumbnail => Some(DEFAULT_THUMBNAIL_LIMIT),
        JobCategory::Export => Some(DEFAULT_EXPORT_LIMIT),
        JobCategory::InteractiveSave => Some(DEFAULT_INTERACTIVE_SAVE_LIMIT),
        JobCategory::Play => Some(DEFAULT_PLAY_LIMIT),
        JobCategory::Import | JobCategory::Compile | JobCategory::Index | JobCategory::Misc => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_defaults_bound_every_job_category() {
        let limits = EditorJobLimits::resolved(4, []);

        for category in JobCategory::ALL {
            assert!(
                limits.limit(category) < usize::MAX,
                "{category:?} must not bypass admission with an unbounded default"
            );
        }
    }

    #[test]
    fn interactive_save_has_an_explicit_finite_default() {
        let limits = EditorJobLimits::resolved(4, []);

        assert_eq!(
            limits.limit(JobCategory::InteractiveSave),
            DEFAULT_INTERACTIVE_SAVE_LIMIT
        );
    }

    #[test]
    fn play_default_does_not_alias_the_export_default_path() {
        assert_eq!(
            user_configurable_default_limit(JobCategory::Play),
            Some(DEFAULT_PLAY_LIMIT)
        );
        assert_eq!(
            user_configurable_default_limit(JobCategory::Export),
            Some(DEFAULT_EXPORT_LIMIT)
        );
    }
}
