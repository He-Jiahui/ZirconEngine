//! 管理可反射 UI 的路由、快照及通知；宿主业务动作仍由注册处理器或宿主自己的 typed dispatcher 执行。

mod manager;

pub use manager::UiEventManager;
