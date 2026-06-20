mod dispatch;
mod native_backend;
mod types;

pub(super) use dispatch::{
    dispatch_live_plugin_backend_action, live_plugin_backend_success_message,
};
pub(in crate::ui::retained_host::app) use types::{
    ModulePluginLiveHostBackend, ModulePluginLiveHostCommand,
};
#[cfg(test)]
pub(in crate::ui::retained_host::app) use types::{
    ModulePluginLiveHostOutcome, ModulePluginLiveHostRequest,
};

#[cfg(test)]
mod tests;
