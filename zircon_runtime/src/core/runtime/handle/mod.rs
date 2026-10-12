//! 运行时句柄集中承载模块生命周期、服务解析与状态等内核入口。
//! `CoreRuntime` 对外委托到此处；句柄副本共享同一运行时状态。

#[path = "activation.rs"]
mod activation;
#[path = "core_handle.rs"]
mod core_handle;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "events.rs"]
mod events;
#[path = "random.rs"]
mod random;
#[path = "registration/mod.rs"]
mod registration;
#[path = "resolution.rs"]
mod resolution;
#[path = "runtime_extensions.rs"]
mod runtime_extensions;
#[path = "service_identity.rs"]
mod service_identity;
#[path = "states.rs"]
mod states;
#[path = "time.rs"]
mod time;

pub use core_handle::CoreHandle;
pub(crate) use resolution::ServiceAliasSource;
pub use resolution::{ServiceCallGuard, ServiceHandle};
pub(crate) use service_identity::RegisteredServiceIdentity;
