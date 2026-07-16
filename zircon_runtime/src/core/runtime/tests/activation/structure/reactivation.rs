use super::fixture::{
    activation_source, batch_activation_source, reactivation_tests_source, service_entry_source,
    service_lifecycle_source,
};

#[test]
fn reactivation_lifecycle_is_complete_and_folder_backed() {
    let activation = activation_source();
    let batch = batch_activation_source();
    let service_lifecycle = service_lifecycle_source();
    let service_entry = service_entry_source();
    let tests = reactivation_tests_source();

    assert_contains_all(
        "single activation orchestration",
        activation,
        &[
            "mod service_lifecycle;",
            "previous_lifecycle",
            "entry.service_names.clone()",
            "prepare_module_services_for_reactivation",
            "rollback_module_services_after_failed_reactivation",
            "self.reset_initializing_module(module_name, previous_lifecycle)",
        ],
    );
    assert_contains_all(
        "batch activation orchestration",
        batch,
        &[
            "previous_lifecycle: LifecycleState",
            "service_names: Box<[RegistryName]>",
            "prepare_batch_reactivation_services",
            "reset_batch_services",
            "entry.lifecycle = pending_module.previous_lifecycle",
        ],
    );
    assert_contains_all(
        "reactivation service lifecycle owner",
        service_lifecycle,
        &[
            "pub(super) fn validate_reactivation_services",
            "pub(super) fn prepare_reactivation_services",
            "pub(super) fn rollback_reactivation_services",
            "pub(super) fn prepare_module_services_for_reactivation",
            "pub(super) fn rollback_module_services_after_failed_reactivation",
            "entry.lifecycle != LifecycleState::Unloaded",
            "entry.prepare_for_reactivation()",
            "entry.reset_after_failed_reactivation()",
        ],
    );
    assert_contains_all(
        "service slot transitions",
        service_entry,
        &[
            "pub(crate) fn prepare_for_reactivation",
            "pub(crate) fn reset_after_failed_reactivation",
            "self.generation = next_service_generation(self.generation)",
            "self.lifecycle = LifecycleState::Unloaded",
        ],
    );
    assert_contains_all(
        "reactivation behavior coverage",
        tests,
        &[
            "fn single_module_reactivation_restores_immediate_and_lazy_service_slots",
            "fn batch_module_reactivation_restores_immediate_and_lazy_service_slots",
            "fn failed_reactivation_restores_unloaded_slots_and_invalidates_discarded_instance",
            "first_immediate_identity.generation() + 2",
            "first_lazy_identity.generation() + 1",
        ],
    );

    for (path, source, budget) in [
        ("handle/activation.rs", activation, 260),
        ("handle/activation/batch.rs", batch, 280),
        (
            "handle/activation/service_lifecycle.rs",
            service_lifecycle,
            180,
        ),
        ("tests/activation/behavior/reactivation.rs", tests, 400),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below {budget} lines; got {line_count}"
        );
    }

    for forbidden in [
        "resolve_manager::<TestManager>",
        "ensure_service_resolution_available",
        "compat",
        "shim",
    ] {
        assert!(
            !service_lifecycle.contains(forbidden),
            "reactivation lifecycle owner should not contain `{forbidden}`"
        );
    }
}

fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    let missing: Vec<&str> = anchors
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(missing.is_empty(), "{label} missing anchors: {missing:?}");
}
