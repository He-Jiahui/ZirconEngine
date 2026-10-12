//! 窗口事件生命周期职责组织。
//! 分派器只负责路由，策略由各 leaf 执行。

#[path = "close.rs"]
mod close;
#[path = "focus.rs"]
mod focus;
#[path = "scale_factor.rs"]
mod scale_factor;
#[path = "status.rs"]
mod status;
