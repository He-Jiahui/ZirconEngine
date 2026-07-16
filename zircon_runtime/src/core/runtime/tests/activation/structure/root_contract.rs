use super::fixture::{
    activation_behavior_mod_source, activation_mod_source, activation_source,
    activation_tests_source, blocked_dependencies_root_source, deactivation_behavior_mod_source,
};

#[test]
fn activation_structure_roots_stay_navigational() {
    let activation_source = activation_source();
    let blocked_dependencies_root_source = blocked_dependencies_root_source();
    let activation_mod_source = activation_mod_source();
    let activation_behavior_mod_source = activation_behavior_mod_source();
    let deactivation_behavior_mod_source = deactivation_behavior_mod_source();
    let activation_tests_source = activation_tests_source();

    assert!(activation_mod_source.contains("mod behavior;"));
    assert!(activation_mod_source.contains("mod structure;"));
    assert!(!activation_mod_source.contains("#[test]"));
    assert!(activation_behavior_mod_source.contains("mod activation;"));
    assert!(activation_behavior_mod_source.contains("mod deactivation;"));
    assert!(activation_behavior_mod_source.contains("mod reactivation;"));
    assert!(!activation_behavior_mod_source.contains("#[test]"));
    assert!(deactivation_behavior_mod_source.contains("mod blocked;"));
    assert!(deactivation_behavior_mod_source.contains("mod clean;"));
    assert!(!deactivation_behavior_mod_source.contains("#[test]"));
    assert!(!deactivation_behavior_mod_source.contains("use "));
    assert!(!activation_tests_source.contains("include_str!(\"../../handle/activation.rs\")"));

    assert!(activation_source.contains("mod blocked_dependencies;"));
    assert!(activation_source.contains("mod blocked_unload;"));
    assert!(activation_source.contains("mod service_lifecycle;"));
    assert!(activation_source.contains("mod startup;"));
    assert!(activation_source.contains("mod unload_mutation;"));
    assert!(
        activation_source.contains("use self::blocked_unload::first_blocked_unload;"),
        "activation should import blocked-unload precheck from its child owner"
    );
    assert!(
        activation_source.contains("use self::unload_mutation::unload_services;"),
        "activation should import successful unload mutation from its child owner"
    );
    assert!(
        !activation_source.contains("fn first_blocked_unload("),
        "blocked-unload precheck should stay in activation/blocked_unload.rs"
    );
    assert!(
        !activation_source.contains("fn first_blocked_single_service_unload("),
        "blocked-unload helper bodies should stay in activation/blocked_unload.rs"
    );
    assert!(
        !activation_source.contains("fn record_blocked_dependent("),
        "blocked dependent mutation should stay in activation/blocked_unload.rs"
    );
    assert!(
        !activation_source.contains("fn resolve_startup_services("),
        "startup resolution should stay in activation/startup.rs"
    );
    assert!(
        !activation_source.contains("fn unload_services("),
        "successful unload mutation should stay in activation/unload_mutation.rs"
    );
    assert!(activation_source.contains("let Some(entry) = modules.get_mut(module_name) else"));
    assert!(activation_source
        .contains("return Err(CoreError::MissingModule(module_name.to_string()));"));
    assert!(!activation_source.contains(".ok_or_else(|| CoreError::MissingModule"));

    assert!(blocked_dependencies_root_source.contains("mod single;"));
    assert!(blocked_dependencies_root_source.contains("mod two_service;"));
    assert!(blocked_dependencies_root_source.contains("mod three_service;"));
    assert!(blocked_dependencies_root_source.contains("mod four_service;"));
    assert!(blocked_dependencies_root_source.contains("mod five_service;"));
    assert!(blocked_dependencies_root_source
        .contains("pub(super) use single::dependency_slice_contains_service;"));
    assert!(blocked_dependencies_root_source.contains("pub(super) use two_service::{"));
    assert!(blocked_dependencies_root_source.contains("pub(super) use three_service::{"));
    assert!(blocked_dependencies_root_source.contains("pub(super) use four_service::{"));
    assert!(blocked_dependencies_root_source.contains("pub(super) use five_service::{"));
    assert!(!blocked_dependencies_root_source.contains("match dependencies"));
    assert!(!blocked_dependencies_root_source.contains("for dependency in dependencies"));
    assert!(!blocked_dependencies_root_source.contains("let mut second_service_blocked"));
}
