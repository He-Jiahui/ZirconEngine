use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_core::CoreRuntime;
use zircon_manager::MANAGER_MODULE_NAME;
use zircon_math::UVec2;
use zircon_scene::DefaultLevelManager;

use crate::{module, EditorEventRuntime, EditorManager, EditorState, EDITOR_MANAGER_NAME};

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
        let config_path = unique_temp_path(prefix);
        std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", &config_path);

        let core = CoreRuntime::new();
        core.register_module(zircon_manager::module_descriptor())
            .unwrap();
        core.register_module(zircon_asset::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.activate_module(MANAGER_MODULE_NAME).unwrap();
        core.activate_module(zircon_asset::ASSET_MODULE_NAME).unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();

        std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");

        let state = EditorState::with_default_selection(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
        );
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
