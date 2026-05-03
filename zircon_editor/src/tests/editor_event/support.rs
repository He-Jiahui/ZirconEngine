use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::host::module::{self, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::host::{
    EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
};
use crate::ui::workbench::state::EditorState;

pub(crate) fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

pub(crate) struct EventRuntimeHarness {
    #[allow(dead_code)]
    pub core: CoreRuntime,
    pub runtime: EditorEventRuntime,
    config_path: PathBuf,
}

impl EventRuntimeHarness {
    pub(crate) fn new(prefix: &str) -> Self {
        Self::with_enabled_subsystems(
            prefix,
            &[
                EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
                EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
                EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
                EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
            ],
        )
    }

    pub(crate) fn with_enabled_subsystems(prefix: &str, enabled_subsystems: &[&str]) -> Self {
        let config_path = unique_temp_path(prefix);
        std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);

        let core = CoreRuntime::new();
        core.register_module(foundation_module_descriptor())
            .unwrap();
        core.register_module(zircon_runtime::asset::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.store_config_value(
            EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
            serde_json::json!(enabled_subsystems),
        );
        core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
            .unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();

        std::env::remove_var("ZIRCON_CONFIG_PATH");

        let mut state = EditorState::with_default_selection(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
        );
        state.mark_project_open();
        let manager = core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        let runtime = EditorEventRuntime::new(state, manager);

        Self {
            core,
            runtime,
            config_path,
        }
    }
}

impl Drop for EventRuntimeHarness {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.config_path);
    }
}
