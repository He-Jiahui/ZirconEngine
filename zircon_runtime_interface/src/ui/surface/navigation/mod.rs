//! 导航命令与路由快照的公共导出边界。
//!
//! Surface 选择目标并构造路由，派发器读取路由；这里的值类型不持有树的借用或执行回调。

#[path = "event_kind.rs"]
mod event_kind;
#[path = "route.rs"]
mod route;

pub use event_kind::UiNavigationEventKind;
pub use route::UiNavigationRoute;
