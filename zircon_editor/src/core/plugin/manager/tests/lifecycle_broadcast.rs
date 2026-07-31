//! Regression tests for one-publish external lifecycle broadcasts.

use std::sync::{Arc, Mutex};

use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::plugin::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleStage};
use crate::core::plugin::{EditorPlugin, EditorPluginDescriptor};

use super::super::{
    EditorPluginCatalog, EditorPluginLoadingPhase, EditorPluginManager, EditorPluginState,
    EditorPluginTransitionError,
};
use super::{FailOnceLifecyclePlugin, LifecyclePhasePlugin};

#[test]
fn external_lifecycle_broadcast_rejects_manager_owned_activation_stages() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.broadcast.reserved",
            "Reserved",
            "reserved",
        )],
        Vec::<PluginPackageManifest>::new(),
    ))
    .expect("the reserved-stage fixture should be admitted");

    for stage in [
        EditorPluginLifecycleStage::Loaded,
        EditorPluginLifecycleStage::Enabled,
        EditorPluginLifecycleStage::Disabled,
    ] {
        let error = manager
            .dispatch_lifecycle_event_to_active(EditorPluginLifecycleEvent::new(stage.clone()))
            .expect_err("a broadcast must not bypass manager-owned activation scheduling");

        assert!(matches!(
            error,
            EditorPluginTransitionError::ManagedLifecycleBroadcastReserved {
                stage: actual
            } if actual == stage
        ));
    }
}

#[test]
fn external_lifecycle_broadcast_updates_active_plugins_in_one_generation() {
    let first = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.broadcast.first", "First", "first"),
        events: Mutex::default(),
    });
    let second = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.broadcast.second", "Second", "second"),
        events: Mutex::default(),
    });
    let disabled = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(
            "plugin.broadcast.disabled",
            "Disabled",
            "disabled",
        ),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([
        (
            Arc::clone(&first) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.broadcast.first", "First"),
        ),
        (
            Arc::clone(&second) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.broadcast.second", "Second"),
        ),
        (
            Arc::clone(&disabled) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.broadcast.disabled", "Disabled"),
        ),
    ]))
    .expect("the broadcast fixtures should be admitted");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate every fixture");
    manager
        .set_enabled("plugin.broadcast.disabled", false)
        .expect("the disabled fixture should leave the active set");
    let before = manager.state_snapshot();

    let report = manager
        .dispatch_lifecycle_event_to_active(
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::AssetChanged)
                .with_subject("assets/example.material"),
        )
        .expect("the active plugins should receive the external lifecycle event");
    let after = manager.state_snapshot();

    assert_eq!(after.generation(), before.generation() + 1);
    assert_eq!(after.catalog_generation(), before.catalog_generation() + 1);
    assert_eq!(report.records().len(), 2);
    assert!(report.records().iter().all(|record| {
        record.event().stage() == &EditorPluginLifecycleStage::AssetChanged
            && record.event().subject() == Some("assets/example.material")
    }));
    assert_eq!(
        after
            .entry("plugin.broadcast.first")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        after
            .entry("plugin.broadcast.second")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        after
            .entry("plugin.broadcast.disabled")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Disabled)
    );
    for plugin in [&first, &second] {
        assert_eq!(
            *plugin
                .events
                .lock()
                .expect("lifecycle event fixture lock should not be poisoned"),
            vec![
                EditorPluginLifecycleStage::Loaded,
                EditorPluginLifecycleStage::Enabled,
                EditorPluginLifecycleStage::AssetChanged,
            ]
        );
    }
    assert_eq!(
        *disabled
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
        ]
    );
}

#[test]
fn external_lifecycle_broadcast_faults_only_the_callback_that_fails() {
    let faulted = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.broadcast.faulted", "Faulted", "faulted"),
        fail_stage: EditorPluginLifecycleStage::SceneChanged,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let healthy = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new("plugin.broadcast.healthy", "Healthy", "healthy"),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([
        (
            Arc::clone(&faulted) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.broadcast.faulted", "Faulted"),
        ),
        (
            Arc::clone(&healthy) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new("plugin.broadcast.healthy", "Healthy"),
        ),
    ]))
    .expect("the broadcast fixtures should be admitted");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate every fixture");

    let report = manager
        .dispatch_lifecycle_event_to_active(EditorPluginLifecycleEvent::new(
            EditorPluginLifecycleStage::SceneChanged,
        ))
        .expect("one callback failure should be reported through the manager snapshot");
    let snapshot = manager.state_snapshot();

    assert!(!report.is_success());
    assert_eq!(report.records().len(), 2);
    assert_eq!(
        snapshot
            .entry("plugin.broadcast.faulted")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Faulted)
    );
    assert_eq!(
        snapshot
            .entry("plugin.broadcast.healthy")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        *healthy
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::SceneChanged,
        ]
    );

    let next_report = manager
        .dispatch_lifecycle_event_to_active(EditorPluginLifecycleEvent::new(
            EditorPluginLifecycleStage::UiMessage,
        ))
        .expect("a later broadcast should skip the faulted plugin");

    assert!(next_report.is_success());
    assert_eq!(next_report.records().len(), 1);
    assert_eq!(
        next_report.records()[0].package_id(),
        "plugin.broadcast.healthy"
    );
    assert_eq!(
        *faulted
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::SceneChanged,
        ]
    );
    assert_eq!(
        *healthy
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::SceneChanged,
            EditorPluginLifecycleStage::UiMessage,
        ]
    );
}
