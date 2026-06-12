use super::fixture::{activation_source, activation_tests_source, blocked_unload_source};

#[test]
fn blocked_unload_fallback_keeps_indexed_six_or_more_scan() {
    let activation_source = activation_source();
    let blocked_unload_source = blocked_unload_source();
    let activation_tests_source = activation_tests_source();

    assert!(activation_source.contains("mod blocked_unload;"));
    assert!(activation_source.contains("use self::blocked_unload::first_blocked_unload;"));
    assert!(activation_source.contains("first_blocked_unload(&services, unload_order)"));
    assert!(!activation_source.contains("fn first_blocked_unload("));
    assert!(!activation_source.contains("fn first_blocked_single_service_unload("));
    assert!(!activation_source.contains("fn record_blocked_dependent("));

    let single_service_precheck_index = blocked_unload_source
        .find("return first_blocked_single_service_unload(services, service_name);")
        .expect("single-service unload should bypass the multi-service unload index");
    let two_service_precheck_index = blocked_unload_source
        .find("return first_blocked_two_service_unload(services, first_service_name, second_service_name);")
        .expect("two-service unload should bypass the multi-service unload index");
    let three_service_precheck_index = blocked_unload_source
        .find("return first_blocked_three_service_unload(")
        .expect("three-service unload should bypass the multi-service unload index");
    let four_service_precheck_index = blocked_unload_source
        .find("return first_blocked_four_service_unload(")
        .expect("four-service unload should bypass the multi-service unload index");
    let five_service_precheck_index = blocked_unload_source
        .find("return first_blocked_five_service_unload(")
        .expect("five-service unload should bypass the multi-service unload index");
    let unload_index_map_index = blocked_unload_source
        .find("HashMap::with_capacity(unload_order.len())")
        .expect("six-or-more blocked unloads should still pre-size the unload index");

    assert!(single_service_precheck_index < unload_index_map_index);
    assert!(single_service_precheck_index < two_service_precheck_index);
    assert!(two_service_precheck_index < unload_index_map_index);
    assert!(two_service_precheck_index < three_service_precheck_index);
    assert!(three_service_precheck_index < four_service_precheck_index);
    assert!(four_service_precheck_index < unload_index_map_index);
    assert!(four_service_precheck_index < five_service_precheck_index);
    assert!(five_service_precheck_index < unload_index_map_index);
    assert!(activation_tests_source
        .contains("fn deactivate_exact_four_services_reports_first_blocked_without_index_map()"));
    assert!(activation_tests_source
        .contains("fn deactivate_exact_five_services_reports_first_blocked_without_index_map()"));

    assert!(blocked_unload_source.contains("const BLOCKED_DEPENDENT_INITIAL_CAPACITY: usize = 1;"));
    assert!(blocked_unload_source.contains("let mut unload_indices: HashMap<&RegistryName, usize>"));
    assert!(blocked_unload_source.contains("HashMap::with_capacity(unload_order.len())"));
    assert!(blocked_unload_source.contains("unload_indices.insert(service_name, index)"));
    assert!(blocked_unload_source.contains("let mut blocked_index = None"));
    assert!(blocked_unload_source.contains("let mut blocked_dependents = None"));
    assert!(blocked_unload_source.contains("fn record_blocked_dependent("));
    assert!(blocked_unload_source.contains("for (dependent_name, entry) in services.iter()"));
    assert!(blocked_unload_source.contains("unload_indices.contains_key(dependent_name)"));
    assert!(blocked_unload_source.contains("for dependency in entry.dependencies.iter()"));
    assert!(blocked_unload_source.contains("unload_indices.get(dependency).copied()"));
    assert!(blocked_unload_source.contains("record_blocked_dependent("));
    assert!(blocked_unload_source.contains("match (blocked_index, blocked_dependents)"));
    assert!(blocked_unload_source.contains("Some((unload_order[index].to_string(), dependents))"));
    assert!(blocked_unload_source
        .contains("get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))"));
    assert!(blocked_unload_source.contains("let Some(dependents) = blocked_dependents else"));
    assert!(blocked_unload_source.contains("Some((service_name.to_string(), dependents))"));
    assert!(blocked_unload_source.contains("dependents.clear()"));

    assert!(!blocked_unload_source.contains("HashSet<RegistryName>"));
    assert!(!blocked_unload_source.contains("HashSet<String>"));
    assert!(!blocked_unload_source.contains("unloading.contains(dependent_name)"));
    assert!(!blocked_unload_source.contains("let mut dependents_by_service = vec![Vec::new();"));
    assert!(!blocked_unload_source.contains("let mut blocked_dependents = Vec::new()"));
    assert!(!blocked_unload_source.contains(".zip(dependents_by_service)"));
    assert!(!blocked_unload_source.contains("fn running_dependents("));
    assert!(!activation_source.contains("owner_module == module_name"));
    assert!(!activation_source.contains(".map(|(name, _)| name.clone())"));
    assert!(!activation_source.contains("(entry.kind, name.clone())"));
    assert!(!activation_source.contains("names.into_iter().map(|(_, name)| name)"));
    assert!(!blocked_unload_source.contains("let service_name = service_name.as_str();"));
    assert!(!blocked_unload_source
        .contains("blocked_dependents.map(|dependents| (service_name.to_string(), dependents))"));
    assert!(!blocked_unload_source.contains("services.get_mut(service_name.as_str())"));
    assert!(!activation_source.contains("self.resolve_named_service(service.as_str(), None)?"));
    assert!(!activation_source.contains("entry.name"));
    assert!(!activation_source.contains("entry.name.to_string()"));
}
