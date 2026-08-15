use super::fixture::{activation_source, activation_tests_source, unload_mutation_source};

#[test]
fn successful_unload_mutation_keeps_exact_count_fast_paths() {
    let activation_source = activation_source();
    let unload_mutation_source = unload_mutation_source();
    let activation_tests_source = activation_tests_source();

    let graph_lookup_index = activation_source
        .find("let module_services = graph.module_services(module_name)?;")
        .expect("deactivation should read shutdown services from the frozen module graph");
    let shutdown_clone_index = activation_source
        .find("module_services.shutdown_service_names().clone()")
        .expect("deactivation should clone graph-owned shutdown services before releasing the module lock");
    let service_precheck_index = activation_source
        .find("let blocked_unload = {")
        .expect("deactivation should retain blocked-unload precheck for non-empty modules");
    assert!(graph_lookup_index < shutdown_clone_index);
    assert!(shutdown_clone_index < service_precheck_index);
    assert!(
        !activation_source.contains("entry.shutdown_service_names"),
        "deactivation must not derive shutdown ordering from mutable module entries after graph freeze"
    );
    assert!(activation_source.contains("fn finish_module_deactivation("));
    assert!(activation_source.contains("let unload_order = unload_order.as_ref()"));
    assert!(activation_source.contains("unload_services(&mut services, unload_order);"));
    assert!(activation_source.contains("mod unload_mutation;"));
    assert!(activation_source.contains("use self::unload_mutation::unload_services;"));
    assert!(!activation_source.contains("fn unload_services("));
    assert!(unload_mutation_source.contains("fn unload_services("));
    assert!(unload_mutation_source.contains("fn unload_service("));
    assert!(unload_mutation_source.contains("unload_service(services, service_name);"));
    assert!(unload_mutation_source.contains("services.get_mut(service_name)"));

    let unload_services_index = unload_mutation_source
        .find("fn unload_services(")
        .expect("successful unload should be routed through a typed helper");
    let single_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("if let [service_name] = unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("single-service unload mutation should bypass the multi-service loop");
    let two_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("if let [first_service_name, second_service_name] = unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("two-service unload mutation should bypass the multi-service loop");
    let first_two_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("unload_service(services, first_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("two-service unload mutation should unload the first service directly");
    let second_two_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("unload_service(services, second_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("two-service unload mutation should unload the second service directly");
    let three_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("if let [first_service_name, second_service_name, third_service_name] = unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("three-service unload mutation should bypass the multi-service loop");
    let third_three_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("unload_service(services, third_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("three-service unload mutation should unload the third service directly");
    let four_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("if let [first_service_name, second_service_name, third_service_name, fourth_service_name]")
        .map(|offset| unload_services_index + offset)
        .expect("four-service unload mutation should bypass the multi-service loop");
    let fourth_four_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("unload_service(services, fourth_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("four-service unload mutation should unload the fourth service directly");
    let five_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("fifth_service_name")
        .map(|offset| unload_services_index + offset)
        .expect("five-service unload mutation should bypass the multi-service loop");
    let fifth_five_service_mutation_index = unload_mutation_source[unload_services_index..]
        .find("unload_service(services, fifth_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("five-service unload mutation should unload the fifth service directly");
    let unload_loop_index = unload_mutation_source[unload_services_index..]
        .find("for service_name in unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("six-or-more service unload mutation should retain the cached unload order loop");

    assert!(single_service_mutation_index < unload_loop_index);
    assert!(single_service_mutation_index < two_service_mutation_index);
    assert!(two_service_mutation_index < first_two_service_mutation_index);
    assert!(first_two_service_mutation_index < second_two_service_mutation_index);
    assert!(second_two_service_mutation_index < three_service_mutation_index);
    assert!(three_service_mutation_index < third_three_service_mutation_index);
    assert!(third_three_service_mutation_index < four_service_mutation_index);
    assert!(four_service_mutation_index < fourth_four_service_mutation_index);
    assert!(fourth_four_service_mutation_index < five_service_mutation_index);
    assert!(five_service_mutation_index < fifth_five_service_mutation_index);
    assert!(fifth_five_service_mutation_index < unload_loop_index);
    assert!(activation_tests_source
        .contains("fn deactivate_exact_four_services_unloads_all_cached_entries_directly()"));
    assert!(activation_tests_source
        .contains("fn deactivate_exact_five_services_unloads_all_cached_entries_directly()"));
}
