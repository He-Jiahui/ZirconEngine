use std::sync::Arc;

use super::super::super::super::*;

pub(super) struct StartupRuntimeBackend {
    pub(super) runtime: EditorHostEventController,
    pub(super) native_plugin_live_host: Arc<zircon_runtime::plugin::native::NativePluginLiveHost>,
}

pub(super) fn create_startup_runtime_backend(
    runtime: EditorHostEventController,
) -> StartupRuntimeBackend {
    let native_plugin_live_host = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_native_plugin_live_host");
        Arc::new(zircon_runtime::plugin::native::NativePluginLiveHost::default())
    };
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_plugin_bridge_activation");
        runtime.set_plugin_bridge_activation(Arc::new(NativePluginBridgeActivation::new(
            native_plugin_live_host.clone(),
        )));
    }

    StartupRuntimeBackend {
        runtime,
        native_plugin_live_host,
    }
}
