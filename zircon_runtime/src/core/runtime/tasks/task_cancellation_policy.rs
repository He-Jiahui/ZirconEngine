use serde::{Deserialize, Serialize};

// TODO: [CR-RUNTIME-TASKS-0001] 明确 DetachOnDrop 与 FinishOnShutdown 的不同保证；当前句柄释放和 scope 关闭只特判 CancelOnDrop，另两项在这两条路径同路。
/// 指定最后一个公开句柄释放及任务作用域关闭时的协作取消策略。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskCancellationPolicy {
    #[default]
    CancelOnDrop,
    DetachOnDrop,
    FinishOnShutdown,
}
