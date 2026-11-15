use serde::{Deserialize, Serialize};

// TODO: [CR-RUNTIME-TASKS-0001] 明确 DetachOnDrop 与 FinishOnShutdown 的不同保证；当前句柄释放和 scope 关闭只特判 CancelOnDrop，另两项在这两条路径同路。
/// 指定最后一个公开句柄释放及任务作用域关闭时的协作取消策略。
///
/// 策略只决定这两个事件是否自动请求取消；显式 `TaskHandle::request_cancellation` 对三项同样生效。
/// graph shutdown 不再按策略分支：它关闭每个 scope 的准入，然后等待已准入任务到达终态，超时则
/// 保持 closing 并在回执中如实报告在途任务。任何一项都不承诺任务在期限内完成。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskCancellationPolicy {
    /// 最后一个公开句柄释放或 scope 关闭准入时请求协作取消。排队任务到达 worker 后不运行用户闭包并记为
    /// `Cancelled`；运行中的任务通过 token 观察请求。
    #[default]
    CancelOnDrop,
    /// 句柄释放和 scope 关闭准入都不请求取消，任务保持已准入。目前在句柄释放、scope 关闭和
    /// graph shutdown 三条路径上与 `FinishOnShutdown` 没有区别（见上方 TODO）。
    DetachOnDrop,
    /// 句柄释放和 scope 关闭准入都不请求取消，任务保持已准入。目前在句柄释放、scope 关闭和
    /// graph shutdown 三条路径上与 `DetachOnDrop` 没有区别（见上方 TODO）。
    FinishOnShutdown,
}
