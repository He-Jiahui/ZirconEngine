//! Project-manifest enablement regressions for the manager publication boundary.

use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};
use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleStage;
use crate::core::plugin::{EditorPlugin, EditorPluginDescriptor};

use super::super::{
    EditorPluginCatalog, EditorPluginLoadingPhase, EditorPluginManager, EditorPluginState,
    EditorPluginTransitionError,
};
use super::{FailOnceLifecyclePlugin, LifecyclePhasePlugin, PhasePlugin};

#[test]
fn project_manifest_applies_editor_enablement_in_one_snapshot_generation() {
    let alpha = phase_plugin("plugin.alpha", "alpha-view");
    let beta = phase_plugin("plugin.beta", "beta-view");
    let gamma = phase_plugin("plugin.gamma", "gamma-view");
    let catalog = EditorPluginCatalog::from_plugins([
        plugin_package(&alpha, "plugin.alpha"),
        plugin_package(&beta, "plugin.beta"),
        plugin_package(&gamma, "plugin.gamma"),
    ]);
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog should be valid");
    let initial = manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate fixture plugins");

    let manifest = ProjectPluginManifest {
        selections: vec![
            selection("plugin.alpha", true, [RuntimeTargetMode::EditorHost]),
            selection("plugin.beta", false, [RuntimeTargetMode::EditorHost]),
            selection("runtime.only", true, [RuntimeTargetMode::ClientRuntime]),
        ],
    };

    let applied = manager
        .apply_project_manifest(&manifest)
        .expect("the project manifest should publish one editor generation");

    assert_eq!(applied.generation(), initial.generation() + 1);
    assert_eq!(
        applied.entry("plugin.alpha").map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        applied.entry("plugin.beta").map(|entry| entry.state()),
        Some(EditorPluginState::Disabled)
    );
    assert_eq!(
        applied.entry("plugin.gamma").map(|entry| entry.state()),
        Some(EditorPluginState::Disabled)
    );
    assert_eq!(
        applied
            .active_extensions()
            .registry
            .views()
            .iter()
            .map(|view| view.id())
            .collect::<Vec<_>>(),
        ["alpha-view"]
    );

    let repeated = manager
        .apply_project_manifest(&manifest)
        .expect("the unchanged project manifest should be a stable read");
    assert!(Arc::ptr_eq(&applied, &repeated));
}

#[test]
fn project_manifest_rejects_duplicate_editor_selection_without_mutating() {
    let descriptor = EditorPluginDescriptor::new("plugin.alpha", "Alpha", "alpha");
    let catalog = EditorPluginCatalog::from_descriptors(
        [descriptor],
        [PluginPackageManifest::new("plugin.alpha", "Alpha")],
    );
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog should be valid");
    let previous = manager.state_snapshot();
    let manifest = ProjectPluginManifest {
        selections: vec![
            selection("plugin.alpha", true, [RuntimeTargetMode::EditorHost]),
            selection("plugin.alpha", false, [RuntimeTargetMode::EditorHost]),
        ],
    };

    let error = manager
        .apply_project_manifest(&manifest)
        .expect_err("duplicate editor selections must be rejected before publication");

    assert!(matches!(
        error,
        EditorPluginTransitionError::DuplicateProjectSelection { ref package_id }
            if package_id == "plugin.alpha"
    ));
    assert!(Arc::ptr_eq(&previous, &manager.state_snapshot()));
}

#[test]
fn invalid_project_manifest_does_not_dispatch_partial_lifecycle_callbacks() {
    let alpha = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.alpha", "Alpha", "alpha"),
        events: Mutex::default(),
    });
    let beta = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.beta", "Beta", "beta"),
        fail_stage: EditorPluginLifecycleStage::Disabled,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let catalog = EditorPluginCatalog::from_plugins([
        (
            Arc::clone(&alpha) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.alpha", "Alpha"),
        ),
        (
            Arc::clone(&beta) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.beta", "Beta"),
        ),
    ]);
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog should be valid");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate fixture plugins");
    manager
        .set_enabled("plugin.beta", false)
        .expect("the disabled callback failure should be recorded as faulted state");
    let previous = manager.state_snapshot();

    let error = manager
        .apply_project_manifest(&ProjectPluginManifest {
            selections: vec![selection(
                "plugin.beta",
                true,
                [RuntimeTargetMode::EditorHost],
            )],
        })
        .expect_err("a failed disabled lifecycle must reject a re-enable request");

    assert!(matches!(
        error,
        EditorPluginTransitionError::DisabledLifecycleRetryRequired { ref package_id }
            if package_id == "plugin.beta"
    ));
    assert_eq!(
        *alpha
            .events
            .lock()
            .expect("lifecycle fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
        ]
    );
    assert!(Arc::ptr_eq(&previous, &manager.state_snapshot()));
}

fn phase_plugin(package_id: &str, view_id: &'static str) -> Arc<PhasePlugin> {
    Arc::new(PhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, package_id, package_id),
        view_id,
    })
}

fn plugin_package(
    plugin: &Arc<PhasePlugin>,
    package_id: &str,
) -> (Arc<dyn EditorPlugin + Send + Sync>, PluginPackageManifest) {
    (
        Arc::clone(plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(package_id, package_id),
    )
}

fn selection(
    package_id: &str,
    enabled: bool,
    target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: package_id.to_string(),
        enabled,
        required: false,
        target_modes: target_modes.into_iter().collect(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    }
}
