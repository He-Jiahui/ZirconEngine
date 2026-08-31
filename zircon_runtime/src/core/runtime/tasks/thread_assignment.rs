#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskPoolThreadAssignmentPolicy {
    pub min_threads: usize,
    pub max_threads: usize,
    pub percent: f32,
}

impl TaskPoolThreadAssignmentPolicy {
    pub fn thread_count(self, remaining_threads: usize, total_threads: usize) -> usize {
        let desired_threads = self.desired_threads(total_threads);
        if remaining_threads == 0 {
            return 0;
        }
        let min_threads = self.minimum_threads().min(remaining_threads);
        let max_threads = self.maximum_threads().min(remaining_threads);
        desired_threads
            .min(remaining_threads)
            .clamp(min_threads, max_threads)
    }

    pub(super) fn minimum_threads(self) -> usize {
        self.min_threads.max(1)
    }

    pub(super) fn maximum_threads(self) -> usize {
        self.max_threads.max(self.minimum_threads())
    }

    pub(super) fn desired_threads(self, total_threads: usize) -> usize {
        assert!(
            self.percent.is_finite() && self.percent >= 0.0,
            "task pool thread percent must be finite and non-negative"
        );
        let proportion = total_threads as f32 * self.percent;
        proportion.round() as usize
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskPoolOptions {
    pub min_total_threads: usize,
    pub max_total_threads: usize,
    pub io: TaskPoolThreadAssignmentPolicy,
    pub async_compute: TaskPoolThreadAssignmentPolicy,
    pub compute: TaskPoolThreadAssignmentPolicy,
}

impl Default for TaskPoolOptions {
    fn default() -> Self {
        Self {
            min_total_threads: 1,
            max_total_threads: usize::MAX,
            io: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 4,
                percent: 0.25,
            },
            async_compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 4,
                percent: 0.25,
            },
            compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: usize::MAX,
                percent: 1.0,
            },
        }
    }
}

impl TaskPoolOptions {
    /// Requests a physical worker budget shared by the three pool assignment policies.
    ///
    /// The resolved total is raised when their combined minimums require more workers.
    pub fn with_num_threads(thread_count: usize) -> Self {
        let thread_count = thread_count.max(1);
        Self {
            min_total_threads: thread_count,
            max_total_threads: thread_count,
            ..Self::default()
        }
    }
}
