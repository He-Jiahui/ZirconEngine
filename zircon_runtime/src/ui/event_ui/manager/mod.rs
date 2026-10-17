//! UI 控制面的路由执行、反射存储及通知发送端的实现集合。
//!
//! 注册和调用入口共享 [`UiEventManager`] 的拥有状态；反射查询读取管理器保存的快照，
//! 宿主业务动作仍由注册回调或宿主自己的分派器执行。订阅者须另行持有并消费接收端。
//! 差异计算与路由记录是内部支持模块，各子模块的实现共同维护同一管理器。

#[path = "diff.rs"]
mod diff;
#[path = "invocation.rs"]
mod invocation;
#[path = "reflection_store.rs"]
mod reflection_store;
#[path = "registration.rs"]
mod registration;
#[path = "route_entry.rs"]
mod route_entry;
#[path = "route_handler.rs"]
mod route_handler;
#[path = "subscription.rs"]
mod subscription;
#[path = "ui_event_manager.rs"]
mod ui_event_manager;

pub use ui_event_manager::UiEventManager;
