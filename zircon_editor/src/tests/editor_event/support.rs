use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::ResourceKind;

use crate::core::asset::{AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use crate::ui::host::editor_asset_manager::{
    editor_asset_manager_handle, EditorAssetCatalogSnapshotRecord,
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

pub(crate) fn env_lock() -> &'static crate::tests::support::TestEnvironmentLock {
    crate::tests::support::env_lock()
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
    ) -> EditorAssetCatalogSnapshotRecord
    where
        F: FnOnce(&TestProjectAssets),
    {
        let location = unique_temp_dir(prefix);
        let created = ProjectAuthority::default()
            .create_project(&NewProjectDraft {
                project_name: "EventFixture".to_string(),
                location: location.to_string_lossy().into_owned(),
                template: NewProjectTemplate::RenderableEmpty,
            })
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
        self.runtime.sync_asset_catalog(catalog.clone());
        catalog
    }

    /// Installs the same typed toolkit contributions exposed by the Timeline Sequence and
    /// Animation Graph editor plugins, using the normal capability-gated registry path.
    pub(crate) fn register_animation_asset_toolkits(&self) {
        const TIMELINE_CAPABILITY: &str = "editor.extension.timeline_sequence_authoring";
        const GRAPH_CAPABILITY: &str = "editor.extension.animation_graph_authoring";

        let manager = self
            .core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .expect("animation registry fixture should resolve EditorManager");
        manager
            .set_editor_capabilities_enabled(
                &[
                    TIMELINE_CAPABILITY.to_string(),
                    GRAPH_CAPABILITY.to_string(),
                ],
                true,
            )
            .expect("animation registry fixture capabilities should enable");

        let timeline_open = EditorOperationPath::parse("timeline_sequence.authoring.open").unwrap();
        let mut timeline = EditorExtensionRegistry::default();
        timeline
            .register_command(EditorCommandDescriptor::operation(
                timeline_open.clone(),
                "Open Timeline Sequence",
            ))
            .unwrap();
        timeline
            .register_asset_type_contribution(
                AssetTypeContribution::augment(AssetTypeId::from_resource_kind(
                    ResourceKind::AnimationSequence,
                ))
                .with_toolkit(
                    AssetToolkitDescriptor::new("editor.animation_sequence", timeline_open)
                        .with_required_capabilities([TIMELINE_CAPABILITY]),
                ),
            )
            .unwrap();
        self.runtime
            .register_editor_extension_with_required_capabilities(
                timeline,
                vec![TIMELINE_CAPABILITY.to_string()],
            )
            .expect("timeline sequence registry fixture should register");

        let graph_open =
            EditorOperationPath::parse("animation_graph.authoring.open_graph").unwrap();
        let state_machine_open =
            EditorOperationPath::parse("animation_graph.authoring.open_state_machine").unwrap();
        let mut graph = EditorExtensionRegistry::default();
        for (operation, label) in [
            (graph_open.clone(), "Open Animation Graph"),
            (state_machine_open.clone(), "Open Animation State Machine"),
        ] {
            graph
                .register_command(EditorCommandDescriptor::operation(operation, label))
                .unwrap();
        }
        for (kind, operation) in [
            (ResourceKind::AnimationGraph, graph_open),
            (ResourceKind::AnimationStateMachine, state_machine_open),
        ] {
            graph
                .register_asset_type_contribution(
                    AssetTypeContribution::augment(AssetTypeId::from_resource_kind(kind))
                        .with_toolkit(
                            AssetToolkitDescriptor::new("editor.animation_graph", operation)
                                .with_required_capabilities([GRAPH_CAPABILITY]),
                        ),
                )
                .unwrap();
        }
        self.runtime
            .register_editor_extension_with_required_capabilities(
                graph,
                vec![GRAPH_CAPABILITY.to_string()],
            )
            .expect("animation graph registry fixture should register");
    }
}

impl Drop for EventRuntimeHarness {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.config_path);
    }
}
