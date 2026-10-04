//! 应用生命周期模块只发布进程级事实；窗口焦点、可见性和原生对象仍由各自宿主管理。
mod service;
mod service_error;
mod state;

pub(crate) use service::ApplicationLifecycleService;
pub use service_error::ApplicationLifecycleServiceError;
