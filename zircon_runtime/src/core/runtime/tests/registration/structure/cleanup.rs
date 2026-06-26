use super::registration_sources;

#[test]
fn registration_source_preserves_legacy_cleanup_boundaries() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();

    assert!(!sources
        .runtime_state
        .contains("HashMap<String, ServiceEntry>"));
    assert!(!sources.service_entry.contains("name: RegistryName"));
    assert!(!sources.service_entry.contains("owner_module: String"));
    assert!(!sources.service_entry.contains("kind: ServiceKind"));
    assert!(!registration.contains("existing_services"));
    assert!(!registration.contains("services.contains_key(name.as_str())"));
    assert!(!registration.contains("pending_keys.contains(name.as_str())"));
    assert!(!registration.contains("HashSet::new()"));
    assert!(!registration.contains(".map(|dependency| dependency.name.clone())"));
    assert!(!registration.contains(".map(|(name, _)| name.clone())"));
    assert!(!registration.contains(".collect::<Vec<_>>()"));
    assert!(!registration.contains("fn startup_service_names("));
    assert!(!registration.contains("fn shutdown_service_names("));
}
