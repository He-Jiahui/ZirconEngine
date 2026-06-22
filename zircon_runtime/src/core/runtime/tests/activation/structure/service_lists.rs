use super::fixture::{
    activation_source, blocked_unload_source, module_entry_source, registration_source,
    unload_mutation_source,
};

#[test]
fn activation_uses_cached_registry_name_service_lists() {
    let activation_source = activation_source();
    let blocked_unload_source = blocked_unload_source();
    let module_entry_source = module_entry_source();
    let registration_source = registration_source();
    let unload_mutation_source = unload_mutation_source();

    assert!(blocked_unload_source.contains("unload_order: &[RegistryName]"));
    assert!(unload_mutation_source.contains("unload_order: &[RegistryName]"));
    assert!(module_entry_source.contains("service_names: Arc<[RegistryName]>"));
    assert!(module_entry_source.contains("startup_service_names: Arc<[RegistryName]>"));
    assert!(module_entry_source.contains("shutdown_service_names: Arc<[RegistryName]>"));
    assert!(registration_source.contains("module_service_lists(&pending_services"));
    assert!(registration_source.contains("fn prepare_four_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_five_descriptor_service_entries("));
    assert!(registration_source.contains("struct ModuleServiceLists"));
    assert!(registration_source.contains("service_names: module_service_lists.service_names,"));
    assert!(registration_source
        .contains("startup_service_names: module_service_lists.startup_service_names,"));
    assert!(registration_source
        .contains("shutdown_service_names: module_service_lists.shutdown_service_names,"));
    assert!(registration_source.contains("ServiceKind::Plugin"));
    assert!(registration_source.contains("ServiceKind::Manager"));
    assert!(registration_source.contains("ServiceKind::Driver"));

    assert!(!activation_source.contains("StartupMode"));
    assert!(!activation_source.contains("let immediate_services: Vec<RegistryName>"));
    assert!(!activation_source.contains(".filter_map(|name|"));
    assert!(!activation_source.contains("sort_by_key"));
    assert!(!activation_source.contains("fn service_start_order"));
    assert!(!activation_source.contains("fn service_stop_order"));
}
