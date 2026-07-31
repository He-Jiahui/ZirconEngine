//! State-machine tests for manager-owned lifecycle transitions.

use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::plugin::EditorPluginDescriptor;

use super::super::{
    EditorPluginCatalog, EditorPluginLoadingPhase, EditorPluginManager, EditorPluginState,
    EditorPluginTransitionError,
};

#[test]
fn state_machine_accepts_only_lifecycle_edges() {
    use EditorPluginState::{Active, Disabled, Discovered, Faulted, Loading, Revoking, Validated};

    assert!(Discovered.can_transition_to(Validated));
    assert!(Discovered.can_transition_to(Disabled));
    assert!(Validated.can_transition_to(Loading));
    assert!(Loading.can_transition_to(Active));
    assert!(Active.can_transition_to(Revoking));
    assert!(Revoking.can_transition_to(Disabled));
    assert!(Disabled.can_transition_to(Validated));
    assert!(Faulted.can_transition_to(Validated));
    assert!(Active.can_transition_to(Faulted));
    assert!(!Active.can_transition_to(Disabled));
    assert!(!Faulted.can_transition_to(Active));
}

#[test]
fn manager_reserves_lifecycle_states_for_its_phase_and_enablement_paths() {
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
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate the fixture plugin");

    let error = manager
        .transition_state("plugin.sample", EditorPluginState::Revoking)
        .expect_err("the generic transition API must not bypass lifecycle callbacks");
    assert!(matches!(
        error,
        EditorPluginTransitionError::ManagedLifecycleTransitionRequired { .. }
    ));
    assert_eq!(
        manager
            .state_snapshot()
            .entry("plugin.sample")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
}
