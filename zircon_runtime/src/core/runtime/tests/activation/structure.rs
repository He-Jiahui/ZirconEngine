#[test]
fn activation_keeps_module_service_lists_as_registry_names() {
    let activation_source = include_str!("../../handle/activation.rs");
    let blocked_dependencies_source =
        include_str!("../../handle/activation/blocked_dependencies.rs");
    let module_entry_source = include_str!("../../state/module_entry.rs");
    let registration_source = [
        include_str!("../../handle/registration/mod.rs"),
        include_str!("../../handle/registration/register_module.rs"),
        include_str!("../../handle/registration/descriptor_entries.rs"),
        include_str!("../../handle/registration/descriptor_entries_five.rs"),
        include_str!("../../handle/registration/descriptor_entries_four.rs"),
        include_str!("../../handle/registration/descriptor_entries_three.rs"),
        include_str!("../../handle/registration/duplicates.rs"),
        include_str!("../../handle/registration/entry.rs"),
        include_str!("../../handle/registration/service_lists.rs"),
        include_str!("../../handle/registration/validation.rs"),
    ]
    .join("\n");
    let activation_mod_source = include_str!("mod.rs");
    let activation_behavior_mod_source = include_str!("behavior.rs");
    let deactivation_behavior_mod_source = include_str!("behavior/deactivation.rs");
    let activation_tests_source = [
        include_str!("behavior/activation.rs"),
        include_str!("behavior/deactivation/blocked.rs"),
        include_str!("behavior/deactivation/clean.rs"),
    ]
    .join("\n");

    assert!(activation_mod_source.contains("mod behavior;"));
    assert!(activation_mod_source.contains("mod structure;"));
    assert!(!activation_mod_source.contains("#[test]"));
    assert!(activation_behavior_mod_source.contains("mod activation;"));
    assert!(activation_behavior_mod_source.contains("mod deactivation;"));
    assert!(!activation_behavior_mod_source.contains("#[test]"));
    assert!(deactivation_behavior_mod_source.contains("mod blocked;"));
    assert!(deactivation_behavior_mod_source.contains("mod clean;"));
    assert!(!deactivation_behavior_mod_source.contains("#[test]"));
    assert!(!deactivation_behavior_mod_source.contains("use "));
    assert!(!activation_tests_source.contains("include_str!(\"../../handle/activation.rs\")"));
    assert!(activation_source.contains("mod blocked_dependencies;"));
    assert!(
        activation_source.contains("use self::blocked_dependencies::{"),
        "activation should import blocked-dependency classifiers from its child owner"
    );
    assert!(activation_source.contains("unload_order: &[RegistryName]"));
    assert!(module_entry_source.contains("service_names: Arc<[RegistryName]>"));
    assert!(module_entry_source.contains("startup_service_names: Arc<[RegistryName]>"));
    assert!(module_entry_source.contains("shutdown_service_names: Arc<[RegistryName]>"));
    assert!(registration_source.contains("let module_service_lists = module_service_lists("));
    assert!(registration_source.contains("fn prepare_four_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_five_descriptor_service_entries("));
    assert!(registration_source.contains("struct ModuleServiceLists"));
    assert!(registration_source.contains("service_names: module_service_lists.service_names,"));
    assert!(registration_source
        .contains("startup_service_names: module_service_lists.startup_service_names,"));
    assert!(registration_source
        .contains("shutdown_service_names: module_service_lists.shutdown_service_names,"));
    assert!(activation_source.contains("let startup_services = {"));
    assert!(
        activation_source.contains("self.resolve_startup_services(startup_services.as_ref())?;")
    );
    assert!(activation_source.contains("fn resolve_startup_services("));
    assert!(activation_source.contains("if let [service] = startup_services"));
    assert!(activation_source.contains("self.resolve_registered_service(service, None)?"));
    assert!(activation_source.contains("if let [first_service, second_service] = startup_services"));
    assert!(activation_source.contains("self.resolve_registered_service(first_service, None)?"));
    assert!(activation_source.contains("self.resolve_registered_service(second_service, None)?"));
    assert!(activation_source
        .contains("if let [first_service, second_service, third_service] = startup_services"));
    assert!(activation_source.contains("self.resolve_registered_service(third_service, None)?"));
    assert!(activation_source.contains(
        "if let [first_service, second_service, third_service, fourth_service] = startup_services"
    ));
    assert!(activation_source.contains("self.resolve_registered_service(fourth_service, None)?"));
    assert!(activation_source.contains("fifth_service"));
    assert!(activation_source.contains("self.resolve_registered_service(fifth_service, None)?"));
    assert!(activation_tests_source.contains(
        "fn activate_exact_three_immediate_services_initializes_each_cached_startup_entry_once()"
    ));
    assert!(activation_tests_source.contains(
        "fn activate_exact_four_immediate_services_initializes_each_cached_startup_entry_once()"
    ));
    assert!(activation_tests_source.contains(
        "fn activate_exact_five_immediate_services_initializes_each_cached_startup_entry_once()"
    ));
    let empty_startup_index = activation_source
        .find("if entry.startup_service_names.is_empty()")
        .expect("activation should fast-path modules with no startup services");
    let startup_clone_index = activation_source
        .find("entry.startup_service_names.clone()")
        .expect("activation should clone cached startup services for non-empty modules");
    let startup_resolve_index = activation_source
        .find("self.resolve_startup_services(startup_services.as_ref())?;")
        .expect("activation should route non-empty startup lists through the typed helper");
    assert!(empty_startup_index < startup_clone_index);
    assert!(empty_startup_index < startup_resolve_index);
    let startup_helper_index = activation_source
        .find("fn resolve_startup_services(")
        .expect("activation should keep startup resolution helper private to the handle");
    let single_startup_index = activation_source[startup_helper_index..]
        .find("if let [service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("single-startup service should bypass the multi-service loop");
    let two_startup_index = activation_source[startup_helper_index..]
        .find("if let [first_service, second_service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("two-startup services should bypass the multi-service loop");
    let first_two_startup_resolve_index = activation_source[startup_helper_index..]
        .find("self.resolve_registered_service(first_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("two-startup services should resolve the first key directly");
    let second_two_startup_resolve_index = activation_source[startup_helper_index..]
        .find("self.resolve_registered_service(second_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("two-startup services should resolve the second key directly");
    let three_startup_index = activation_source[startup_helper_index..]
        .find("if let [first_service, second_service, third_service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("three-startup services should bypass the multi-service loop");
    let third_startup_resolve_index = activation_source[startup_helper_index..]
        .find("self.resolve_registered_service(third_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("three-startup services should resolve the third key directly");
    let four_startup_index = activation_source[startup_helper_index..]
        .find("if let [first_service, second_service, third_service, fourth_service] = startup_services")
        .map(|offset| startup_helper_index + offset)
        .expect("four-startup services should bypass the multi-service loop");
    let fourth_startup_resolve_index = activation_source[startup_helper_index..]
        .find("self.resolve_registered_service(fourth_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("four-startup services should resolve the fourth key directly");
    let five_startup_index = activation_source[startup_helper_index..]
        .find("fifth_service")
        .map(|offset| startup_helper_index + offset)
        .expect("five-startup services should bypass the multi-service loop");
    let fifth_startup_resolve_index = activation_source[startup_helper_index..]
        .find("self.resolve_registered_service(fifth_service, None)?")
        .map(|offset| startup_helper_index + offset)
        .expect("five-startup services should resolve the fifth key directly");
    let startup_loop_index = activation_source[startup_helper_index..]
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
    let empty_shutdown_index = activation_source
        .find("if entry.shutdown_service_names.is_empty()")
        .expect("deactivation should fast-path modules with no shutdown services");
    let shutdown_clone_index = activation_source
        .find("entry.shutdown_service_names.clone()")
        .expect("deactivation should clone cached shutdown services for non-empty modules");
    let service_precheck_index = activation_source
        .find("let blocked_unload = {")
        .expect("deactivation should retain blocked-unload precheck for non-empty modules");
    assert!(empty_shutdown_index < shutdown_clone_index);
    assert!(empty_shutdown_index < service_precheck_index);
    assert!(activation_source.contains("fn finish_module_deactivation("));
    assert!(activation_source.contains("let unload_order = unload_order.as_ref()"));
    assert!(activation_source.contains("unload_services(&mut services, unload_order);"));
    assert!(activation_source.contains("fn unload_services("));
    assert!(activation_source.contains("fn unload_service("));
    assert!(activation_source.contains("unload_service(services, service_name);"));
    assert!(activation_source.contains("services.get_mut(service_name)"));
    let unload_services_index = activation_source
        .find("fn unload_services(")
        .expect("successful unload should be routed through a typed helper");
    let single_service_mutation_index = activation_source[unload_services_index..]
        .find("if let [service_name] = unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("single-service unload mutation should bypass the multi-service loop");
    let two_service_mutation_index = activation_source[unload_services_index..]
        .find("if let [first_service_name, second_service_name] = unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("two-service unload mutation should bypass the multi-service loop");
    let first_two_service_mutation_index = activation_source[unload_services_index..]
        .find("unload_service(services, first_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("two-service unload mutation should unload the first service directly");
    let second_two_service_mutation_index = activation_source[unload_services_index..]
        .find("unload_service(services, second_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("two-service unload mutation should unload the second service directly");
    let three_service_mutation_index = activation_source[unload_services_index..]
        .find("if let [first_service_name, second_service_name, third_service_name] = unload_order")
        .map(|offset| unload_services_index + offset)
        .expect("three-service unload mutation should bypass the multi-service loop");
    let third_three_service_mutation_index = activation_source[unload_services_index..]
        .find("unload_service(services, third_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("three-service unload mutation should unload the third service directly");
    let four_service_mutation_index = activation_source[unload_services_index..]
        .find("if let [first_service_name, second_service_name, third_service_name, fourth_service_name]")
        .map(|offset| unload_services_index + offset)
        .expect("four-service unload mutation should bypass the multi-service loop");
    let fourth_four_service_mutation_index = activation_source[unload_services_index..]
        .find("unload_service(services, fourth_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("four-service unload mutation should unload the fourth service directly");
    let five_service_mutation_index = activation_source[unload_services_index..]
        .find("fifth_service_name")
        .map(|offset| unload_services_index + offset)
        .expect("five-service unload mutation should bypass the multi-service loop");
    let fifth_five_service_mutation_index = activation_source[unload_services_index..]
        .find("unload_service(services, fifth_service_name);")
        .map(|offset| unload_services_index + offset)
        .expect("five-service unload mutation should unload the fifth service directly");
    let unload_loop_index = activation_source[unload_services_index..]
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
    assert!(registration_source.contains("ServiceKind::Plugin"));
    assert!(registration_source.contains("ServiceKind::Manager"));
    assert!(registration_source.contains("ServiceKind::Driver"));
    assert!(!activation_source.contains("StartupMode"));
    assert!(!activation_source.contains("let immediate_services: Vec<RegistryName>"));
    assert!(!activation_source.contains(".filter_map(|name|"));
    assert!(!activation_source.contains("sort_by_key"));
    assert!(!activation_source.contains("fn service_start_order"));
    assert!(!activation_source.contains("fn service_stop_order"));
    assert!(activation_source.contains("fn first_blocked_unload("));
    assert!(activation_source.contains("if let [service_name] = unload_order"));
    assert!(activation_source.contains("fn first_blocked_single_service_unload("));
    assert!(activation_source
        .contains("return first_blocked_single_service_unload(services, service_name);"));
    assert!(activation_source
        .contains("if let [first_service_name, second_service_name] = unload_order"));
    assert!(activation_source.contains("fn first_blocked_two_service_unload("));
    assert!(activation_source.contains("return first_blocked_two_service_unload("));
    assert!(activation_source.contains(
        "if let [first_service_name, second_service_name, third_service_name] = unload_order"
    ));
    assert!(activation_source.contains("fn first_blocked_three_service_unload("));
    assert!(activation_source.contains("return first_blocked_three_service_unload("));
    assert!(activation_source.contains("fn first_blocked_four_service_unload("));
    assert!(activation_source.contains("return first_blocked_four_service_unload("));
    assert!(activation_source.contains("fn first_blocked_five_service_unload("));
    assert!(activation_source.contains("return first_blocked_five_service_unload("));
    assert!(activation_source.contains("dependent_name == first_service_name"));
    assert!(activation_source.contains("dependent_name == second_service_name"));
    assert!(activation_source.contains("dependent_name == third_service_name"));
    assert!(activation_source.contains("dependent_name == fourth_service_name"));
    assert!(activation_source.contains("dependent_name == fifth_service_name"));
    assert!(!activation_source.contains("fn first_blocked_two_service_dependency("));
    assert!(!activation_source.contains("fn first_blocked_three_service_dependency("));
    assert!(!activation_source.contains("fn first_blocked_four_service_dependency("));
    assert!(!activation_source.contains("fn first_blocked_five_service_dependency("));
    assert!(!activation_source.contains("let mut second_service_blocked = false;"));
    assert!(!activation_source.contains("let mut fifth_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("enum TwoServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_two_service_dependency("));
    assert!(blocked_dependencies_source.contains("enum ThreeServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_three_service_dependency("));
    assert!(blocked_dependencies_source.contains("enum FourServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_four_service_dependency("));
    assert!(blocked_dependencies_source.contains("enum FiveServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_five_service_dependency("));
    assert!(blocked_dependencies_source.contains("let mut second_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("let mut third_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("let mut fourth_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("let mut fifth_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("if dependency == first_service_name"));
    assert!(blocked_dependencies_source.contains("if dependency == second_service_name"));
    assert!(blocked_dependencies_source.contains("dependency == third_service_name"));
    assert!(blocked_dependencies_source.contains("dependency == fourth_service_name"));
    assert!(blocked_dependencies_source.contains("dependency == fifth_service_name"));
    assert!(activation_source
        .contains("let blocked_service_name = if first_blocked_dependents.is_some()"));
    assert!(activation_source.contains("first_blocked_dependents.or(second_blocked_dependents)"));
    assert!(activation_source.contains("dependent_name == service_name"));
    let single_blocked_helper_index = activation_source
        .find("fn first_blocked_single_service_unload(")
        .expect("single-service blocked unload should stay in its own helper");
    let two_blocked_helper_index = activation_source
        .find("fn first_blocked_two_service_unload(")
        .expect("two-service blocked unload should stay in its own helper");
    let three_blocked_helper_index = activation_source
        .find("fn first_blocked_three_service_unload(")
        .expect("three-service blocked unload should stay in its own helper");
    let four_blocked_helper_index = activation_source
        .find("fn first_blocked_four_service_unload(")
        .expect("four-service blocked unload should stay in its own helper");
    let five_blocked_helper_index = activation_source
        .find("fn first_blocked_five_service_unload(")
        .expect("five-service blocked unload should stay in its own helper");
    assert!(single_blocked_helper_index < two_blocked_helper_index);
    assert!(two_blocked_helper_index < three_blocked_helper_index);
    assert!(three_blocked_helper_index < four_blocked_helper_index);
    assert!(four_blocked_helper_index < five_blocked_helper_index);
    let dependency_matcher_index = blocked_dependencies_source
        .find("fn dependency_slice_contains_service(")
        .expect("single-service blocked unload should route dependency matching through a helper");
    let two_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_two_service_dependency(")
        .expect("two-service blocked unload should route dependency matching through a helper");
    let three_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_three_service_dependency(")
        .expect("three-service blocked unload should route dependency matching through a helper");
    let four_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_four_service_dependency(")
        .expect("four-service blocked unload should route dependency matching through a helper");
    let five_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_five_service_dependency(")
        .expect("five-service blocked unload should route dependency matching through a helper");
    assert!(dependency_matcher_index < two_dependency_matcher_index);
    assert!(two_dependency_matcher_index < three_dependency_matcher_index);
    assert!(three_dependency_matcher_index < four_dependency_matcher_index);
    assert!(four_dependency_matcher_index < five_dependency_matcher_index);
    let single_blocked_helper_source =
        &activation_source[single_blocked_helper_index..two_blocked_helper_index];
    assert!(single_blocked_helper_source
        .contains("dependency_slice_contains_service(entry.dependencies.as_ref(), service_name)"));
    assert!(!single_blocked_helper_source.contains(".any(|dependency| dependency == service_name)"));
    let dependency_matcher_source =
        &blocked_dependencies_source[dependency_matcher_index..two_dependency_matcher_index];
    assert!(dependency_matcher_source.contains("match dependencies"));
    assert!(dependency_matcher_source.contains("[] => false"));
    assert!(dependency_matcher_source.contains("[dependency] => dependency == service_name"));
    assert!(dependency_matcher_source.contains("[first_dependency, second_dependency]"));
    assert!(dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(dependency_matcher_source.contains("fifth_dependency"));
    assert!(dependency_matcher_source.contains("_ => dependencies"));
    assert!(dependency_matcher_source.contains(".any(|dependency| dependency == service_name)"));
    let two_dependency_matcher_source =
        &blocked_dependencies_source[two_dependency_matcher_index..three_dependency_matcher_index];
    assert!(two_dependency_matcher_source.contains("match dependencies"));
    assert!(two_dependency_matcher_source.contains("[] => None"));
    assert!(two_dependency_matcher_source.contains("Some(TwoServiceDependencyMatch::FirstService)"));
    assert!(
        two_dependency_matcher_source.contains("Some(TwoServiceDependencyMatch::SecondService)")
    );
    assert!(two_dependency_matcher_source.contains("[dependency]"));
    assert!(two_dependency_matcher_source.contains("[first_dependency, second_dependency]"));
    assert!(two_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(two_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(two_dependency_matcher_source.contains("fifth_dependency"));
    assert!(two_dependency_matcher_source.contains("_ => {"));
    assert!(two_dependency_matcher_source.contains("for dependency in dependencies"));
    let two_blocked_helper_source =
        &activation_source[two_blocked_helper_index..three_blocked_helper_index];
    assert!(two_blocked_helper_source.contains("first_blocked_two_service_dependency("));
    assert!(two_blocked_helper_source.contains("Some(TwoServiceDependencyMatch::FirstService)"));
    assert!(two_blocked_helper_source.contains("Some(TwoServiceDependencyMatch::SecondService)"));
    assert!(!two_blocked_helper_source.contains("for dependency in entry.dependencies.iter()"));
    let three_dependency_matcher_source =
        &blocked_dependencies_source[three_dependency_matcher_index..four_dependency_matcher_index];
    assert!(three_dependency_matcher_source.contains("match dependencies"));
    assert!(three_dependency_matcher_source.contains("[] => None"));
    assert!(
        three_dependency_matcher_source.contains("Some(ThreeServiceDependencyMatch::FirstService)")
    );
    assert!(three_dependency_matcher_source
        .contains("Some(ThreeServiceDependencyMatch::SecondService)"));
    assert!(
        three_dependency_matcher_source.contains("Some(ThreeServiceDependencyMatch::ThirdService)")
    );
    assert!(three_dependency_matcher_source.contains("[dependency]"));
    assert!(three_dependency_matcher_source.contains("[first_dependency, second_dependency]"));
    assert!(three_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(three_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(three_dependency_matcher_source.contains("fifth_dependency"));
    assert!(three_dependency_matcher_source.contains("_ => {"));
    assert!(three_dependency_matcher_source.contains("for dependency in dependencies"));
    let three_blocked_helper_source = activation_source[three_blocked_helper_index
        ..activation_source
            .find("fn first_blocked_four_service_unload(")
            .expect("four-service blocked unload should stay in its own helper")]
        .to_string();
    assert!(three_blocked_helper_source.contains("first_blocked_three_service_dependency("));
    assert!(three_blocked_helper_source.contains("Some(ThreeServiceDependencyMatch::FirstService)"));
    assert!(
        three_blocked_helper_source.contains("Some(ThreeServiceDependencyMatch::SecondService)")
    );
    assert!(three_blocked_helper_source.contains("Some(ThreeServiceDependencyMatch::ThirdService)"));
    assert!(!three_blocked_helper_source.contains("for dependency in entry.dependencies.iter()"));
    let four_dependency_matcher_source =
        &blocked_dependencies_source[four_dependency_matcher_index..five_dependency_matcher_index];
    assert!(four_dependency_matcher_source.contains("match dependencies"));
    assert!(four_dependency_matcher_source.contains("[] => None"));
    assert!(
        four_dependency_matcher_source.contains("Some(FourServiceDependencyMatch::FirstService)")
    );
    assert!(
        four_dependency_matcher_source.contains("Some(FourServiceDependencyMatch::SecondService)")
    );
    assert!(
        four_dependency_matcher_source.contains("Some(FourServiceDependencyMatch::ThirdService)")
    );
    assert!(
        four_dependency_matcher_source.contains("Some(FourServiceDependencyMatch::FourthService)")
    );
    assert!(four_dependency_matcher_source.contains("[dependency]"));
    assert!(four_dependency_matcher_source.contains("[first_dependency, second_dependency]"));
    assert!(four_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(four_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(four_dependency_matcher_source.contains("fifth_dependency"));
    assert!(four_dependency_matcher_source.contains("_ => {"));
    assert!(four_dependency_matcher_source.contains("for dependency in dependencies"));
    let four_blocked_helper_source = activation_source[four_blocked_helper_index
        ..activation_source
            .find("fn first_blocked_five_service_unload(")
            .expect("five-service blocked unload should stay in its own helper")]
        .to_string();
    assert!(four_blocked_helper_source.contains("first_blocked_four_service_dependency("));
    assert!(four_blocked_helper_source.contains("Some(FourServiceDependencyMatch::FirstService)"));
    assert!(four_blocked_helper_source.contains("Some(FourServiceDependencyMatch::SecondService)"));
    assert!(four_blocked_helper_source.contains("Some(FourServiceDependencyMatch::ThirdService)"));
    assert!(four_blocked_helper_source.contains("Some(FourServiceDependencyMatch::FourthService)"));
    assert!(!four_blocked_helper_source.contains("for dependency in entry.dependencies.iter()"));
    let five_dependency_matcher_source = &blocked_dependencies_source[five_dependency_matcher_index..];
    assert!(five_dependency_matcher_source.contains("match dependencies"));
    assert!(five_dependency_matcher_source.contains("[] => None"));
    assert!(
        five_dependency_matcher_source.contains("Some(FiveServiceDependencyMatch::FirstService)")
    );
    assert!(
        five_dependency_matcher_source.contains("Some(FiveServiceDependencyMatch::SecondService)")
    );
    assert!(
        five_dependency_matcher_source.contains("Some(FiveServiceDependencyMatch::ThirdService)")
    );
    assert!(
        five_dependency_matcher_source.contains("Some(FiveServiceDependencyMatch::FourthService)")
    );
    assert!(
        five_dependency_matcher_source.contains("Some(FiveServiceDependencyMatch::FifthService)")
    );
    assert!(five_dependency_matcher_source.contains("[dependency]"));
    assert!(five_dependency_matcher_source.contains("[first_dependency, second_dependency]"));
    assert!(five_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(five_dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(five_dependency_matcher_source.contains("fifth_dependency"));
    assert!(five_dependency_matcher_source.contains("_ => {"));
    assert!(five_dependency_matcher_source.contains("for dependency in dependencies"));
    let five_blocked_helper_source = &activation_source[five_blocked_helper_index..];
    assert!(five_blocked_helper_source.contains("first_blocked_five_service_dependency("));
    assert!(five_blocked_helper_source.contains("Some(FiveServiceDependencyMatch::FirstService)"));
    assert!(five_blocked_helper_source.contains("Some(FiveServiceDependencyMatch::SecondService)"));
    assert!(five_blocked_helper_source.contains("Some(FiveServiceDependencyMatch::ThirdService)"));
    assert!(five_blocked_helper_source.contains("Some(FiveServiceDependencyMatch::FourthService)"));
    assert!(five_blocked_helper_source.contains("Some(FiveServiceDependencyMatch::FifthService)"));
    assert!(!five_blocked_helper_source.contains("for dependency in entry.dependencies.iter()"));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_two_services_reports_first_shutdown_service_when_dependent_names_both()"
    ));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_three_services_reports_first_shutdown_service_when_dependent_names_all()"
    ));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_four_services_reports_first_shutdown_service_when_dependent_names_all()"
    ));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_five_services_reports_first_shutdown_service_when_dependent_names_all()"
    ));
    let single_service_precheck_index = activation_source
        .find("return first_blocked_single_service_unload(services, service_name);")
        .expect("single-service unload should bypass the multi-service unload index");
    let two_service_precheck_index = activation_source
        .find("return first_blocked_two_service_unload(services, first_service_name, second_service_name);")
        .expect("two-service unload should bypass the multi-service unload index");
    let three_service_precheck_index = activation_source
        .find("return first_blocked_three_service_unload(")
        .expect("three-service unload should bypass the multi-service unload index");
    let four_service_precheck_index = activation_source
        .find("return first_blocked_four_service_unload(")
        .expect("four-service unload should bypass the multi-service unload index");
    let five_service_precheck_index = activation_source
        .find("return first_blocked_five_service_unload(")
        .expect("five-service unload should bypass the multi-service unload index");
    let unload_index_map_index = activation_source
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
    assert!(activation_source.contains("const BLOCKED_DEPENDENT_INITIAL_CAPACITY: usize = 1;"));
    assert!(activation_source.contains("let unload_indices: HashMap<&RegistryName, usize>"));
    assert!(activation_source.contains("HashMap::with_capacity(unload_order.len())"));
    assert!(activation_source.contains("unload_indices.insert(service_name, index)"));
    assert!(activation_source.contains("let mut blocked_index = None"));
    assert!(activation_source.contains("let mut blocked_dependents = None"));
    assert!(activation_source.contains("fn record_blocked_dependent("));
    assert!(activation_source.contains("for (dependent_name, entry) in services.iter()"));
    assert!(activation_source.contains("unload_indices.contains_key(dependent_name)"));
    assert!(activation_source.contains("for dependency in entry.dependencies.iter()"));
    assert!(activation_source.contains("unload_indices.get(dependency).copied()"));
    assert!(activation_source.contains("record_blocked_dependent("));
    assert!(activation_source.contains("match (blocked_index, blocked_dependents)"));
    assert!(activation_source.contains("Some((unload_order[index].to_string(), dependents))"));
    assert!(activation_source
        .contains("get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))"));
    assert!(activation_source.contains("dependents.clear()"));
    assert!(!activation_source.contains("HashSet<RegistryName>"));
    assert!(!activation_source.contains("HashSet<String>"));
    assert!(!activation_source.contains("unloading.contains(dependent_name)"));
    assert!(!activation_source.contains("let mut dependents_by_service = vec![Vec::new();"));
    assert!(!activation_source.contains("let mut blocked_dependents = Vec::new()"));
    assert!(!activation_source.contains(".zip(dependents_by_service)"));
    assert!(!activation_source.contains("fn running_dependents("));
    assert!(!activation_source.contains("owner_module == module_name"));
    assert!(!activation_source.contains(".map(|(name, _)| name.clone())"));
    assert!(!activation_source.contains("(entry.kind, name.clone())"));
    assert!(!activation_source.contains("names.into_iter().map(|(_, name)| name)"));
    assert!(!activation_source.contains("let service_name = service_name.as_str();"));
    assert!(!activation_source.contains("services.get_mut(service_name.as_str())"));
    assert!(!activation_source.contains("self.resolve_named_service(service.as_str(), None)?"));
    assert!(!activation_source.contains("entry.name"));
    assert!(!activation_source.contains("entry.name.to_string()"));
}
