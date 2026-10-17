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
    initialize_once, EditorPluginCatalog, EditorPluginDiscovery, EditorPluginDiscoveryError,
    EditorPluginLoadingPhase, EditorPluginManager, EditorPluginSource, EditorPluginState,
    EditorPluginTransitionError,
};

#[path = "lifecycle_broadcast.rs"]
mod lifecycle_broadcast;
#[path = "lifecycle_replacement.rs"]
mod lifecycle_replacement;
#[path = "lifecycle_state.rs"]
mod lifecycle_state;
#[path = "project_registration.rs"]
mod project_registration;
#[path = "project_selection.rs"]
mod project_selection;
#[path = "snapshot_publication.rs"]
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

    let snapshot = manager.state_snapshot();
    let entry = snapshot
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

    let snapshot = manager.state_snapshot();
    let entry = snapshot
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
        .map(|(package_id, _)| EditorPluginDescriptor::new(*package_id, *package_id, *package_id))
        .collect::<Vec<_>>();
    let manifests = dependencies
        .iter()
        .map(|(package_id, dependency_id)| {
            let mut manifest = PluginPackageManifest::new(*package_id, *package_id);
            manifest
                .dependencies
                .push(PluginDependencyManifest::new(*dependency_id, true));
            manifest
        })
        .collect::<Vec<_>>();
    EditorPluginCatalog::from_descriptors(descriptors, manifests)
}

fn retirement_test_plugin(package_id: &str) -> Arc<LifecyclePhasePlugin> {
    Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, package_id, package_id),
        events: Mutex::default(),
    })
}

fn retirement_test_failure(
    package_id: &str,
    fail_stage: EditorPluginLifecycleStage,
    should_fail: bool,
) -> Arc<FailOnceLifecyclePlugin> {
    Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, package_id, package_id),
        fail_stage,
        should_fail: Mutex::new(should_fail),
        events: Mutex::default(),
    })
}

fn retirement_test_catalog(plugins: &[Arc<dyn EditorPlugin + Send + Sync>]) -> EditorPluginCatalog {
    EditorPluginCatalog::from_plugins(plugins.iter().map(|plugin| {
        (
            Arc::clone(plugin),
            PluginPackageManifest::new(
                plugin.descriptor().package_id.clone(),
                plugin.descriptor().display_name.clone(),
            ),
        )
    }))
}

fn retirement_test_manager(plugins: &[Arc<dyn EditorPlugin + Send + Sync>]) -> EditorPluginManager {
    let manager = EditorPluginManager::new(retirement_test_catalog(plugins))
        .expect("the lifecycle fixture should be admitted");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should dispatch the fixture callbacks");
    manager
}

fn retirement_test_events(
    events: &Mutex<Vec<EditorPluginLifecycleStage>>,
) -> Vec<EditorPluginLifecycleStage> {
    events
        .lock()
        .expect("fixture events should be available")
        .clone()
}

fn retirement_test_progress(manager: &EditorPluginManager, package_id: &str) -> [bool; 2] {
    let catalog = manager.state_snapshot().catalog_snapshot().clone_catalog();
    [
        catalog.retirement_stage_succeeded(package_id, &EditorPluginLifecycleStage::Disabled),
        catalog.retirement_stage_succeeded(package_id, &EditorPluginLifecycleStage::Unloaded),
    ]
}

fn retirement_test_history(
    manager: &EditorPluginManager,
    package_id: &str,
) -> Vec<EditorPluginLifecycleStage> {
    let snapshot = manager.state_snapshot();
    snapshot
        .catalog_snapshot()
        .registration(package_id)
        .expect("the fixture registration should remain published")
        .lifecycle
        .records()
        .iter()
        .map(|record| record.event().stage().clone())
        .collect()
}

fn retirement_test_host_report(
    package_id: &str,
) -> crate::core::plugin::EditorPluginRegistrationReport {
    use crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleReport;
    use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;
    use zircon_runtime::plugin::PluginModuleManifest;

    crate::core::plugin::EditorPluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(package_id, package_id).with_module(
            PluginModuleManifest::editor(
                format!("{package_id}.editor"),
                package_id.replace('.', "_"),
            ),
        ),
        capabilities: Vec::new(),
        extensions: EditorExtensionRegistry::default(),
        lifecycle: EditorPluginLifecycleReport::default(),
        successful_lifecycle_stages: Vec::new(),
        failed_lifecycle_stages: Vec::new(),
        runtime_event_consumers: EditorRuntimeEventConsumerRegistry::default(),
        native_command_bindings: Default::default(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn replacement_retires_reenabled_instance_in_current_cycle() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, HotReloaded, Loaded, Unloaded};

    let old = retirement_test_plugin("fixture.cycle");
    let manager = retirement_test_manager(&[old.clone()]);
    manager
        .set_enabled("fixture.cycle", false)
        .expect("disable should finish");
    manager
        .set_enabled("fixture.cycle", true)
        .expect("reenable should finish");
    let reenabled = manager.state_snapshot();
    assert_eq!(
        reenabled.entry("fixture.cycle").unwrap().state(),
        EditorPluginState::Active
    );
    assert!(reenabled
        .catalog_snapshot()
        .registration("fixture.cycle")
        .unwrap()
        .lifecycle_stage_succeeded(&Disabled));
    assert_eq!(
        retirement_test_progress(&manager, "fixture.cycle"),
        [false, false]
    );

    let replacement = retirement_test_plugin("fixture.cycle");
    manager
        .publish_catalog(retirement_test_catalog(&[replacement.clone()]))
        .expect("replacement should retire the current activation");
    assert_eq!(
        retirement_test_events(&old.events),
        vec![Loaded, Enabled, Disabled, Enabled, Disabled, Unloaded]
    );
    assert_eq!(
        retirement_test_events(&replacement.events),
        vec![Loaded, Enabled, HotReloaded]
    );
    assert_eq!(
        manager
            .state_snapshot()
            .entry("fixture.cycle")
            .unwrap()
            .state(),
        EditorPluginState::Active
    );
    assert_eq!(
        retirement_test_history(&manager, "fixture.cycle"),
        retirement_test_events(&replacement.events)
    );
}

#[test]
fn replacement_retries_only_current_cycle_cleanup_failures() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, HotReloaded, Loaded, Unloaded};

    for fail_stage in [Disabled, Unloaded] {
        let old = retirement_test_failure("fixture.retry.cycle", fail_stage.clone(), false);
        let manager = retirement_test_manager(&[old.clone()]);
        manager
            .set_enabled("fixture.retry.cycle", false)
            .expect("initial disable should finish");
        manager
            .set_enabled("fixture.retry.cycle", true)
            .expect("reenable should finish");
        *old.should_fail.lock().unwrap() = true;
        let replacement = retirement_test_plugin("fixture.retry.cycle");
        let error = manager
            .publish_catalog(retirement_test_catalog(&[replacement.clone()]))
            .expect_err("the current cleanup callback must run and fail once");
        assert!(
            matches!(error, EditorPluginDiscoveryError::LifecycleCleanupFailed { stage, .. }
            if stage == fail_stage)
        );
        assert_eq!(
            manager
                .state_snapshot()
                .entry("fixture.retry.cycle")
                .unwrap()
                .state(),
            EditorPluginState::Faulted
        );
        assert_eq!(
            retirement_test_progress(&manager, "fixture.retry.cycle"),
            [fail_stage == Unloaded, false]
        );
        assert!(retirement_test_events(&replacement.events).is_empty());
        let interrupted = manager.state_snapshot();
        let registration = interrupted
            .catalog_snapshot()
            .registration("fixture.retry.cycle")
            .unwrap();
        assert!(registration.lifecycle_stage_succeeded(&Disabled));
        assert!(registration.lifecycle_stage_failed(&fail_stage));
        assert_eq!(
            retirement_test_history(&manager, "fixture.retry.cycle"),
            retirement_test_events(&old.events)
        );

        manager
            .publish_catalog(retirement_test_catalog(&[replacement.clone()]))
            .expect("retry should finish only pending cleanup steps");
        let expected = if fail_stage == Disabled {
            vec![
                Loaded, Enabled, Disabled, Enabled, Disabled, Disabled, Unloaded,
            ]
        } else {
            vec![
                Loaded, Enabled, Disabled, Enabled, Disabled, Unloaded, Unloaded,
            ]
        };
        assert_eq!(retirement_test_events(&old.events), expected);
        assert_eq!(
            retirement_test_events(&replacement.events),
            vec![Loaded, Enabled, HotReloaded]
        );
        assert_eq!(
            manager
                .state_snapshot()
                .entry("fixture.retry.cycle")
                .unwrap()
                .state(),
            EditorPluginState::Active
        );
    }
}

#[test]
fn reenable_attempt_starts_fresh_cleanup_even_when_enabled_fails() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, Loaded, Unloaded};

    for previously_enabled in [false, true] {
        let old = retirement_test_failure("fixture.a.enabled", Enabled, !previously_enabled);
        let blocker = retirement_test_failure("fixture.b.blocker", Disabled, true);
        let manager = retirement_test_manager(&[old.clone(), blocker.clone()]);
        if previously_enabled {
            let unused_owner = retirement_test_plugin("fixture.a.enabled");
            let unused = retirement_test_plugin("fixture.b.blocker");
            let error = manager
                .publish_catalog(retirement_test_catalog(&[unused_owner, unused]))
                .expect_err(
                    "the later owner should interrupt retirement after the first owner finishes",
                );
            assert!(matches!(
                error,
                EditorPluginDiscoveryError::LifecycleCleanupFailed { .. }
            ));
            assert_eq!(
                retirement_test_progress(&manager, "fixture.a.enabled"),
                [true, true]
            );
            assert_eq!(
                manager
                    .state_snapshot()
                    .entry("fixture.a.enabled")
                    .unwrap()
                    .state(),
                EditorPluginState::Revoking
            );
            manager
                .transition_state("fixture.a.enabled", EditorPluginState::Faulted)
                .expect("the retired owner may fault before a new activation attempt");
            *old.should_fail.lock().unwrap() = true;
            manager
                .set_enabled("fixture.a.enabled", true)
                .expect("the failed Enabled attempt should publish its faulted state");
        }
        assert_eq!(
            manager
                .state_snapshot()
                .entry("fixture.a.enabled")
                .unwrap()
                .state(),
            EditorPluginState::Faulted
        );
        assert_eq!(
            retirement_test_progress(&manager, "fixture.a.enabled"),
            [false, false]
        );
        let faulted = manager.state_snapshot();
        let registration = faulted
            .catalog_snapshot()
            .registration("fixture.a.enabled")
            .unwrap();
        assert_eq!(
            registration.lifecycle_stage_succeeded(&Enabled),
            previously_enabled
        );
        assert!(registration.lifecycle_stage_failed(&Enabled));
        let replacement = retirement_test_plugin("fixture.a.enabled");
        let blocker_handle: Arc<dyn EditorPlugin + Send + Sync> = blocker.clone();
        manager
            .publish_catalog(retirement_test_catalog(&[
                replacement.clone(),
                blocker_handle,
            ]))
            .expect("replacement should follow the old owner's actual activation history");
        let expected = if previously_enabled {
            vec![
                Loaded, Enabled, Disabled, Unloaded, Enabled, Disabled, Unloaded,
            ]
        } else {
            vec![Loaded, Enabled]
        };
        assert_eq!(retirement_test_events(&old.events), expected);
        assert_eq!(
            retirement_test_events(&old.events)
                .iter()
                .filter(|stage| **stage == Loaded)
                .count(),
            1
        );
    }
}

#[test]
fn same_plugin_catalog_refresh_preserves_lifecycle_authority() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, HotReloaded, Loaded, Unloaded};

    let old = retirement_test_failure("fixture.refresh", Unloaded, true);
    let manager = retirement_test_manager(&[old.clone()]);
    let replacement = retirement_test_plugin("fixture.refresh");
    manager
        .publish_catalog(retirement_test_catalog(&[replacement.clone()]))
        .expect_err("the first real unload should fail");
    let interrupted = manager.state_snapshot();
    let original = interrupted
        .catalog_snapshot()
        .registration("fixture.refresh")
        .unwrap();
    let history = original.lifecycle.clone();
    let successful = original.successful_lifecycle_stages.clone();
    let failed = original.failed_lifecycle_stages.clone();
    let before_events = retirement_test_events(&old.events);

    let mut refreshed = retirement_test_catalog(&[old.clone()]);
    refreshed.registrations[0].capabilities = vec!["candidate.metadata".to_string()];
    manager
        .publish_catalog(refreshed)
        .expect("the same Arc may refresh its catalog metadata");
    let current = manager.state_snapshot();
    let registration = current
        .catalog_snapshot()
        .registration("fixture.refresh")
        .unwrap();
    assert_eq!(
        current.entry("fixture.refresh").unwrap().state(),
        EditorPluginState::Faulted
    );
    assert_eq!(registration.lifecycle, history);
    assert_eq!(registration.successful_lifecycle_stages, successful);
    assert_eq!(registration.failed_lifecycle_stages, failed);
    assert_eq!(
        registration.capabilities,
        vec!["candidate.metadata".to_string()]
    );
    assert_eq!(
        retirement_test_progress(&manager, "fixture.refresh"),
        [true, false]
    );
    assert_eq!(retirement_test_events(&old.events), before_events);

    let mut changed_owner = current.catalog_snapshot().clone_catalog();
    changed_owner.register(
        replacement.clone(),
        PluginPackageManifest::new("fixture.refresh", "replacement"),
    );
    assert!(!changed_owner.retirement_stage_succeeded("fixture.refresh", &Disabled));
    assert!(!changed_owner.retirement_stage_succeeded("fixture.refresh", &Unloaded));
    assert_eq!(
        retirement_test_progress(&manager, "fixture.refresh"),
        [true, false]
    );

    manager
        .publish_catalog(retirement_test_catalog(&[replacement.clone()]))
        .expect("retirement must remain eligible and retry the pending unload");
    assert_eq!(
        retirement_test_events(&old.events),
        vec![Loaded, Enabled, Disabled, Unloaded, Unloaded]
    );
    assert_eq!(
        retirement_test_events(&replacement.events),
        vec![Loaded, Enabled, HotReloaded]
    );
}

#[test]
fn native_cleanup_progress_survives_interrupted_replacement_retries() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, HotReloaded, Loaded, Unloaded};

    let native_id = "fixture.a.native";
    let first = retirement_test_failure("fixture.b.blocker", Disabled, true);
    let second = retirement_test_failure("fixture.c.blocker", Disabled, true);
    let manager = retirement_test_manager(&[first.clone(), second.clone()]);
    manager
        .publish_project_registration_reports([retirement_test_host_report(native_id)])
        .expect("the native host registration should activate through the production project path");
    assert_eq!(
        retirement_test_history(&manager, native_id),
        vec![Loaded, Enabled]
    );

    let new_first = retirement_test_plugin("fixture.b.blocker");
    let new_second = retirement_test_plugin("fixture.c.blocker");
    let candidate = || {
        let mut catalog = retirement_test_catalog(&[new_first.clone(), new_second.clone()]);
        catalog.replace_project_registration_reports(
            &Default::default(),
            [retirement_test_host_report(native_id)],
        );
        catalog
    };
    for blocked_id in ["fixture.b.blocker", "fixture.c.blocker"] {
        let error = manager
            .publish_catalog_with_discoveries(
                candidate(),
                [EditorPluginDiscovery::project(native_id)],
            )
            .expect_err("a later Rust owner should interrupt the real native retirement");
        assert!(
            matches!(error, EditorPluginDiscoveryError::LifecycleCleanupFailed { package_id, stage }
            if package_id == blocked_id && stage == Disabled)
        );
        let interrupted = manager.state_snapshot();
        assert_eq!(
            interrupted.entry(native_id).unwrap().source(),
            EditorPluginSource::Project
        );
        assert_eq!(
            interrupted.entry(native_id).unwrap().state(),
            EditorPluginState::Revoking
        );
        assert_eq!(retirement_test_progress(&manager, native_id), [true, true]);
        assert_eq!(
            retirement_test_history(&manager, native_id),
            vec![Loaded, Enabled, Disabled, Unloaded]
        );
        assert!(retirement_test_events(&new_first.events).is_empty());
        assert!(retirement_test_events(&new_second.events).is_empty());
    }

    let previous = manager.state_snapshot();
    let mut fresh_catalog = previous.catalog_snapshot().clone_catalog();
    let project_ids = std::collections::BTreeSet::from([native_id.to_string()]);
    fresh_catalog.replace_project_registration_reports(
        &project_ids,
        [retirement_test_host_report(native_id)],
    );
    assert!(!fresh_catalog.retirement_stage_succeeded(native_id, &Disabled));
    assert!(!fresh_catalog.retirement_stage_succeeded(native_id, &Unloaded));
    manager
        .publish_project_registration_reports([retirement_test_host_report(native_id)])
        .expect("a same-ID native report must begin a new host lifecycle generation");
    assert_eq!(
        retirement_test_progress(&manager, native_id),
        [false, false]
    );
    assert_eq!(
        retirement_test_history(&manager, native_id),
        vec![Loaded, Enabled, HotReloaded]
    );
    assert_eq!(
        manager.state_snapshot().entry(native_id).unwrap().state(),
        EditorPluginState::Active
    );

    *second.should_fail.lock().unwrap() = true;
    let error = manager
        .publish_catalog_with_discoveries(candidate(), [EditorPluginDiscovery::project(native_id)])
        .expect_err("the fresh native owner should retire before another interrupted publication");
    assert!(
        matches!(error, EditorPluginDiscoveryError::LifecycleCleanupFailed { package_id, stage }
        if package_id == "fixture.c.blocker" && stage == Disabled)
    );
    assert_eq!(retirement_test_progress(&manager, native_id), [true, true]);
    assert_eq!(
        retirement_test_history(&manager, native_id),
        vec![Loaded, Enabled, HotReloaded, Disabled, Unloaded]
    );
    manager
        .publish_catalog_with_discoveries(candidate(), [EditorPluginDiscovery::project(native_id)])
        .expect("the final retry should preserve completed native cleanup");
    assert_eq!(
        retirement_test_events(&first.events),
        vec![Loaded, Enabled, Disabled, Disabled, Unloaded]
    );
    assert_eq!(
        retirement_test_events(&second.events),
        vec![Loaded, Enabled, Disabled, Disabled, Disabled, Unloaded]
    );
    assert_eq!(
        retirement_test_events(&new_first.events),
        vec![Loaded, Enabled, HotReloaded]
    );
    assert_eq!(
        retirement_test_events(&new_second.events),
        vec![Loaded, Enabled, HotReloaded]
    );
    assert_eq!(
        retirement_test_history(&manager, native_id),
        vec![Loaded, Enabled, HotReloaded]
    );
    assert_eq!(
        retirement_test_progress(&manager, native_id),
        [false, false]
    );
}

#[test]
fn ordinary_unloaded_notifications_do_not_complete_retirement() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, HotReloaded, Loaded, Unloaded};

    for broadcast in [false, true] {
        for fails in [false, true] {
            let old = retirement_test_failure("fixture.notification", Unloaded, fails);
            let manager = retirement_test_manager(&[old.clone()]);
            let event = EditorPluginLifecycleEvent::new(Unloaded);
            let report = if broadcast {
                manager.dispatch_lifecycle_event_to_active(event)
            } else {
                manager.dispatch_lifecycle_event("fixture.notification", event)
            }
            .expect("Unloaded is an ordinary lifecycle notification");
            assert_eq!(report.is_success(), !fails);
            assert_eq!(report.records().len(), 1);
            assert_eq!(
                manager
                    .state_snapshot()
                    .entry("fixture.notification")
                    .unwrap()
                    .state(),
                if fails {
                    EditorPluginState::Faulted
                } else {
                    EditorPluginState::Active
                }
            );
            assert_eq!(
                retirement_test_progress(&manager, "fixture.notification"),
                [false, false]
            );
            assert_eq!(
                retirement_test_history(&manager, "fixture.notification"),
                vec![Loaded, Enabled, Unloaded]
            );
            let notified = manager.state_snapshot();
            let registration = notified
                .catalog_snapshot()
                .registration("fixture.notification")
                .unwrap();
            assert_eq!(registration.lifecycle_stage_failed(&Unloaded), fails);
            assert_eq!(registration.lifecycle_stage_succeeded(&Unloaded), !fails);

            let replacement = retirement_test_plugin("fixture.notification");
            manager
                .publish_catalog(retirement_test_catalog(&[replacement.clone()]))
                .expect("an ordinary notification cannot stand in for actual retirement");
            assert_eq!(
                retirement_test_events(&old.events),
                vec![Loaded, Enabled, Unloaded, Disabled, Unloaded]
            );
            assert_eq!(
                retirement_test_events(&replacement.events),
                vec![Loaded, Enabled, HotReloaded]
            );
            assert_eq!(
                manager
                    .state_snapshot()
                    .entry("fixture.notification")
                    .unwrap()
                    .state(),
                EditorPluginState::Active
            );
            assert_eq!(
                retirement_test_progress(&manager, "fixture.notification"),
                [false, false]
            );
        }
    }
}

#[test]
fn ordinary_unloaded_failure_preserves_completed_retirement_progress() {
    use EditorPluginLifecycleStage::{Disabled, Enabled, HotReloaded, Loaded, Unloaded};

    let old = retirement_test_failure("fixture.a.retired", Unloaded, false);
    let blocker = retirement_test_failure("fixture.b.blocker", Disabled, true);
    let manager = retirement_test_manager(&[old.clone(), blocker.clone()]);
    let replacement = retirement_test_plugin("fixture.a.retired");
    let new_blocker = retirement_test_plugin("fixture.b.blocker");
    manager
        .publish_catalog(retirement_test_catalog(&[
            replacement.clone(),
            new_blocker.clone(),
        ]))
        .expect_err("a later failure should leave the first owner's actual retirement complete");
    assert_eq!(
        manager
            .state_snapshot()
            .entry("fixture.a.retired")
            .unwrap()
            .state(),
        EditorPluginState::Revoking
    );
    assert_eq!(
        retirement_test_progress(&manager, "fixture.a.retired"),
        [true, true]
    );

    *old.should_fail.lock().unwrap() = true;
    let report = manager
        .dispatch_lifecycle_event(
            "fixture.a.retired",
            EditorPluginLifecycleEvent::new(Unloaded),
        )
        .expect("a targeted ordinary notification may reach the retained old owner");
    assert!(!report.is_success());
    assert_eq!(
        manager
            .state_snapshot()
            .entry("fixture.a.retired")
            .unwrap()
            .state(),
        EditorPluginState::Faulted
    );
    assert_eq!(
        retirement_test_progress(&manager, "fixture.a.retired"),
        [true, true]
    );
    assert_eq!(
        retirement_test_history(&manager, "fixture.a.retired"),
        vec![Loaded, Enabled, Disabled, Unloaded, Unloaded]
    );
    let interrupted = manager.state_snapshot();
    let registration = interrupted
        .catalog_snapshot()
        .registration("fixture.a.retired")
        .unwrap();
    assert!(registration.lifecycle_stage_succeeded(&Unloaded));
    assert!(registration.lifecycle_stage_failed(&Unloaded));

    manager.publish_catalog(retirement_test_catalog(&[replacement.clone(), new_blocker.clone()]))
        .expect("the retry must not repeat completed retirement because an ordinary notification failed");
    assert_eq!(
        retirement_test_events(&old.events),
        vec![Loaded, Enabled, Disabled, Unloaded, Unloaded]
    );
    assert_eq!(
        retirement_test_events(&blocker.events),
        vec![Loaded, Enabled, Disabled, Disabled, Unloaded]
    );
    assert_eq!(
        retirement_test_events(&replacement.events),
        vec![Loaded, Enabled, HotReloaded]
    );
    assert_eq!(
        retirement_test_events(&new_blocker.events),
        vec![Loaded, Enabled, HotReloaded]
    );
}
