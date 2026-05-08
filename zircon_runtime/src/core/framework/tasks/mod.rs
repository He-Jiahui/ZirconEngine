//! Neutral task pool and async task contracts for runtime framework consumers.

mod async_task_descriptor;
mod async_task_handle;
mod async_task_state;
mod async_task_status;
mod task_cancellation_policy;
mod task_poll_budget;
mod task_pool_descriptor;
mod task_pool_kind;

pub use async_task_descriptor::AsyncTaskDescriptor;
pub use async_task_handle::AsyncTaskHandle;
pub use async_task_state::AsyncTaskState;
pub use async_task_status::AsyncTaskStatus;
pub use task_cancellation_policy::TaskCancellationPolicy;
pub use task_poll_budget::{TaskPollBudget, DEFAULT_MAIN_THREAD_POLLS_PER_FRAME};
pub use task_pool_descriptor::TaskPoolDescriptor;
pub use task_pool_kind::TaskPoolKind;
