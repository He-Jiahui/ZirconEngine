//! Linked plugin roots and the independent project-selection/capability gates.

use std::path::PathBuf;
use std::sync::Arc;

use super::super::*;
use crate::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
};
use crate::core::extension::InspectorCustomizationDescriptor;
use crate::core::plugin::{
    EditorPlugin, EditorPluginDescriptor, EditorPluginLoadingPhase, EditorPluginManager,
    EditorPluginRegistrationReport, EditorPluginState,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::template_runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::plugin::PluginPackageManifest;

const CAPABILITY: &str = "editor.extension.fixture_authoring";
const TEMPLATE_ID: &str = "plugin.fixture.editor.x";
const TEMPLATE_URI: &str = "plugins://fixture/editor/x.zui";
const COMPONENT_TYPE: &str = "fixture.Component.Example";

struct FixturePlugin {
    descriptor: EditorPluginDescriptor,
}

impl FixturePlugin {
    fn new() -> Self {
        Self {
            descriptor: EditorPluginDescriptor::new("fixture", "Fixture", "fixture_editor")
                .with_capability(CAPABILITY),
        }
    }

    fn registration(&self) -> EditorPluginRegistrationReport {
        let report = EditorPluginRegistrationReport::from_plugin(self, fixture_manifest());
        assert!(report.is_success(), "{:?}", report.diagnostics);
        report
    }
}

impl EditorPlugin for FixturePlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        registry
            .register_ui_template(EditorUiTemplateDescriptor::new(TEMPLATE_ID, TEMPLATE_URI))?;
        registry.register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                COMPONENT_TYPE,
                TEMPLATE_URI,
                "fixture.editor.ExampleInspectorController",
            )
            .with_id("plugin.fixture.example")
            .with_template_id(TEMPLATE_ID),
        )
    }
}

fn fixture_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new("fixture", "Fixture")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/runtime/extensions_registration/fixtures")
}

fn register_and_enable(runtime: &EventRuntimeHarness, report: EditorPluginRegistrationReport) {
    runtime
        .runtime
        .register_editor_plugin_registration(report)
        .expect("publish a Plugin-sourced fixture registration");
    assert!(!runtime
        .runtime
        .enabled_plugin_template_descriptors()
        .unwrap()
        .2
        .contains_key("fixture"));
    runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap()
        .set_editor_capabilities_enabled(&[CAPABILITY.to_string()], true)
        .expect("enable the fixture capability explicitly");
}

#[test]
fn linked_plugin_template_without_root_reports_plugin_document_template_root() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems("u01_template_without_root", &[]);
    register_and_enable(&runtime, FixturePlugin::new().registration());
    let (_, capabilities, templates) = runtime
        .runtime
        .enabled_plugin_template_descriptors()
        .unwrap();
    assert!(capabilities.contains(&CAPABILITY.to_string()));
    assert_eq!(templates["fixture"].len(), 1);
    assert_eq!(templates["fixture"][0].plugin_root(), None);

    let error = EditorUiHostRuntime::default()
        .sync_plugin_v2_template_descriptor_sets(&templates)
        .expect_err("an enabled linked template still requires a root");
    assert!(matches!(
        error,
        EditorUiHostRuntimeError::PluginDocumentTemplateRoot { owner_id, template_id }
            if owner_id == "fixture" && template_id == TEMPLATE_ID
    ));
}

#[test]
fn linked_plugin_template_with_registration_root_materializes() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems("u01_template_with_root", &[]);
    let root = fixture_root();
    assert!(root.join("editor/x.zui").is_file());
    let report = FixturePlugin::new()
        .registration()
        .with_ui_template_root(&root);
    register_and_enable(&runtime, report);
    let (_, _, templates) = runtime
        .runtime
        .enabled_plugin_template_descriptors()
        .unwrap();
    assert_eq!(templates["fixture"][0].plugin_root(), Some(root.as_path()));

    let ui_runtime = EditorUiHostRuntime::default();
    ui_runtime
        .sync_plugin_v2_template_descriptor_sets(&templates)
        .expect("the linked fixture template should materialize");
    let surface = ui_runtime
        .build_shared_surface(TEMPLATE_ID)
        .expect("materialized plugin document should build a shared surface");
    assert!(!surface.tree.nodes.is_empty());
}

#[test]
fn enabled_project_selection_without_capability_keeps_customization_unavailable() {
    let _guard = env_lock().lock().unwrap();
    let runtime =
        EventRuntimeHarness::with_enabled_subsystems("u01_project_without_capability", &[]);
    let plugin: Arc<dyn EditorPlugin + Send + Sync> = Arc::new(FixturePlugin::new());
    let manager = EditorPluginManager::from_plugins([(plugin, fixture_manifest())]).unwrap();
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .unwrap();
    let selection = ProjectPluginSelection::runtime_plugin("fixture", true, false)
        .with_target_modes([RuntimeTargetMode::EditorHost]);
    let selected = manager
        .apply_project_manifest(&ProjectPluginManifest {
            selections: vec![selection],
        })
        .expect("enable the fixture's EditorHost project selection");
    assert_eq!(
        selected.entry("fixture").unwrap().state(),
        EditorPluginState::Active
    );
    let catalog = selected.catalog_snapshot();
    let report = catalog
        .registration("fixture")
        .expect("selected linked registration");
    assert!(report.is_success());
    runtime
        .runtime
        .register_editor_plugin_registration(report.clone())
        .unwrap();

    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .unwrap()
        .inspector
        .as_ref()
        .unwrap()
        .id;
    runtime
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(COMPONENT_TYPE, "fixture", "Example")
                        .with_property("value", "scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, COMPONENT_TYPE, json!({ "value": 0.5 }))
                .unwrap();
            assert!(scene.reflect_schema(COMPONENT_TYPE).is_ok());
        });
    runtime.runtime.refresh_reflection();

    let (_, capabilities, templates) = runtime
        .runtime
        .enabled_plugin_template_descriptors()
        .unwrap();
    assert!(!capabilities.contains(&CAPABILITY.to_string()));
    assert!(!templates.contains_key("fixture"));
    let snapshot = runtime.runtime.editor_snapshot().unwrap();
    let component = snapshot
        .inspector
        .as_ref()
        .unwrap()
        .plugin_components
        .iter()
        .find(|component| component.component_id == COMPONENT_TYPE)
        .expect("the reflected fixture component should be visible");
    // U-01b characterization: project selection does not imply an editor capability.
    assert!(!component.customization_available);
    assert_eq!(component.customization_ui_document, None);
}
