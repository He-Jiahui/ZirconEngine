#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskPoolThreadAssignmentPolicy {
    pub min_threads: usize,
    pub max_threads: usize,
    pub percent: f32,
}

impl TaskPoolThreadAssignmentPolicy {
    pub fn thread_count(self, remaining_threads: usize, total_threads: usize) -> usize {
        assert!(
            self.percent.is_finite() && self.percent >= 0.0,
            "task pool thread percent must be finite and non-negative"
        );
        let min_threads = self.min_threads.max(1);
        let max_threads = self.max_threads.max(min_threads);
        let proportion = total_threads as f32 * self.percent;
        let desired = proportion.round() as usize;
        desired
            .min(remaining_threads)
            .clamp(min_threads, max_threads)
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
    pub fn with_num_threads(thread_count: usize) -> Self {
        let thread_count = thread_count.max(1);
        Self {
            min_total_threads: thread_count,
            max_total_threads: thread_count,
            ..Self::default()
        }
    }
}
