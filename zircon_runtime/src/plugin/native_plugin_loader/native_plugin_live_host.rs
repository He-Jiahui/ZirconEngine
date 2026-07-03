use std::collections::BTreeMap;
use std::sync::Mutex;

use super::{LoadedNativePlugin, NativeBridgeMethodBinding, NativePluginLoader};

mod bridge_lifecycle;
mod bridge_methods;
mod diagnostics;
mod hot_reload;
mod hot_update_application;
mod keys;
mod lifecycle;
mod loading;
mod registration_replay;
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
use bridge_lifecycle::NativePluginBridgeLifecycleError;
#[cfg(test)]
use bridge_methods::NativePluginBridgeMethodError;
#[cfg(test)]
use diagnostics::{diagnostics_from_behavior_report, NativePluginBehaviorDiagnosticError};
#[cfg(test)]
use hot_reload::{
    restore_runtime_snapshot, NativePluginHotReloadError, NativePluginHotReloadState,
};
#[cfg(test)]
use keys::live_key;
#[cfg(test)]
use lifecycle::{load_for_module_kind, NativePluginLiveHostLifecycleError};
#[cfg(test)]
use loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
#[cfg(test)]
use registration_replay::NativePluginRegistrationReplayError;
pub use reports::{
    NativePluginLiveHostBridgeLifecycleReport, NativePluginLiveHostBridgeReloadReport,
    NativePluginLiveHostCommand, NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimeDeltaHotUpdateReport,
    NativePluginRuntimeDeltaHotUpdateRequest, NativePluginRuntimeHotUpdateReport,
    NativePluginRuntimePlayModeExitReport, NativePluginRuntimePlayModeSnapshot,
    NativePluginRuntimePluginState, NativePluginRuntimeRegistrationReplayReport,
    NativePluginRuntimeRegistrationSystemReplay, NativePluginRuntimeStateRestoreReport,
    NativePluginRuntimeStateSnapshot, NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND,
    NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
#[cfg(test)]
use runtime_behavior::NativePluginRuntimeBehaviorError;
#[cfg(test)]
use runtime_behavior::{allow_missing_unload_callback_to_drop_handle, unload_behavior};

#[derive(Debug, Default)]
pub struct NativePluginLiveHost {
    loader: NativePluginLoader,
    loaded: Mutex<BTreeMap<String, LoadedNativePlugin>>,
    runtime_bridge_method_bindings: Mutex<BTreeMap<String, Vec<NativeBridgeMethodBinding>>>,
}

#[cfg(test)]
#[path = "native_plugin_live_host/tests.rs"]
mod tests;
