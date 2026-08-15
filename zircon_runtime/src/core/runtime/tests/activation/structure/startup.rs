use super::fixture::{activation_source, activation_tests_source, startup_source};

#[test]
fn startup_resolution_keeps_exact_count_fast_paths() {
    let activation_source = activation_source();
    let startup_source = startup_source();
    let activation_tests_source = activation_tests_source();

    assert!(
        activation_source.contains("let module_services = graph.module_services(module_name)?;")
    );
    assert!(
        activation_source.contains("let (previous_lifecycle, service_names, startup_services) = {")
    );
    assert!(activation_source.contains("module_services.startup_service_names().clone()"));
    assert!(
        activation_source.contains("self.resolve_startup_services(startup_services.as_ref())?;")
    );
    assert!(activation_source.contains("mod startup;"));
    assert!(!activation_source.contains("fn resolve_startup_services("));
    assert!(startup_source.contains("fn resolve_startup_services("));
    assert!(startup_source.contains("if let [service] = startup_services"));
    assert!(startup_source.contains("self.resolve_registered_service(service, None)?"));
    assert!(startup_source.contains("if let [first_service, second_service] = startup_services"));
    assert!(startup_source.contains("self.resolve_registered_service(first_service, None)?"));
    assert!(startup_source.contains("self.resolve_registered_service(second_service, None)?"));
    assert!(startup_source
        .contains("if let [first_service, second_service, third_service] = startup_services"));
    assert!(startup_source.contains("self.resolve_registered_service(third_service, None)?"));
    assert!(startup_source.contains(
        "if let [first_service, second_service, third_service, fourth_service] = startup_services"
    ));
    assert!(startup_source.contains("self.resolve_registered_service(fourth_service, None)?"));
    assert!(startup_source.contains("fifth_service"));
    assert!(startup_source.contains("self.resolve_registered_service(fifth_service, None)?"));
    assert!(activation_tests_source.contains(
        "fn activate_exact_three_immediate_services_initializes_each_cached_startup_entry_once()"
    ));
    assert!(activation_tests_source.contains(
        "fn activate_exact_four_immediate_services_initializes_each_cached_startup_entry_once()"
    ));
    assert!(activation_tests_source.contains(
        "fn activate_exact_five_immediate_services_initializes_each_cached_startup_entry_once()"
    ));

    let graph_lookup_index = activation_source
        .find("let module_services = graph.module_services(module_name)?;")
        .expect("activation should read startup services from the frozen module graph");
    let startup_clone_index = activation_source
        .find("module_services.startup_service_names().clone()")
        .expect(
            "activation should clone graph-owned startup services before releasing the module lock",
        );
    let startup_resolve_index = activation_source
        .find("self.resolve_startup_services(startup_services.as_ref())?;")
        .expect("activation should route non-empty startup lists through the typed helper");
    assert!(graph_lookup_index < startup_clone_index);
    assert!(startup_clone_index < startup_resolve_index);
    assert!(
        !activation_source.contains("entry.startup_service_names"),
        "activation must not derive startup ordering from mutable module entries after graph freeze"
    );

    let startup_helper_index = startup_source
        .find("fn resolve_startup_services(")
        .expect("activation should keep startup resolution helper private to the handle");
    let single_startup_index = startup_source[startup_helper_index..]
        .find("if let [service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("single-startup service should bypass the multi-service loop");
    let two_startup_index = startup_source[startup_helper_index..]
        .find("if let [first_service, second_service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("two-startup services should bypass the multi-service loop");
    let first_two_startup_resolve_index = startup_source[startup_helper_index..]
        .find("self.resolve_registered_service(first_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("two-startup services should resolve the first key directly");
    let second_two_startup_resolve_index = startup_source[startup_helper_index..]
        .find("self.resolve_registered_service(second_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("two-startup services should resolve the second key directly");
    let three_startup_index = startup_source[startup_helper_index..]
        .find("if let [first_service, second_service, third_service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("three-startup services should bypass the multi-service loop");
    let third_startup_resolve_index = startup_source[startup_helper_index..]
        .find("self.resolve_registered_service(third_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("three-startup services should resolve the third key directly");
    let four_startup_index = startup_source[startup_helper_index..]
        .find("if let [first_service, second_service, third_service, fourth_service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("four-startup services should bypass the multi-service loop");
    let fourth_startup_resolve_index = startup_source[startup_helper_index..]
        .find("self.resolve_registered_service(fourth_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("four-startup services should resolve the fourth key directly");
    let five_startup_index = startup_source[startup_helper_index..]
        .find("fifth_service")
        .map(|offset| startup_helper_index + offset)
        .expect("five-startup services should bypass the multi-service loop");
    let fifth_startup_resolve_index = startup_source[startup_helper_index..]
        .find("self.resolve_registered_service(fifth_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("five-startup services should resolve the fifth key directly");
    let startup_loop_index = startup_source[startup_helper_index..]
        .find("for service in startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("six-or-more startup service activation should retain the cached service loop");

    assert!(single_startup_index < startup_loop_index);
    assert!(single_startup_index < two_startup_index);
    assert!(two_startup_index < first_two_startup_resolve_index);
    assert!(first_two_startup_resolve_index < second_two_startup_resolve_index);
    assert!(second_two_startup_resolve_index < three_startup_index);
    assert!(three_startup_index < third_startup_resolve_index);
    assert!(third_startup_resolve_index < four_startup_index);
    assert!(four_startup_index < fourth_startup_resolve_index);
    assert!(fourth_startup_resolve_index < five_startup_index);
    assert!(five_startup_index < fifth_startup_resolve_index);
    assert!(fifth_startup_resolve_index < startup_loop_index);
    assert!(activation_source.contains("fn finish_module_activation("));
}
