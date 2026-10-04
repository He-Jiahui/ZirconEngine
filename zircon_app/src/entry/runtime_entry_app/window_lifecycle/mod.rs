//! 窗口事件生命周期职责组织。
//! 分派器只负责路由，策略由各 leaf 执行。

mod close;
mod focus;
mod scale_factor;
mod status;
