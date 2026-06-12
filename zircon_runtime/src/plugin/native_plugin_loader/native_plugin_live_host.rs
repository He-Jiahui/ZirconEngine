use std::collections::BTreeMap;
use std::sync::Mutex;

use super::{LoadedNativePlugin, NativePluginLoader};

mod diagnostics;
mod hot_reload;
mod keys;
mod lifecycle;
mod loading;
mod reports;
mod runtime_behavior;

#[cfg(test)]
use super::{
    NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
#[cfg(test)]
use crate::plugin::PluginModuleKind;
#[cfg(test)]
use diagnostics::diagnostics_from_behavior_report;
#[cfg(test)]
use hot_reload::{restore_runtime_snapshot, NativePluginHotReloadState};
#[cfg(test)]
use keys::live_key;
#[cfg(test)]
use loading::lock_loaded_native_plugins;
pub use reports::{
    NativePluginLiveHostCommand, NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimePlayModeExitReport,
    NativePluginRuntimePlayModeSnapshot, NativePluginRuntimePluginState,
    NativePluginRuntimeStateRestoreReport, NativePluginRuntimeStateSnapshot,
    NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
#[cfg(test)]
use runtime_behavior::{allow_missing_unload_callback_to_drop_handle, unload_behavior};

#[derive(Debug, Default)]
pub struct NativePluginLiveHost {
    loader: NativePluginLoader,
    loaded: Mutex<BTreeMap<String, LoadedNativePlugin>>,
}

#[cfg(test)]
#[path = "native_plugin_live_host/tests.rs"]
mod tests;
