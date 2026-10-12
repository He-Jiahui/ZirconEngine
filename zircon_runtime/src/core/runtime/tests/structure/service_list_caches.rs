use super::registration_sources;

#[test]
fn registration_source_preserves_service_list_cache_paths() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();
    let behavior_tests = sources.behavior_tests.as_str();

    assert!(registration.contains("service_names: Arc::default()"));
    assert!(registration.contains("startup_service_names: Arc::default()"));
    assert!(registration.contains("shutdown_service_names: Arc::default()"));
    assert!(registration.contains("let module_service_lists ="));
    assert!(registration.contains(
        "module_service_lists(&pending_services, driver_count, manager_count, plugin_count)"
    ));
    assert!(registration.contains("struct ModuleServiceLists"));
    assert!(registration.contains("service_names: Arc<[RegistryName]>"));
    assert!(registration.contains("startup_service_names: Arc<[RegistryName]>"));
    assert!(registration.contains("shutdown_service_names: Arc<[RegistryName]>"));
    assert!(registration.contains("Vec::with_capacity(pending_services.len())"));
    assert!(registration.contains("if let [(name, entry)] = pending_services"));
    assert!(registration.contains("fn single_service_module_lists("));
    assert!(
        registration.contains("let service_names = Arc::<[RegistryName]>::from([name.clone()]);")
    );
    assert!(registration
        .contains("let startup_service_names = if entry.startup_mode == StartupMode::Immediate"));
    let single_service_helper_start = registration
        .find("fn single_service_module_lists(")
        .expect("single-service list helper should exist");
    let two_service_helper_start = registration
        .find("fn two_service_module_lists(")
        .expect("two-service list helper should exist");
    let single_service_helper =
        &registration[single_service_helper_start..two_service_helper_start];
    assert!(single_service_helper.contains(
        "let startup_service_names = if entry.startup_mode == StartupMode::Immediate {\n        service_names.clone()"
    ));
    assert!(registration.contains("let shutdown_service_names = service_names.clone();"));
    assert!(!registration.contains("let mut service_names = Vec::with_capacity(1);"));
    assert!(!registration.contains("let mut shutdown_service_names = Vec::with_capacity(1);"));
    let single_service_lists_match = registration
        .find("if let [(name, entry)] = pending_services")
        .expect("single-service modules should bypass the multi-service list builder");
    assert!(registration.contains(
        "if let [(first_name, first_entry), (second_name, second_entry)] = pending_services"
    ));
    assert!(registration.contains(
        "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry)] ="
    ));
    assert!(registration.contains(
        "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry)] ="
    ));
    assert!(registration.contains(
        "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry), (fifth_name, fifth_entry)] ="
    ));
    assert!(registration.contains("fn two_service_module_lists("));
    assert!(registration.contains("fn three_service_module_lists("));
    assert!(registration.contains("fn four_service_module_lists("));
    assert!(registration.contains("fn five_service_module_lists("));
    assert!(registration.contains("let service_names = Arc::<[RegistryName]>::from(["));
    assert!(registration
        .contains("let first_immediate = first_entry.startup_mode == StartupMode::Immediate"));
    assert!(registration
        .contains("let second_immediate = second_entry.startup_mode == StartupMode::Immediate"));
    assert!(registration.contains("let startup_service_names = match"));
    assert!(registration.contains("(true, true) => service_names.clone()"));
    assert!(
        registration.contains("if driver_count == 2 || manager_count == 2 || plugin_count == 2")
    );
    assert!(registration.contains("service_names.clone()"));
    assert!(registration.contains("Arc::<[RegistryName]>::from(["));
    assert!(!registration.contains("let mut startup_capacity = 0_usize"));
    assert!(!registration
        .contains("let mut startup_service_names = Vec::with_capacity(startup_capacity)"));
    assert!(!registration.contains("let mut shutdown_service_names = Vec::with_capacity(2);"));
    let two_service_lists_match = registration
        .find("if let [(first_name, first_entry), (second_name, second_entry)] = pending_services")
        .expect("two-service modules should bypass the multi-service immediate-count path");
    let four_service_lists_match = registration
        .find(
            "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry)] =",
        )
        .expect("four-service modules should bypass the six-or-more immediate-count path");
    let five_service_lists_match = registration
        .find(
            "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry), (fifth_name, fifth_entry)] =",
        )
        .expect("five-service modules should bypass the six-or-more immediate-count path");
    let two_service_helper_index = registration
        .find("fn two_service_module_lists(")
        .expect("two-service modules should use direct cached-list construction");
    assert!(registration.contains("struct MultiServiceListScan"));
    assert!(registration.contains("fn scan_multi_service_module_lists("));
    assert!(registration.contains("let scan = scan_multi_service_module_lists(pending_services);"));
    assert!(registration.contains("service_names: service_names.into()"));
    let scan_helper_index = registration
        .find("fn scan_multi_service_module_lists(")
        .expect("six-or-more service list construction should have one scan helper");
    let immediate_count_index = registration
        .find("let mut immediate_count = 0_usize")
        .expect("six-or-more service modules should still count immediate services exactly");
    let service_names_push_index = registration
        .find("service_names.push(name.clone())")
        .expect("five-or-more scan should build the owner service cache");
    let three_service_lists_index = registration
        .find("fn three_service_module_lists(")
        .expect("three-service modules should use direct cached-list construction");
    let four_service_helper_index = registration
        .find("fn four_service_module_lists(")
        .expect("four-service modules should use direct cached-list construction");
    let five_service_helper_index = registration
        .find("fn five_service_module_lists(")
        .expect("five-service modules should use direct cached-list construction");
    assert!(single_service_lists_match < immediate_count_index);
    assert!(single_service_lists_match < two_service_lists_match);
    assert!(two_service_lists_match < immediate_count_index);
    assert!(two_service_lists_match < four_service_lists_match);
    assert!(four_service_lists_match < immediate_count_index);
    assert!(four_service_lists_match < five_service_lists_match);
    assert!(five_service_lists_match < immediate_count_index);
    assert!(scan_helper_index < immediate_count_index);
    assert!(immediate_count_index < service_names_push_index);
    assert!(three_service_lists_index > immediate_count_index);
    assert!(four_service_helper_index > three_service_lists_index);
    assert!(five_service_helper_index > four_service_helper_index);
    assert!(immediate_count_index < two_service_helper_index);
    assert!(registration.contains("(true, false, true)"));
    assert!(registration.contains("(1, 1, 1) => Arc::<[RegistryName]>::from(["));
    assert!(registration.contains("(true, false, false, true)"));
    assert!(registration.contains("(1, 2, 1) => Arc::<[RegistryName]>::from(["));
    assert!(registration.contains("(true, false, false, true, false)"));
    assert!(registration.contains("(1, 2, 2) => Arc::<[RegistryName]>::from(["));
    assert!(
        behavior_tests.contains("fn register_exact_three_mixed_kind_services_keep_direct_caches()")
    );
    assert!(
        behavior_tests.contains("fn register_exact_four_mixed_kind_services_keep_direct_caches()")
    );
    assert!(
        behavior_tests.contains("fn register_exact_five_mixed_kind_services_keep_direct_caches()")
    );
    assert!(!behavior_tests
        .contains("fn register_four_service_mixed_startup_uses_scanned_owner_cache()"));
    assert!(registration.contains("if scan.immediate_count == 0"));
    assert!(registration.contains("fn lazy_multi_service_module_lists("));
    let lazy_multi_service_lists_index = registration
        .find("fn lazy_multi_service_module_lists(")
        .expect("lazy multi-service list helper should exist");
    let single_service_lists_index = registration
        .find("fn single_service_module_lists(")
        .expect("single-service list helper should exist");
    let lazy_multi_service_lists =
        &registration[lazy_multi_service_lists_index..single_service_lists_index];
    assert!(lazy_multi_service_lists.contains("startup_service_names: Arc::default()"));
    assert!(!lazy_multi_service_lists.contains("Vec::with_capacity(0)"));
    assert!(!lazy_multi_service_lists
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    assert!(registration.contains("fn push_shutdown_service_names("));
    let lazy_multi_service_index = registration
        .find("if scan.immediate_count == 0")
        .expect("lazy multi-service modules should bypass startup cache fill");
    let startup_cache_fill_index = registration
        .find("let mut startup_service_names = Vec::with_capacity(immediate_count)")
        .expect("immediate multi-service modules should keep exact startup capacity");
    assert!(lazy_multi_service_index < startup_cache_fill_index);
    assert!(registration.contains("if scan.immediate_count == pending_services.len()"));
    assert!(registration.contains("let startup_service_names = service_names.clone();"));
    assert!(behavior_tests.contains(
        "fn register_exact_two_all_immediate_services_reuses_owner_cache_for_startup_cache()"
    ));
    assert!(behavior_tests.contains(
        "fn register_all_immediate_multi_service_reuses_owner_cache_for_startup_cache()"
    ));
    assert!(registration.contains("if scan.immediate_count == 1"));
    assert!(registration.contains("let mut single_immediate_index = 0_usize"));
    assert!(
        registration.contains("for (index, (name, entry)) in pending_services.iter().enumerate()")
    );
    assert!(registration.contains("service_names.push(name.clone())"));
    assert!(registration.contains("single_immediate_index = index"));
    assert!(registration.contains("scan.single_immediate_index"));
    assert!(registration.contains("fn single_startup_multi_service_module_lists("));
    assert!(behavior_tests
        .contains("fn register_single_startup_multi_service_keeps_direct_startup_cache()"));
    assert!(registration.contains("fn mixed_startup_multi_service_module_lists("));
    assert!(registration
        .contains("mixed_startup_multi_service_module_lists(\n        scan.service_names,"));
    let all_immediate_service_index = registration
        .find("if scan.immediate_count == pending_services.len()")
        .expect("all-immediate multi-service modules should share owner/startup cache");
    let single_startup_service_index = registration
        .find("if scan.immediate_count == 1")
        .expect("single-startup multi-service modules should bypass startup Vec construction");
    assert!(lazy_multi_service_index < all_immediate_service_index);
    assert!(all_immediate_service_index < startup_cache_fill_index);
    assert!(all_immediate_service_index < single_startup_service_index);
    assert!(single_startup_service_index < startup_cache_fill_index);
    let single_startup_helper_index = registration
        .find("fn single_startup_multi_service_module_lists(")
        .expect("single-startup list helper should own exact-one startup construction");
    let mixed_startup_helper_index = registration
        .find("fn mixed_startup_multi_service_module_lists(")
        .expect("mixed-startup list helper should own two-or-more startup construction");
    assert!(single_startup_service_index < single_startup_helper_index);
    assert!(single_startup_helper_index < mixed_startup_helper_index);
    let single_startup_helper =
        &registration[single_startup_helper_index..mixed_startup_helper_index];
    assert!(single_startup_helper.contains("service_names: Arc<[RegistryName]>"));
    assert!(single_startup_helper.contains("startup_service_index: usize"));
    assert!(
        single_startup_helper.contains("pending_services[startup_service_index].1.startup_mode")
    );
    assert!(single_startup_helper
        .contains("let startup_service_name = pending_services[startup_service_index].0.clone();"));
    assert!(!single_startup_helper.contains("for (name, _) in pending_services"));
    assert!(!single_startup_helper
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    assert!(single_startup_helper.contains("Arc::<[RegistryName]>::from([startup_service_name])"));
    assert!(!single_startup_helper.contains("Vec::with_capacity(immediate_count)"));
    assert!(!single_startup_helper.contains("if entry.startup_mode"));
    let lazy_helper = &registration[lazy_multi_service_lists_index..single_service_lists_index];
    assert!(lazy_helper.contains("service_names: Arc<[RegistryName]>"));
    let all_immediate_helper_index = registration
        .find("fn all_immediate_multi_service_module_lists(")
        .expect("all-immediate helper should exist");
    let all_immediate_helper =
        &registration[all_immediate_helper_index..lazy_multi_service_lists_index];
    assert!(all_immediate_helper.contains("service_names: Arc<[RegistryName]>"));
    assert!(!all_immediate_helper
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    let mixed_startup_helper =
        &registration[mixed_startup_helper_index..all_immediate_helper_index];
    assert!(mixed_startup_helper.contains("service_names: Arc<[RegistryName]>"));
    assert!(!mixed_startup_helper
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    assert!(registration.contains("fn shutdown_service_names_or_owner_clone("));
    assert!(registration.contains("fn shutdown_order_matches_owner_order("));
    assert!(registration.contains("return owner_service_names.clone();"));
    assert!(registration.contains("driver_count == service_count"));
    assert!(registration.contains("manager_count == service_count"));
    assert!(registration.contains("plugin_count == service_count"));
    assert!(behavior_tests
        .contains("fn register_same_kind_multi_service_reuses_owner_cache_for_shutdown_cache()"));
    assert!(registration.contains("debug_assert_eq!("));
    assert!(registration.contains("let manager_start = driver_count"));
    assert!(registration.contains("let plugin_start = driver_count + manager_count"));
    assert!(registration.contains("let plugin_end = plugin_start + plugin_count"));
    assert!(registration.contains("fn push_service_names("));
    assert!(registration.contains("target: &mut Vec<RegistryName>"));
    assert!(registration.contains("for (name, _) in services"));
    assert!(registration.contains("target.push(name.clone())"));
    assert!(registration.contains("push_shutdown_service_names("));
    assert!(registration.contains("pending_services[plugin_start..plugin_end]"));
    assert!(registration.contains("pending_services[manager_start..plugin_start]"));
    assert!(registration.contains("pending_services[..driver_count]"));
}
