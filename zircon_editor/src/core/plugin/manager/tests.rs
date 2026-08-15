use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use zircon_runtime::plugin::{PluginDependencyManifest, PluginPackageManifest};

use crate::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, ViewDescriptor,
};
use crate::core::plugin::sdk::lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleStage,
};
use crate::core::plugin::{EditorPlugin, EditorPluginDescriptor};

use super::super::admission::EditorPluginCatalogAdmissionError;

use super::{
    EditorPluginCatalog, EditorPluginDiscovery, EditorPluginDiscoveryError,
    EditorPluginLoadingPhase, EditorPluginManager, EditorPluginSource, EditorPluginState,
    EditorPluginTransitionError,
};

mod lifecycle_broadcast;
mod lifecycle_replacement;
mod lifecycle_state;
mod project_registration;
mod project_selection;
mod snapshot_publication;

struct PhasePlugin {
    descriptor: EditorPluginDescriptor,
    view_id: &'static str,
}

impl EditorPlugin for PhasePlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        registry.register_view(ViewDescriptor::new(
            self.view_id,
            self.descriptor.display_name.clone(),
            "Phase fixture",
        ))
    }
}

struct LifecyclePhasePlugin {
    descriptor: EditorPluginDescriptor,
    events: Mutex<Vec<EditorPluginLifecycleStage>>,
}

struct FailOnceLifecyclePlugin {
    descriptor: EditorPluginDescriptor,
    fail_stage: EditorPluginLifecycleStage,
    should_fail: Mutex<bool>,
    events: Mutex<Vec<EditorPluginLifecycleStage>>,
}

impl EditorPlugin for FailOnceLifecyclePlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn on_lifecycle_event(
        &self,
        event: &EditorPluginLifecycleEvent,
    ) -> Result<(), EditorPluginLifecycleError> {
        self.events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned")
            .push(event.stage().clone());
        let mut should_fail = self
            .should_fail
            .lock()
            .expect("lifecycle failure fixture lock should not be poisoned");
        if *should_fail && event.stage() == &self.fail_stage {
            *should_fail = false;
            return Err(EditorPluginLifecycleError::new(
                event.stage().clone(),
                "simulated one-time lifecycle failure",
            ));
        }
        Ok(())
    }
}

impl EditorPlugin for LifecyclePhasePlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn on_lifecycle_event(
        &self,
        event: &EditorPluginLifecycleEvent,
    ) -> Result<(), EditorPluginLifecycleError> {
        self.events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned")
            .push(event.stage().clone());
        Ok(())
    }
}

#[test]
fn failed_enablement_request_keeps_the_current_manager_generation() {
    let catalog = EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.sample",
            "Sample",
            "sample",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");
    manager
        .transition_state("plugin.sample", EditorPluginState::Faulted)
        .expect("an active plugin may fault");
    let previous = manager.state_snapshot();

    let error = manager
        .set_enabled("plugin.sample", false)
        .expect_err("a faulted plugin cannot be disabled without recovery");

    assert!(matches!(
        error,
        EditorPluginTransitionError::InvalidEnablement { .. }
    ));
    assert!(Arc::ptr_eq(&previous, &manager.state_snapshot()));
}

#[test]
fn initial_generation_keeps_the_validated_discovery_source_and_phase() {
    let catalog = EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.project",
            "Project",
            "project",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let manager = EditorPluginManager::new_with_discoveries(
        catalog,
        [EditorPluginDiscovery::project("plugin.project")],
    )
    .expect("known package discovery should be accepted");

    let entry = manager
        .state_snapshot()
        .entry("plugin.project")
        .expect("project plugin should be published");
    assert_eq!(entry.source(), EditorPluginSource::Project);
    assert_eq!(
        entry.loading_phase(),
        EditorPluginLoadingPhase::PreWorkbench
    );
    assert_eq!(entry.state(), EditorPluginState::Validated);
}

#[test]
fn loading_phase_publishes_only_eligible_extensions_in_one_manager_generation() {
    let pre_workbench = Arc::new(PhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.pre", "Pre Workbench", "plugin_pre"),
        view_id: "plugin.phase.pre",
    });
    let default = Arc::new(PhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.default", "Default", "plugin_default"),
        view_id: "plugin.phase.default",
    });
    let post_workbench = Arc::new(PhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.post", "Post Workbench", "plugin_post"),
        view_id: "plugin.phase.post",
    });
    let catalog = EditorPluginCatalog::from_plugins([
        (
            Arc::clone(&pre_workbench) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.pre", "Pre Workbench"),
        ),
        (
            Arc::clone(&default) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.default", "Default"),
        ),
        (
            Arc::clone(&post_workbench) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.post", "Post Workbench"),
        ),
    ]);
    let manager = EditorPluginManager::new_with_discoveries(
        catalog,
        [
            EditorPluginDiscovery::new(
                "plugin.pre",
                EditorPluginSource::Project,
                EditorPluginLoadingPhase::PreWorkbench,
            ),
            EditorPluginDiscovery::new(
                "plugin.default",
                EditorPluginSource::Builtin,
                EditorPluginLoadingPhase::Default,
            ),
            EditorPluginDiscovery::new(
                "plugin.post",
                EditorPluginSource::PackageManifest,
                EditorPluginLoadingPhase::PostWorkbench,
            ),
        ],
    )
    .expect("phase fixtures should be admitted");

    let initial = manager.state_snapshot();
    assert_eq!(initial.reached_loading_phase(), None);
    assert_eq!(
        initial.active_extensions().active_manager_generation,
        Some(initial.generation())
    );
    assert!(initial.active_extensions().registry.views().is_empty());
    assert!(initial
        .entries()
        .iter()
        .all(|entry| entry.state() == EditorPluginState::Validated));

    let pre = manager
        .advance_loading_phase(EditorPluginLoadingPhase::PreWorkbench)
        .expect("the pre-workbench phase should publish");
    assert_eq!(
        pre.entry("plugin.pre").map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        pre.entry("plugin.default").map(|entry| entry.state()),
        Some(EditorPluginState::Validated)
    );
    assert_eq!(
        pre.active_extensions()
            .registry
            .views()
            .iter()
            .map(|view| view.id())
            .collect::<Vec<_>>(),
        ["plugin.phase.pre"]
    );
    assert_eq!(
        pre.active_extensions().active_manager_generation,
        Some(pre.generation())
    );

    let default = manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should publish");
    assert_eq!(
        default
            .active_extensions()
            .registry
            .views()
            .iter()
            .map(|view| view.id())
            .collect::<Vec<_>>(),
        ["plugin.phase.pre", "plugin.phase.default"]
    );
    assert_eq!(
        default.active_extensions().active_manager_generation,
        Some(default.generation())
    );
    let repeated = manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("repeating the reached phase should be a stable read");
    assert!(Arc::ptr_eq(&default, &repeated));
    let error = manager
        .advance_loading_phase(EditorPluginLoadingPhase::PreWorkbench)
        .expect_err("the phase scheduler must not move backward");
    assert!(matches!(
        error,
        EditorPluginTransitionError::InvalidLoadingPhaseAdvance { .. }
    ));
}

#[test]
fn loading_phase_dispatches_lifecycle_only_when_the_manager_reaches_the_phase() {
    let plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(
            "plugin.lifecycle.post",
            "Post Workbench Lifecycle",
            "plugin_lifecycle_post",
        ),
        events: Mutex::default(),
    });
    let catalog = EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new("plugin.lifecycle.post", "Post Workbench Lifecycle"),
    )]);
    let manager = EditorPluginManager::new_with_discoveries(
        catalog,
        [EditorPluginDiscovery::new(
            "plugin.lifecycle.post",
            EditorPluginSource::Project,
            EditorPluginLoadingPhase::PostWorkbench,
        )],
    )
    .expect("the lifecycle fixture should be admitted");

    assert!(plugin
        .events
        .lock()
        .expect("lifecycle event fixture lock should not be poisoned")
        .is_empty());
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::PreWorkbench)
        .expect("the pre-workbench phase should publish without lifecycle callbacks");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should publish without lifecycle callbacks");
    assert!(plugin
        .events
        .lock()
        .expect("lifecycle event fixture lock should not be poisoned")
        .is_empty());

    manager
        .advance_loading_phase(EditorPluginLoadingPhase::PostWorkbench)
        .expect("the post-workbench phase should dispatch lifecycle callbacks");
    assert_eq!(
        *plugin
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
        ]
    );
}

#[test]
fn lifecycle_retry_retries_only_unsuccessful_stages_and_clears_the_manager_fault() {
    for (package_id, fail_stage, expected_events) in [
        (
            "plugin.retry.loaded",
            EditorPluginLifecycleStage::Loaded,
            vec![
                EditorPluginLifecycleStage::Loaded,
                EditorPluginLifecycleStage::Loaded,
                EditorPluginLifecycleStage::Enabled,
            ],
        ),
        (
            "plugin.retry.enabled",
            EditorPluginLifecycleStage::Enabled,
            vec![
                EditorPluginLifecycleStage::Loaded,
                EditorPluginLifecycleStage::Enabled,
                EditorPluginLifecycleStage::Enabled,
            ],
        ),
    ] {
        let plugin = Arc::new(FailOnceLifecyclePlugin {
            descriptor: EditorPluginDescriptor::new(package_id, package_id, package_id),
            fail_stage,
            should_fail: Mutex::new(true),
            events: Mutex::default(),
        });
        let catalog = EditorPluginCatalog::from_plugins([(
            Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(package_id, package_id),
        )]);
        let manager =
            EditorPluginManager::new(catalog).expect("the retry fixture catalog is admissible");

        let failed = manager
            .advance_loading_phase(EditorPluginLoadingPhase::Default)
            .expect("the initial phase dispatch should publish the faulted state");
        assert_eq!(
            failed.entry(package_id).map(|entry| entry.state()),
            Some(EditorPluginState::Faulted)
        );

        let recovered = manager
            .set_enabled(package_id, true)
            .expect("the manager should retry the failed lifecycle stage");
        assert_eq!(
            recovered.entry(package_id).map(|entry| entry.state()),
            Some(EditorPluginState::Active)
        );
        assert!(recovered
            .catalog_snapshot()
            .registration(package_id)
            .expect("the retry fixture registration should remain published")
            .is_success());
        assert_eq!(
            *plugin
                .events
                .lock()
                .expect("lifecycle event fixture lock should not be poisoned"),
            expected_events
        );
    }
}

#[test]
fn serialized_once_initialization_runs_the_creator_once() {
    const WORKERS: usize = 4;

    let slot = Arc::new(OnceLock::new());
    let initialization = Arc::new(Mutex::new(()));
    let calls = Arc::new(AtomicUsize::new(0));
    let workers = (0..WORKERS)
        .map(|_| {
            let slot = Arc::clone(&slot);
            let initialization = Arc::clone(&initialization);
            let calls = Arc::clone(&calls);
            thread::spawn(move || {
                let value = initialize_once(slot.as_ref(), initialization.as_ref(), || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, ()>(7usize)
                })
                .expect("serialized initializer should succeed");
                assert_eq!(*value, 7);
            })
        })
        .collect::<Vec<_>>();

    for worker in workers {
        worker
            .join()
            .expect("serialized initializer worker should not panic");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn transition_state_cannot_activate_a_plugin_before_its_loading_phase() {
    let catalog = EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "plugin.post",
            "Post Workbench",
            "plugin_post",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let manager = EditorPluginManager::new_with_discoveries(
        catalog,
        [EditorPluginDiscovery::new(
            "plugin.post",
            EditorPluginSource::Project,
            EditorPluginLoadingPhase::PostWorkbench,
        )],
    )
    .expect("the phase fixture should be admitted");
    let previous = manager.state_snapshot();

    let error = manager
        .transition_state("plugin.post", EditorPluginState::Loading)
        .expect_err("the loading phase has not been reached");

    assert!(matches!(
        error,
        EditorPluginTransitionError::LoadingPhaseUnavailable { .. }
    ));
    assert!(Arc::ptr_eq(&previous, &manager.state_snapshot()));
}

#[test]
fn publishing_later_phase_metadata_requires_explicit_disable_of_an_active_entry() {
    let catalog = EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "plugin.sample",
            "Sample",
            "plugin_sample",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let manager = EditorPluginManager::new_with_discoveries(
        catalog,
        [EditorPluginDiscovery::builtin("plugin.sample")],
    )
    .expect("the default-phase fixture should be admitted");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate the fixture");

    let replacement = EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "plugin.sample",
            "Sample",
            "plugin_sample",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let error = manager
        .publish_catalog_with_discoveries(
            replacement,
            [EditorPluginDiscovery::new(
                "plugin.sample",
                EditorPluginSource::Builtin,
                EditorPluginLoadingPhase::PostWorkbench,
            )],
        )
        .expect_err("rephasing an active plugin must not skip its disabled lifecycle callback");
    let current = manager.state_snapshot();

    assert!(matches!(
        error,
        EditorPluginDiscoveryError::PhaseRetractionRequiresDisable { .. }
    ));
    assert_eq!(
        current.entry("plugin.sample").map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        current.reached_loading_phase(),
        Some(EditorPluginLoadingPhase::Default)
    );
    assert_eq!(
        current.active_extensions().active_manager_generation,
        Some(current.generation())
    );
}

#[test]
fn initial_generation_rejects_duplicate_or_unknown_discoveries() {
    let catalog = EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.project",
            "Project",
            "project",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let duplicate = EditorPluginManager::new_with_discoveries(
        catalog,
        [
            EditorPluginDiscovery::builtin("plugin.project"),
            EditorPluginDiscovery::project("plugin.project"),
        ],
    )
    .expect_err("duplicate discovery must be rejected");
    assert!(matches!(
        duplicate,
        EditorPluginDiscoveryError::DuplicateDiscovery { .. }
    ));
}

#[test]
fn initial_generation_rejects_declared_package_dependency_cycles() {
    let catalog = catalog_with_dependencies(&[
        ("plugin.alpha", "plugin.beta"),
        ("plugin.beta", "plugin.alpha"),
    ]);

    let error = EditorPluginManager::new_with_discoveries(catalog, [])
        .expect_err("a package dependency cycle cannot enter the initial generation");
    assert!(matches!(
        error,
        EditorPluginDiscoveryError::CatalogAdmission(
            EditorPluginCatalogAdmissionError::DependencyCycle { .. }
        )
    ));
}

#[test]
fn new_returns_a_dependency_cycle_as_an_admission_error() {
    let catalog = catalog_with_dependencies(&[
        ("plugin.alpha", "plugin.beta"),
        ("plugin.beta", "plugin.alpha"),
    ]);

    let error = EditorPluginManager::new(catalog)
        .expect_err("the non-discovery facade must return a catalog admission failure");
    assert!(matches!(
        error,
        EditorPluginDiscoveryError::CatalogAdmission(
            EditorPluginCatalogAdmissionError::DependencyCycle { .. }
        )
    ));
}

#[test]
fn publish_catalog_returns_a_cycle_and_keeps_the_published_generation() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::default())
        .expect("an empty catalog is admissible");
    let previous = manager.state_snapshot();
    let replacement = catalog_with_dependencies(&[
        ("plugin.alpha", "plugin.beta"),
        ("plugin.beta", "plugin.alpha"),
    ]);

    let error = manager
        .publish_catalog(replacement)
        .expect_err("the non-discovery facade must reject a package dependency cycle");

    assert!(matches!(
        error,
        EditorPluginDiscoveryError::CatalogAdmission(
            EditorPluginCatalogAdmissionError::DependencyCycle { .. }
        )
    ));
    assert!(Arc::ptr_eq(&previous, &manager.state_snapshot()));
    assert_eq!(manager.state_snapshot().generation(), 1);
}

#[test]
fn publish_replaces_discovery_metadata_with_the_new_generation() {
    let catalog = EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.project",
            "Project",
            "project",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");
    let replacement = EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.project",
            "Project",
            "project",
        )],
        Vec::<PluginPackageManifest>::new(),
    );

    manager
        .publish_catalog_with_discoveries(
            replacement,
            [EditorPluginDiscovery::project("plugin.project")],
        )
        .expect("known project discovery should publish");

    let entry = manager
        .state_snapshot()
        .entry("plugin.project")
        .expect("replacement entry should be published");
    assert_eq!(entry.source(), EditorPluginSource::Project);
    assert_eq!(
        entry.loading_phase(),
        EditorPluginLoadingPhase::PreWorkbench
    );
}

fn catalog_with_dependencies(dependencies: &[(&str, &str)]) -> EditorPluginCatalog {
    let descriptors = dependencies
        .iter()
        .map(|(package_id, _)| EditorPluginDescriptor::new(*package_id, package_id, package_id))
        .collect::<Vec<_>>();
    let manifests = dependencies
        .iter()
        .map(|(package_id, dependency_id)| {
            let mut manifest = PluginPackageManifest::new(*package_id, package_id);
            manifest
                .dependencies
                .push(PluginDependencyManifest::new(*dependency_id, true));
            manifest
        })
        .collect::<Vec<_>>();
    EditorPluginCatalog::from_descriptors(descriptors, manifests)
}
