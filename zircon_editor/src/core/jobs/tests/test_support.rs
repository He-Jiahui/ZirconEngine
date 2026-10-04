//! 只为测试集中提供运行时调度器和任务系统夹具；生产上下文必须显式复用自己的工作线程所有者，不应沿测试构造路径建立隐藏线程池。
use zircon_runtime::core::runtime::tasks::{JobScheduler, TaskPools};

use super::{EditorJobLimits, EditorJobSystem};
use crate::core::editor_message::SharedEditorMessageBus;

pub(crate) fn test_job_scheduler() -> JobScheduler {
    // Isolate diagnostics and callback queues while sharing one physical test worker owner.
    JobScheduler::from_pool(TaskPools::process_default().compute().clone())
}

pub(crate) fn test_job_system() -> EditorJobSystem {
    test_job_system_with_limits(EditorJobLimits::default())
}

pub(crate) fn test_job_system_with_limits(limits: EditorJobLimits) -> EditorJobSystem {
    EditorJobSystem::with_scheduler(test_job_scheduler(), limits)
}

pub(crate) fn test_job_system_with_bus(
    bus: SharedEditorMessageBus,
    limits: EditorJobLimits,
) -> EditorJobSystem {
    EditorJobSystem::with_scheduler_and_bus(test_job_scheduler(), bus, limits)
}
