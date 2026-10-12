#[path = "live_host/development_watch.rs"]
mod development_watch;
#[path = "live_host/dispatch.rs"]
mod dispatch;
#[path = "live_host/native_backend.rs"]
mod native_backend;
#[path = "live_host/types.rs"]
mod types;

pub(super) use dispatch::{
    dispatch_live_plugin_backend_action, live_plugin_backend_success_message,
};
pub(in crate::ui::retained_host::app) use native_backend::NativePluginDevelopmentLiveHostBackend;
pub(in crate::ui::retained_host::app) use types::{
    ModulePluginLiveHostBackend, ModulePluginLiveHostCommand, ModulePluginLiveHostCompletion,
    ModulePluginLiveHostProject,
};
#[cfg(test)]
pub(in crate::ui::retained_host::app) use types::{
    ModulePluginLiveHostOutcome, ModulePluginLiveHostRequest,
};

#[cfg(test)]
#[path = "live_host/tests/cases.rs"]
mod tests;
