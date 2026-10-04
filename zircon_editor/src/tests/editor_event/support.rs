use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};
use crate::ui::host::editor_asset_manager::{
    editor_asset_manager_handle, EditorAssetCatalogGeneration,
};
use crate::ui::host::module::{self, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorHostEventController;
use crate::ui::host::EditorManager;
use crate::ui::host::{
    EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
};
use crate::ui::workbench::state::EditorState;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

pub(crate) fn env_lock() -> &'static crate::tests::support::TestEnvironmentLock {
    crate::tests::support::env_lock()
}

struct EnvironmentVariableRestoreGuard {
    key: &'static str,
    previous_value: Option<std::ffi::OsString>,
}

impl EnvironmentVariableRestoreGuard {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            previous_value: std::env::var_os(key),
        }
    }
}

impl Drop for EnvironmentVariableRestoreGuard {
    fn drop(&mut self) {
        match self.previous_value.take() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

pub(crate) struct TestProjectAssets {
    project: ProjectManager,
}

impl TestProjectAssets {
    pub(crate) fn source_path(&self, locator: &str) -> PathBuf {
        let locator = AssetUri::parse(locator).expect("test asset locator should be canonical");
        let source_path = self
            .project
            .existing_or_primary_project_source_path_for_uri(&locator)
            .expect("test asset locator should resolve through ProjectManager");
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent).expect("test asset source directory should be created");
        }
        source_path
    }
}

#[derive(Default)]
struct TestProjectCleanup {
    roots: Vec<PathBuf>,
}

impl Drop for TestProjectCleanup {
    fn drop(&mut self) {
        for root in self.roots.drain(..) {
            let _ = fs::remove_dir_all(root);
        }
    }
}

pub(crate) struct EventRuntimeHarness {
    #[allow(dead_code)]
    pub core: CoreRuntime,
    pub runtime: EditorHostEventController,
    config_path: PathBuf,
    project_cleanup: TestProjectCleanup,
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
        let restore_config_path = EnvironmentVariableRestoreGuard::capture("ZIRCON_CONFIG_PATH");
        std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);

        let core = CoreRuntime::new();
        for engine_module in zircon_runtime::builtin::runtime_core_modules() {
            core.register_module(engine_module.descriptor()).unwrap();
        }
        core.register_module(zircon_runtime::ui::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.store_config_value(
            EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
            serde_json::json!(enabled_subsystems),
        );
        core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
            .unwrap();
        core.activate_module(SCENE_MODULE_NAME).unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();
        crate::tests::support::configure_editor_test_runtime_build_set(&core);

        drop(restore_config_path);

        let manager = core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        // Mirror production startup: the shell and controller share one stable edit gateway.
        let mut state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            manager.context().clone(),
        );
        state.mark_project_open();
        let runtime = EditorHostEventController::new(state, manager);

        Self {
            core,
            runtime,
            config_path,
            project_cleanup: TestProjectCleanup::default(),
        }
    }

    /// Creates one real ProjectAuthority project, lets the caller populate canonical asset
    /// locators, then opens and indexes it through the same managers used by the editor host.
    pub(crate) fn open_project_with_assets<F>(
        &mut self,
        prefix: &str,
        populate: F,
    ) -> Arc<EditorAssetCatalogGeneration>
    where
        F: FnOnce(&TestProjectAssets),
    {
        let location = unique_temp_dir(prefix);
        let created = ProjectAuthority::default()
            .create_project(
                &NewProjectDraft {
                    project_name: "EventFixture".to_string(),
                    location: location.to_string_lossy().into_owned(),
                    template: ProjectTemplateId::RenderableEmpty,
                },
                &crate::tests::support::test_project_creation_provenance(),
            )
            .expect("ProjectAuthority should create the event-runtime fixture project");
        self.project_cleanup.roots.push(location);

        let assets = TestProjectAssets {
            project: ProjectManager::open(&created.root)
                .expect("event-runtime fixture ProjectManager should open"),
        };
        populate(&assets);

        let manager = self
            .core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .expect("event-runtime fixture should resolve EditorManager");
        manager
            .open_project(&created.root)
            .expect("event-runtime fixture project should open through EditorManager");

        let core = self.core.handle();
        let catalog = zircon_runtime::core::manager::resolve_manager_service(
            &core,
            editor_asset_manager_handle(&core)
                .expect("event-runtime fixture should resolve EditorAssetManager handle"),
        )
        .expect("event-runtime fixture should resolve EditorAssetManager")
        .catalog_snapshot();
        self.runtime.sync_asset_catalog(Arc::clone(&catalog));
        catalog
    }
}

impl Drop for EventRuntimeHarness {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.config_path);
    }
}

#[cfg(test)]
mod environment_variable_restore_tests {
    use super::{env_lock, EnvironmentVariableRestoreGuard, EventRuntimeHarness};

    const TEST_ENVIRONMENT_KEY: &str = "ZIRCON_EDITOR_EVENT_ENVIRONMENT_RESTORE_TEST";

    #[test]
    fn environment_guard_restores_previous_value_or_absence() {
        let _lock = env_lock().lock().unwrap();
        let _restore_process_value = EnvironmentVariableRestoreGuard::capture(TEST_ENVIRONMENT_KEY);

        std::env::set_var(TEST_ENVIRONMENT_KEY, "preexisting");
        {
            let _restore = EnvironmentVariableRestoreGuard::capture(TEST_ENVIRONMENT_KEY);
            std::env::set_var(TEST_ENVIRONMENT_KEY, "temporary");
        }
        assert_eq!(
            std::env::var_os(TEST_ENVIRONMENT_KEY),
            Some(std::ffi::OsString::from("preexisting"))
        );

        std::env::remove_var(TEST_ENVIRONMENT_KEY);
        {
            let _restore = EnvironmentVariableRestoreGuard::capture(TEST_ENVIRONMENT_KEY);
            std::env::set_var(TEST_ENVIRONMENT_KEY, "temporary");
        }
        assert!(std::env::var_os(TEST_ENVIRONMENT_KEY).is_none());
    }

    #[test]
    fn environment_guard_restores_during_unwind() {
        let _lock = env_lock().lock().unwrap();
        let _restore_process_value = EnvironmentVariableRestoreGuard::capture(TEST_ENVIRONMENT_KEY);
        std::env::set_var(TEST_ENVIRONMENT_KEY, "preexisting");

        let unwind = std::panic::catch_unwind(|| {
            let _restore = EnvironmentVariableRestoreGuard::capture(TEST_ENVIRONMENT_KEY);
            std::env::set_var(TEST_ENVIRONMENT_KEY, "temporary");
            panic!("exercise environment restoration during unwinding");
        });

        assert!(unwind.is_err());
        assert_eq!(
            std::env::var_os(TEST_ENVIRONMENT_KEY),
            Some(std::ffi::OsString::from("preexisting"))
        );
    }

    #[test]
    fn event_runtime_harness_restores_the_callers_config_path() {
        let _lock = env_lock().lock().unwrap();
        let _restore_process_value = EnvironmentVariableRestoreGuard::capture("ZIRCON_CONFIG_PATH");
        let previous_path = super::unique_temp_path("editor-event-config-previous");
        std::env::set_var("ZIRCON_CONFIG_PATH", &previous_path);

        let harness = EventRuntimeHarness::new("editor-event-config-restore");

        assert_eq!(
            std::env::var_os("ZIRCON_CONFIG_PATH"),
            Some(previous_path.as_os_str().to_owned())
        );
        drop(harness);
    }
}
