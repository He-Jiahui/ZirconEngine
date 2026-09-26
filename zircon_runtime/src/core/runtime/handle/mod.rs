//! 运行时句柄集中承载模块生命周期、服务解析与状态等内核入口。
//! `CoreRuntime` 对外委托到此处；句柄副本共享同一运行时状态。

mod activation;
mod core_handle;
mod diagnostics;
mod events;
mod random;
mod registration;
mod resolution;
mod runtime_extensions;
mod service_identity;
mod states;
mod time;

pub use core_handle::CoreHandle;
pub use resolution::{ServiceCallGuard, ServiceHandle};
pub(crate) use service_identity::RegisteredServiceIdentity;
