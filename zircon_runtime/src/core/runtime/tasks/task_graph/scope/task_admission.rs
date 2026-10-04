use crate::core::runtime::tasks::{JobHandle, TaskPoolSubmission};

// 一次 TaskAdmission 同时持有图级 completion 与物理 worker submission；只有两者都取得后 scope 才能增加 queued 计数。
pub(super) struct TaskAdmission {
    pub(super) completion: JobHandle,
    pub(super) submission: TaskPoolSubmission,
}
