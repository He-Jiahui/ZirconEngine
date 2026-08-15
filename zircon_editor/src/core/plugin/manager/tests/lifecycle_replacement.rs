//! Recovery and replacement tests for manager-owned lifecycle mutations.

use std::sync::{Arc, Mutex};

use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleStage;
use crate::core::plugin::{EditorPlugin, EditorPluginDescriptor};

use super::super::{
    EditorPluginCatalog, EditorPluginDiscoveryError, EditorPluginLoadingPhase, EditorPluginManager,
    EditorPluginState, EditorPluginTransitionError,
};
use super::{FailOnceLifecyclePlugin, LifecyclePhasePlugin};
#[test]
fn disabling_retries_a_failed_disabled_lifecycle_callback() {
    let package_id = "plugin.retry.disabled";
    let plugin = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, package_id, package_id),
        fail_stage: EditorPluginLifecycleStage::Disabled,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let catalog = EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(package_id, package_id),
    )]);
    let manager = EditorPluginManager::new(catalog)
        .expect("the disabled retry fixture catalog is admissible");

    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the fixture should activate before it is disabled");
    let failed = manager
        .set_enabled(package_id, false)
        .expect("the manager should retain the failed disabled lifecycle result");
    assert_eq!(
        failed.entry(package_id).map(|entry| entry.state()),
        Some(EditorPluginState::Faulted)
    );
    let transition_error = manager
        .transition_state(package_id, EditorPluginState::Validated)
        .expect_err("the public transition surface must not bypass failed cleanup");
    assert!(matches!(
        transition_error,
        EditorPluginTransitionError::DisabledLifecycleRetryRequired { .. }
    ));
    assert!(matches!(
        manager.validate_enablement(package_id, true),
        Err(EditorPluginTransitionError::DisabledLifecycleRetryRequired { .. })
    ));
    assert!(matches!(
        manager.set_enabled(package_id, true),
        Err(EditorPluginTransitionError::DisabledLifecycleRetryRequired { .. })
    ));
    manager
        .validate_enablement(package_id, false)
        .expect("the manager facade should permit a disabled lifecycle retry");

    let recovered = manager
        .set_enabled(package_id, false)
        .expect("the manager should retry a failed disabled lifecycle callback");
    assert_eq!(
        recovered.entry(package_id).map(|entry| entry.state()),
        Some(EditorPluginState::Disabled)
    );
    assert_eq!(
        *plugin
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Disabled,
        ]
    );
}

#[test]
fn replacing_a_package_retries_failed_disabled_cleanup_before_activation() {
    let package_id = "plugin.retry.disabled-replacement";
    let plugin = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, package_id, package_id),
        fail_stage: EditorPluginLifecycleStage::Disabled,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(package_id, package_id),
    )]))
    .expect("the disabled retry fixture catalog is admissible");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the fixture should activate before it is disabled");
    manager
        .set_enabled(package_id, false)
        .expect("the fixture should retain the failed disabled lifecycle result");

    let replacement = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "Replacement", "plugin_replacement"),
        events: Mutex::default(),
    });
    manager
        .publish_catalog(EditorPluginCatalog::from_plugins([(
            Arc::clone(&replacement) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(package_id, "Replacement"),
        )]))
        .expect("a replacement must retry the failed disabled cleanup before activation");
    assert_eq!(
        manager
            .state_snapshot()
            .entry(package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        *plugin
            .events
            .lock()
            .expect("lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ]
    );
    assert_eq!(
        *replacement
            .events
            .lock()
            .expect("replacement lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::HotReloaded,
        ]
    );
}

#[test]
fn replacing_an_active_package_retires_the_old_instance_before_hot_reload() {
    let package_id = "plugin.lifecycle.replaced";
    let first_plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "First", "plugin_first"),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([(
        Arc::clone(&first_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(package_id, "First"),
    )]))
    .expect("the first lifecycle fixture should be admitted");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the first fixture should activate");

    let replacement_plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "Replacement", "plugin_replacement"),
        events: Mutex::default(),
    });
    manager
        .publish_catalog(EditorPluginCatalog::from_plugins([(
            Arc::clone(&replacement_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(package_id, "Replacement"),
        )]))
        .expect("replacing an active package should initialize the new plugin instance");

    assert_eq!(
        manager
            .state_snapshot()
            .entry(package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        *first_plugin
            .events
            .lock()
            .expect("first lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ]
    );
    assert_eq!(
        *replacement_plugin
            .events
            .lock()
            .expect("replacement lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::HotReloaded,
        ]
    );
}

#[test]
fn replacement_retries_failed_unload_before_activating_candidate() {
    let package_id = "plugin.lifecycle.unload-failure";
    let first_plugin = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "First", "plugin_first"),
        fail_stage: EditorPluginLifecycleStage::Unloaded,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([(
        Arc::clone(&first_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(package_id, "First"),
    )]))
    .expect("the first lifecycle fixture should be admitted");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the first fixture should activate");
    let replacement_plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "Replacement", "plugin_replacement"),
        events: Mutex::default(),
    });

    let error = manager
        .publish_catalog(EditorPluginCatalog::from_plugins([(
            Arc::clone(&replacement_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(package_id, "Replacement"),
        )]))
        .expect_err("a failed old-instance unload must reject the replacement catalog");

    assert!(matches!(
        error,
        EditorPluginDiscoveryError::LifecycleCleanupFailed {
            package_id: actual,
            stage: EditorPluginLifecycleStage::Unloaded,
        } if actual == package_id
    ));
    assert_eq!(
        manager
            .state_snapshot()
            .entry(package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Faulted)
    );
    assert_eq!(
        *first_plugin
            .events
            .lock()
            .expect("first lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ]
    );
    assert!(replacement_plugin
        .events
        .lock()
        .expect("replacement lifecycle event lock should not be poisoned")
        .is_empty());

    let recovered_plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "Recovered", "plugin_recovered"),
        events: Mutex::default(),
    });
    manager
        .publish_catalog(EditorPluginCatalog::from_plugins([(
            Arc::clone(&recovered_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(package_id, "Recovered"),
        )]))
        .expect("the next replacement must retry the unfinished old-instance unload first");

    assert_eq!(
        manager
            .state_snapshot()
            .entry(package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        *first_plugin
            .events
            .lock()
            .expect("first lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ]
    );
    assert_eq!(
        *recovered_plugin
            .events
            .lock()
            .expect("recovered lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::HotReloaded,
        ]
    );
}

#[test]
fn multi_package_replacement_recovers_completed_cleanup_before_later_failure() {
    let first_package_id = "plugin.lifecycle.recovery.alpha";
    let second_package_id = "plugin.lifecycle.recovery.beta";
    let first_plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(first_package_id, "First", "first"),
        events: Mutex::default(),
    });
    let second_plugin = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(second_package_id, "Second", "second"),
        fail_stage: EditorPluginLifecycleStage::Disabled,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([
        (
            Arc::clone(&first_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(first_package_id, "First"),
        ),
        (
            Arc::clone(&second_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(second_package_id, "Second"),
        ),
    ]))
    .expect("the replacement fixture catalog is admissible");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("both old instances should activate");

    let replacement_first = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(first_package_id, "Replacement First", "first_v2"),
        events: Mutex::default(),
    });
    let replacement_second = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(
            second_package_id,
            "Replacement Second",
            "second_v2",
        ),
        events: Mutex::default(),
    });

    let error = manager
        .publish_catalog(EditorPluginCatalog::from_plugins([
            (
                Arc::clone(&replacement_first) as Arc<dyn EditorPlugin + Send + Sync>,
                PluginPackageManifest::new(first_package_id, "Replacement First"),
            ),
            (
                Arc::clone(&replacement_second) as Arc<dyn EditorPlugin + Send + Sync>,
                PluginPackageManifest::new(second_package_id, "Replacement Second"),
            ),
        ]))
        .expect_err("the second old instance should fail its first disabled attempt");
    assert!(matches!(
        error,
        EditorPluginDiscoveryError::LifecycleCleanupFailed {
            package_id,
            stage: EditorPluginLifecycleStage::Disabled,
        } if package_id == second_package_id
    ));
    let interrupted = manager.state_snapshot();
    assert_eq!(
        interrupted
            .entry(first_package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Revoking)
    );
    assert_eq!(
        interrupted
            .entry(second_package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Faulted)
    );

    manager
        .publish_catalog(EditorPluginCatalog::from_plugins([
            (
                Arc::clone(&replacement_first) as Arc<dyn EditorPlugin + Send + Sync>,
                PluginPackageManifest::new(first_package_id, "Replacement First"),
            ),
            (
                Arc::clone(&replacement_second) as Arc<dyn EditorPlugin + Send + Sync>,
                PluginPackageManifest::new(second_package_id, "Replacement Second"),
            ),
        ]))
        .expect("a retry must complete every old cleanup before activating both candidates");

    let recovered = manager.state_snapshot();
    for package_id in [first_package_id, second_package_id] {
        assert_eq!(
            recovered.entry(package_id).map(|entry| entry.state()),
            Some(EditorPluginState::Active)
        );
    }
    assert_eq!(
        *first_plugin
            .events
            .lock()
            .expect("first lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ]
    );
    assert_eq!(
        *second_plugin
            .events
            .lock()
            .expect("second lifecycle event fixture lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ]
    );
    for replacement in [&replacement_first, &replacement_second] {
        assert_eq!(
            *replacement
                .events
                .lock()
                .expect("replacement lifecycle event lock should not be poisoned"),
            vec![
                EditorPluginLifecycleStage::Loaded,
                EditorPluginLifecycleStage::Enabled,
                EditorPluginLifecycleStage::HotReloaded,
            ]
        );
    }
}

#[test]
fn replacing_a_faulted_package_dispatches_lifecycle_for_the_new_plugin_instance() {
    let package_id = "plugin.lifecycle.faulted-replaced";
    let first_plugin = Arc::new(FailOnceLifecyclePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "First", "plugin_first"),
        fail_stage: EditorPluginLifecycleStage::Enabled,
        should_fail: Mutex::new(true),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([(
        Arc::clone(&first_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(package_id, "First"),
    )]))
    .expect("the first lifecycle fixture should be admitted");
    let failed = manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the first fixture should publish its lifecycle failure");
    assert_eq!(
        failed.entry(package_id).map(|entry| entry.state()),
        Some(EditorPluginState::Faulted)
    );

    let replacement_plugin = Arc::new(LifecyclePhasePlugin {
        descriptor: EditorPluginDescriptor::new(package_id, "Replacement", "plugin_replacement"),
        events: Mutex::default(),
    });
    manager
        .publish_catalog(EditorPluginCatalog::from_plugins([(
            Arc::clone(&replacement_plugin) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(package_id, "Replacement"),
        )]))
        .expect("a healthy replacement should not retain the prior instance fault");

    assert_eq!(
        manager
            .state_snapshot()
            .entry(package_id)
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    assert_eq!(
        *replacement_plugin
            .events
            .lock()
            .expect("replacement lifecycle event lock should not be poisoned"),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
        ]
    );
}
