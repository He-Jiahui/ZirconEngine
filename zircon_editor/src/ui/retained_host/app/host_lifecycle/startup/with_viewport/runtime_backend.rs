use std::sync::Arc;

use super::super::super::super::*;

pub(super) struct StartupRuntimeBackend {
    pub(super) runtime: EditorEventRuntime,
    pub(super) native_plugin_live_host: Arc<zircon_runtime::plugin::native::NativePluginLiveHost>,
}

pub(super) fn create_startup_runtime_backend(
    state: EditorState,
    editor_manager: Arc<EditorManager>,
) -> StartupRuntimeBackend {
    let native_plugin_live_host = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_native_plugin_live_host");
        Arc::new(zircon_runtime::plugin::native::NativePluginLiveHost::default())
    };
    let runtime = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_editor_event_runtime");
        EditorEventRuntime::new(state, editor_manager)
    };
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_set_play_mode_backend");
        runtime.set_runtime_play_mode_backend(Arc::new(
            NativePluginEditorRuntimePlayModeBackend::new(native_plugin_live_host.clone()),
        ));
    }

    StartupRuntimeBackend {
        runtime,
        native_plugin_live_host,
    }
}
