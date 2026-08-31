use std::sync::Arc;
use std::time::Duration;

use super::{
    CancellationToken, EditorJobAdmissionKey, JobCategory, JobId, JobPriority, MutexGroup,
};

const DEFAULT_ESTIMATED_PENDING_BYTES: usize = 4 * 1024;

#[derive(Clone, Debug)]
pub struct EditorJobSpec {
    pub(super) label: Arc<str>,
    pub(super) category: JobCategory,
    pub(super) priority: JobPriority,
    pub(super) mutex_group: Option<MutexGroup>,
    pub(super) cancel: CancellationToken,
    pub(super) after: Vec<JobId>,
    pub(super) estimated_pending_bytes: usize,
    pub(super) admission_key: Option<EditorJobAdmissionKey>,
    pub(super) max_pending_age: Option<Duration>,
}

impl EditorJobSpec {
    pub fn new(label: impl Into<String>, category: JobCategory) -> Self {
        Self {
            label: Arc::from(label.into()),
            category,
            priority: JobPriority::Normal,
            mutex_group: None,
            cancel: CancellationToken::default(),
            after: Vec::new(),
            estimated_pending_bytes: DEFAULT_ESTIMATED_PENDING_BYTES,
            admission_key: None,
            max_pending_age: None,
        }
    }

    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_mutex_group(mut self, mutex_group: MutexGroup) -> Self {
        self.mutex_group = Some(mutex_group);
        self
    }

    pub fn with_cancel(mut self, cancel: CancellationToken) -> Self {
        self.cancel = cancel;
        self
    }

    pub fn with_estimated_bytes(mut self, estimated_pending_bytes: usize) -> Self {
        self.estimated_pending_bytes = estimated_pending_bytes.max(1);
        self
    }

    pub fn with_admission_key(mut self, admission_key: EditorJobAdmissionKey) -> Self {
        self.admission_key = Some(admission_key);
        self
    }

    pub fn with_max_pending_age(mut self, max_pending_age: Duration) -> Self {
        self.max_pending_age = Some(max_pending_age);
        self
    }

    pub fn after(mut self, dependency: JobId) -> Self {
        if let Err(index) = self.after.binary_search(&dependency) {
            self.after.insert(index, dependency);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::EditorJobSpec;
    use crate::core::jobs::{JobCategory, JobId};

    const DEPENDENCY_EVIDENCE_COUNT: usize = 4_096;
    const MAX_DEPENDENCY_BUILD_LATENCY: Duration = Duration::from_millis(100);

    #[test]
    fn dependencies_are_sorted_and_deduplicated_independent_of_builder_order() {
        let spec = EditorJobSpec::new("ordered-dependencies", JobCategory::Misc)
            .after(JobId::new(9))
            .after(JobId::new(2))
            .after(JobId::new(5))
            .after(JobId::new(2));

        assert_eq!(
            spec.after,
            vec![JobId::new(2), JobId::new(5), JobId::new(9)]
        );
    }

    #[test]
    fn dependency_dedup_uses_binary_search_instead_of_a_linear_contains_scan() {
        let source = include_str!("spec.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("job spec implementation");

        assert!(implementation.contains("self.after.binary_search(&dependency)"));
        assert!(!implementation.contains("self.after.contains(&dependency)"));
    }

    #[test]
    #[ignore = "managed Editor09 performance evidence"]
    fn editor09_dependency_dedup_binary_search_evidence() {
        let started = Instant::now();
        let mut spec = EditorJobSpec::new("dependency-evidence", JobCategory::Misc);
        for id in 1..=DEPENDENCY_EVIDENCE_COUNT as u64 {
            spec = spec.after(JobId::new(id));
        }
        let elapsed = started.elapsed();

        assert_eq!(spec.after.len(), DEPENDENCY_EVIDENCE_COUNT);
        assert!(elapsed <= MAX_DEPENDENCY_BUILD_LATENCY);
        let comparisons_before = DEPENDENCY_EVIDENCE_COUNT * (DEPENDENCY_EVIDENCE_COUNT - 1) / 2;
        let comparisons_after_upper_bound = (0..DEPENDENCY_EVIDENCE_COUNT)
            .map(|existing| {
                if existing == 0 {
                    0
                } else {
                    existing.ilog2() as usize + 1
                }
            })
            .sum::<usize>();
        let comparison_reduction_percent =
            (1.0 - comparisons_after_upper_bound as f64 / comparisons_before as f64) * 100.0;
        println!(
            "EDITOR_JOB_BENCH_V1 kind=dependency_dedup dependencies={} comparisons_before={} comparisons_after_upper_bound={} comparison_reduction_percent={:.4} elapsed_ns={} target_ns={}",
            DEPENDENCY_EVIDENCE_COUNT,
            comparisons_before,
            comparisons_after_upper_bound,
            comparison_reduction_percent,
            elapsed.as_nanos(),
            MAX_DEPENDENCY_BUILD_LATENCY.as_nanos(),
        );
    }
}
