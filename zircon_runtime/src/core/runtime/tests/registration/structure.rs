#[test]
fn registration_source_preserves_hot_path_structure() {
    let runtime_inner_source = include_str!("../../state/runtime_inner.rs");
    let service_entry_source = include_str!("../../state/service_entry.rs");
    let registration_mod_source = include_str!("../../handle/registration/mod.rs");
    let registration_register_module_source =
        include_str!("../../handle/registration/register_module.rs");
    let registration_descriptor_entries_source =
        include_str!("../../handle/registration/descriptor_entries.rs");
    let registration_descriptor_entries_five_source =
        include_str!("../../handle/registration/descriptor_entries_five.rs");
    let registration_descriptor_entries_four_source =
        include_str!("../../handle/registration/descriptor_entries_four.rs");
    let registration_descriptor_entries_three_source =
        include_str!("../../handle/registration/descriptor_entries_three.rs");
    let registration_duplicates_source = include_str!("../../handle/registration/duplicates.rs");
    let registration_entry_source = include_str!("../../handle/registration/entry.rs");
    let registration_service_lists_source =
        include_str!("../../handle/registration/service_lists.rs");
    let registration_validation_source = include_str!("../../handle/registration/validation.rs");
    let registration_source = [
        registration_mod_source,
        registration_register_module_source,
        registration_descriptor_entries_source,
        registration_descriptor_entries_five_source,
        registration_descriptor_entries_four_source,
        registration_descriptor_entries_three_source,
        registration_service_lists_source,
        registration_entry_source,
        registration_duplicates_source,
        registration_validation_source,
    ]
    .join("\n");
    let registration_behavior_mod_source = include_str!("behavior.rs");
    let registration_behavior_tests_source = [
        include_str!("behavior/validation.rs"),
        include_str!("behavior/cache_lists.rs"),
        include_str!("behavior/commit.rs"),
        include_str!("behavior/canonical_keys.rs"),
    ]
    .join("\n");
    assert!(registration_behavior_mod_source.contains("mod validation;"));
    assert!(registration_behavior_mod_source.contains("mod cache_lists;"));
    assert!(registration_behavior_mod_source.contains("mod commit;"));
    assert!(registration_behavior_mod_source.contains("mod canonical_keys;"));
    assert!(!registration_behavior_mod_source.contains("#[test]"));
    assert!(!registration_behavior_mod_source.contains("use "));
    assert!(registration_behavior_tests_source
        .contains("fn register_module_rejects_noncanonical_module_names()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_single_immediate_service_keeps_exact_cached_service_lists()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_single_service_reports_existing_service_table_key()"));
    assert!(registration_behavior_tests_source
        .contains("fn service_table_is_keyed_by_canonical_registry_names()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_exact_four_dependencies_keeps_direct_dependency_name_cache()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_module_rejects_fourth_driver_dependency_on_manager()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_exact_five_dependencies_keeps_direct_dependency_name_cache()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_module_rejects_fifth_driver_dependency_on_manager()"));
    assert!(registration_behavior_tests_source.contains(
        "fn register_exact_four_services_reports_existing_fourth_key_without_partial_commit()"
    ));
    assert!(registration_behavior_tests_source.contains(
        "fn register_exact_five_services_reports_existing_fifth_key_without_partial_commit()"
    ));
    assert!(registration_mod_source.contains("mod descriptor_entries;"));
    assert!(registration_mod_source.contains("mod descriptor_entries_five;"));
    assert!(registration_mod_source.contains("mod descriptor_entries_four;"));
    assert!(registration_mod_source.contains("mod descriptor_entries_three;"));
    assert!(registration_mod_source.contains("mod duplicates;"));
    assert!(registration_mod_source.contains("mod entry;"));
    assert!(registration_mod_source.contains("mod register_module;"));
    assert!(registration_mod_source.contains("mod service_lists;"));
    assert!(registration_mod_source.contains("mod validation;"));
    assert!(!registration_mod_source.contains("use "));
    assert!(!registration_mod_source.contains("impl CoreHandle"));
    assert!(registration_register_module_source.contains("impl CoreHandle"));
    assert!(registration_register_module_source.contains("pub fn register_module("));
    assert!(registration_descriptor_entries_source.contains("pub(super) fn prepare_service_entry("));
    assert!(registration_descriptor_entries_source.contains("pub(super) fn prepare_driver_entry("));
    assert!(registration_descriptor_entries_source.contains("pub(super) fn prepare_manager_entry("));
    assert!(registration_descriptor_entries_source.contains("pub(super) fn prepare_plugin_entry("));
    assert!(!registration_descriptor_entries_three_source.contains("fn prepare_driver_entry("));
    assert!(!registration_descriptor_entries_three_source.contains("fn prepare_manager_entry("));
    assert!(!registration_descriptor_entries_three_source.contains("fn prepare_plugin_entry("));
    assert!(!registration_descriptor_entries_four_source.contains("fn prepare_driver_entry("));
    assert!(!registration_descriptor_entries_four_source.contains("fn prepare_manager_entry("));
    assert!(!registration_descriptor_entries_four_source.contains("fn prepare_plugin_entry("));
    assert!(!registration_descriptor_entries_five_source.contains("fn prepare_driver_entry("));
    assert!(!registration_descriptor_entries_five_source.contains("fn prepare_manager_entry("));
    assert!(!registration_descriptor_entries_five_source.contains("fn prepare_plugin_entry("));
    assert!(registration_descriptor_entries_three_source
        .contains("pub(super) fn prepare_three_descriptor_service_entries("));
    assert!(registration_descriptor_entries_four_source
        .contains("pub(super) fn prepare_four_descriptor_service_entries("));
    assert!(registration_descriptor_entries_five_source
        .contains("pub(super) fn prepare_five_descriptor_service_entries("));
    assert!(registration_duplicates_source
        .contains("pub(super) fn duplicate_existing_pending_service_name"));
    assert!(registration_entry_source.contains("pub(super) fn service_entry("));
    assert!(registration_service_lists_source.contains("pub(super) struct ModuleServiceLists"));
    assert!(registration_validation_source.contains("pub(super) fn validate_service_descriptor("));
    let pending_prepare_index = registration_source
        .find("let mut pending_services = Vec::with_capacity(")
        .expect("registration should prepare pending service entries before commit");
    let commit_lock_index = registration_source
        .rfind("let mut modules = self.inner.modules.lock().unwrap()")
        .expect("registration should retain a final transactional module commit lock");
    assert!(runtime_inner_source.contains("HashMap<RegistryName, ServiceEntry>"));
    assert!(service_entry_source.contains("dependencies: Arc<[RegistryName]>"));
    assert!(registration_source.contains("let driver_count = descriptor.drivers.len()"));
    assert!(registration_source.contains("let manager_count = descriptor.managers.len()"));
    assert!(registration_source.contains("let plugin_count = descriptor.plugins.len()"));
    assert!(registration_source
        .contains("let service_count = driver_count + manager_count + plugin_count"));
    let service_count_index = registration_source
        .find("let service_count = driver_count + manager_count + plugin_count")
        .expect("registration should derive service count before choosing registration path");
    let empty_module_fast_path_index = registration_source
        .find("if service_count == 0")
        .expect("registration should fast-path empty module descriptors");
    let non_empty_duplicate_precheck_index = registration_source
        .find("let modules = self.inner.modules.lock().unwrap();")
        .expect("non-empty registration should keep its duplicate-module precheck");
    let pending_services_index = registration_source
        .find("let mut pending_services = Vec::with_capacity(service_count)")
        .expect("non-empty registration should pre-size pending service entries");
    let pending_keys_index = registration_source
        .find("let mut pending_keys = HashSet::with_capacity(service_count)")
        .expect("non-empty registration should pre-size pending duplicate keys");
    let single_service_fast_path_index = registration_source
        .find("if service_count == 1")
        .expect("single-service registration should bypass pending duplicate storage");
    let two_service_fast_path_index = registration_source
        .find("if service_count == 2")
        .expect("two-service registration should bypass pending duplicate storage");
    let three_service_fast_path_index = registration_source
        .find("if service_count == 3")
        .expect("three-service registration should bypass pending duplicate storage");
    let four_service_fast_path_index = registration_source
        .find("if service_count == 4")
        .expect("four-service registration should bypass pending duplicate storage");
    let five_service_fast_path_index = registration_source
        .find("if service_count == 5")
        .expect("five-service registration should bypass pending duplicate storage");
    assert!(service_count_index < empty_module_fast_path_index);
    assert!(empty_module_fast_path_index < non_empty_duplicate_precheck_index);
    assert!(non_empty_duplicate_precheck_index < single_service_fast_path_index);
    assert!(single_service_fast_path_index < two_service_fast_path_index);
    assert!(two_service_fast_path_index < three_service_fast_path_index);
    assert!(three_service_fast_path_index < four_service_fast_path_index);
    assert!(four_service_fast_path_index < five_service_fast_path_index);
    assert!(five_service_fast_path_index < pending_services_index);
    assert!(five_service_fast_path_index < pending_keys_index);
    assert!(four_service_fast_path_index < pending_services_index);
    assert!(four_service_fast_path_index < pending_keys_index);
    assert!(three_service_fast_path_index < pending_services_index);
    assert!(three_service_fast_path_index < pending_keys_index);
    assert!(two_service_fast_path_index < pending_services_index);
    assert!(two_service_fast_path_index < pending_keys_index);
    assert!(single_service_fast_path_index < pending_services_index);
    assert!(single_service_fast_path_index < pending_keys_index);
    assert!(non_empty_duplicate_precheck_index < pending_services_index);
    assert!(empty_module_fast_path_index < pending_services_index);
    assert!(empty_module_fast_path_index < pending_keys_index);
    assert!(registration_source.contains("fn register_empty_module("));
    assert!(
        registration_source.contains("return self.register_empty_module(module_name, descriptor);")
    );
    assert!(registration_source
        .contains("return self.register_single_service_module(module_name, descriptor);"));
    assert!(registration_source.contains("fn register_single_service_module("));
    assert!(registration_source.contains("return self.register_two_service_module("));
    assert!(registration_source.contains("fn register_two_service_module("));
    assert!(registration_source.contains("return self.register_three_service_module("));
    assert!(registration_source.contains("fn register_three_service_module("));
    assert!(registration_source.contains("return self.register_four_service_module("));
    assert!(registration_source.contains("fn register_four_service_module("));
    assert!(registration_source.contains("return self.register_five_service_module("));
    assert!(registration_source.contains("fn register_five_service_module("));
    assert!(registration_source.contains("fn prepare_single_descriptor_service_entry("));
    assert!(registration_source.contains("fn prepare_two_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_three_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_four_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_five_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_single_service_entry("));
    assert!(registration_source.contains("fn validate_service_descriptor("));
    assert!(registration_source.contains("fn service_entry("));
    assert!(registration_source.contains("if services.contains_key(&service_name)"));
    assert!(registration_source.contains("services.insert(service_name, service_entry);"));
    assert!(registration_source.contains("let pending_services = ["));
    assert!(registration_source.contains("if first_service_name == second_service_name"));
    let two_service_registration_start = registration_register_module_source
        .find("fn register_two_service_module(")
        .expect("two-service registration should have a dedicated setup helper");
    let register_empty_module_start = registration_source
        .find("fn register_empty_module(")
        .expect("empty-module registration should have a dedicated commit helper");
    let three_service_registration_start = registration_register_module_source
        .find("fn register_three_service_module(")
        .expect("three-service registration should have a dedicated setup helper");
    let two_service_registration_source = &registration_register_module_source
        [two_service_registration_start..three_service_registration_start];
    assert!(!two_service_registration_source.contains("HashSet"));
    assert!(!two_service_registration_source.contains("Vec::with_capacity(service_count)"));
    let four_service_registration_start = registration_register_module_source
        .find("fn register_four_service_module(")
        .expect("four-service registration should have a dedicated setup helper");
    let five_service_registration_start = registration_register_module_source
        .find("fn register_five_service_module(")
        .expect("five-service registration should have a dedicated setup helper");
    let three_service_registration_source = &registration_register_module_source
        [three_service_registration_start..four_service_registration_start];
    assert!(!three_service_registration_source.contains("HashSet"));
    assert!(!three_service_registration_source.contains("Vec::with_capacity(service_count)"));
    assert!(!three_service_registration_source
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(!three_service_registration_source.contains("for (key, entry) in pending_services"));
    assert!(three_service_registration_source
        .contains("services.insert(first_service_name, first_service_entry);"));
    assert!(three_service_registration_source
        .contains("services.insert(second_service_name, second_service_entry);"));
    assert!(three_service_registration_source
        .contains("services.insert(third_service_name, third_service_entry);"));
    let four_service_registration_source = &registration_register_module_source
        [four_service_registration_start..five_service_registration_start];
    assert!(!four_service_registration_source.contains("HashSet"));
    assert!(!four_service_registration_source.contains("Vec::with_capacity(service_count)"));
    assert!(!four_service_registration_source
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(!four_service_registration_source.contains("for (key, entry) in pending_services"));
    assert!(four_service_registration_source.contains("prepare_four_descriptor_service_entries("));
    assert!(four_service_registration_source
        .contains("services.insert(fourth_service_name, fourth_service_entry);"));
    let five_service_registration_source =
        &registration_register_module_source[five_service_registration_start..];
    assert!(!five_service_registration_source.contains("HashSet"));
    assert!(!five_service_registration_source.contains("Vec::with_capacity(service_count)"));
    assert!(!five_service_registration_source
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(!five_service_registration_source.contains("for (key, entry) in pending_services"));
    assert!(five_service_registration_source.contains("prepare_five_descriptor_service_entries("));
    assert!(five_service_registration_source
        .contains("services.insert(fifth_service_name, fifth_service_entry);"));
    let generic_registration_source =
        &registration_source[pending_services_index..register_empty_module_start];
    assert!(generic_registration_source
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(generic_registration_source.contains("for (key, entry) in pending_services"));
    assert!(!generic_registration_source
        .contains("services.insert(first_service_name, first_service_entry);"));
    assert!(registration_source.contains("service_names: Arc::default()"));
    assert!(registration_source.contains("startup_service_names: Arc::default()"));
    assert!(registration_source.contains("shutdown_service_names: Arc::default()"));
    assert!(registration_source.contains("HashSet::with_capacity(service_count)"));
    assert!(registration_source.contains("!pending_keys.insert(name.clone())"));
    assert!(registration_source.contains("let module_service_lists ="));
    assert!(registration_source.contains(
        "module_service_lists(&pending_services, driver_count, manager_count, plugin_count)"
    ));
    assert!(registration_source.contains("struct ModuleServiceLists"));
    assert!(registration_source.contains("service_names: Arc<[RegistryName]>"));
    assert!(registration_source.contains("startup_service_names: Arc<[RegistryName]>"));
    assert!(registration_source.contains("shutdown_service_names: Arc<[RegistryName]>"));
    assert!(registration_source.contains("Vec::with_capacity(pending_services.len())"));
    assert!(registration_source.contains("if let [(name, entry)] = pending_services"));
    assert!(registration_source.contains("fn single_service_module_lists("));
    assert!(registration_source
        .contains("let service_names = Arc::<[RegistryName]>::from([name.clone()]);"));
    assert!(registration_source
        .contains("let startup_service_names = if entry.startup_mode == StartupMode::Immediate"));
    let single_service_helper_start = registration_source
        .find("fn single_service_module_lists(")
        .expect("single-service list helper should exist");
    let two_service_helper_start = registration_source
        .find("fn two_service_module_lists(")
        .expect("two-service list helper should exist");
    let single_service_helper_source =
        &registration_source[single_service_helper_start..two_service_helper_start];
    assert!(single_service_helper_source.contains(
        "let startup_service_names = if entry.startup_mode == StartupMode::Immediate {\n        service_names.clone()"
    ));
    assert!(registration_source.contains("let shutdown_service_names = service_names.clone();"));
    assert!(!registration_source.contains("let mut service_names = Vec::with_capacity(1);"));
    assert!(
        !registration_source.contains("let mut shutdown_service_names = Vec::with_capacity(1);")
    );
    let single_service_lists_index = registration_source
        .find("if let [(name, entry)] = pending_services")
        .expect("single-service modules should bypass the multi-service list builder");
    assert!(registration_source.contains(
        "if let [(first_name, first_entry), (second_name, second_entry)] = pending_services"
    ));
    assert!(registration_source.contains(
        "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry)] ="
    ));
    assert!(registration_source.contains(
        "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry)] ="
    ));
    assert!(registration_source.contains(
        "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry), (fifth_name, fifth_entry)] ="
    ));
    assert!(registration_source.contains("fn two_service_module_lists("));
    assert!(registration_source.contains("fn three_service_module_lists("));
    assert!(registration_source.contains("fn four_service_module_lists("));
    assert!(registration_source.contains("fn five_service_module_lists("));
    assert!(registration_source.contains("let service_names = Arc::<[RegistryName]>::from(["));
    assert!(registration_source
        .contains("let first_immediate = first_entry.startup_mode == StartupMode::Immediate"));
    assert!(registration_source
        .contains("let second_immediate = second_entry.startup_mode == StartupMode::Immediate"));
    assert!(registration_source.contains("let startup_service_names = match"));
    assert!(registration_source.contains("(true, true) => service_names.clone()"));
    assert!(registration_source
        .contains("if driver_count == 2 || manager_count == 2 || plugin_count == 2"));
    assert!(registration_source.contains("service_names.clone()"));
    assert!(registration_source.contains("Arc::<[RegistryName]>::from(["));
    assert!(!registration_source.contains("let mut startup_capacity = 0_usize"));
    assert!(!registration_source
        .contains("let mut startup_service_names = Vec::with_capacity(startup_capacity)"));
    assert!(
        !registration_source.contains("let mut shutdown_service_names = Vec::with_capacity(2);")
    );
    let two_service_lists_index = registration_source
        .find("if let [(first_name, first_entry), (second_name, second_entry)] = pending_services")
        .expect("two-service modules should bypass the multi-service immediate-count path");
    let four_service_lists_index = registration_source
        .find(
            "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry)] =",
        )
        .expect("four-service modules should bypass the six-or-more immediate-count path");
    let five_service_lists_index = registration_source
        .find(
            "if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry), (fifth_name, fifth_entry)] =",
        )
        .expect("five-service modules should bypass the six-or-more immediate-count path");
    let two_service_helper_index = registration_source
        .find("fn two_service_module_lists(")
        .expect("two-service modules should use direct cached-list construction");
    assert!(registration_source.contains("struct MultiServiceListScan"));
    assert!(registration_source.contains("fn scan_multi_service_module_lists("));
    assert!(registration_duplicates_source.contains("debug_assert!(pending_services.len() >= 6);"));
    assert!(registration_source
        .contains("let scan = scan_multi_service_module_lists(pending_services);"));
    assert!(registration_source.contains("service_names: service_names.into()"));
    let scan_helper_index = registration_source
        .find("fn scan_multi_service_module_lists(")
        .expect("six-or-more service list construction should have one scan helper");
    let immediate_count_index = registration_source
        .find("let mut immediate_count = 0_usize")
        .expect("six-or-more service modules should still count immediate services exactly");
    let service_names_push_index = registration_source
        .find("service_names.push(name.clone())")
        .expect("five-or-more scan should build the owner service cache");
    let three_service_lists_index = registration_source
        .find("fn three_service_module_lists(")
        .expect("three-service modules should use direct cached-list construction");
    let four_service_helper_index = registration_source
        .find("fn four_service_module_lists(")
        .expect("four-service modules should use direct cached-list construction");
    let five_service_helper_index = registration_source
        .find("fn five_service_module_lists(")
        .expect("five-service modules should use direct cached-list construction");
    assert!(single_service_lists_index < immediate_count_index);
    assert!(single_service_lists_index < two_service_lists_index);
    assert!(two_service_lists_index < immediate_count_index);
    assert!(two_service_lists_index < four_service_lists_index);
    assert!(four_service_lists_index < immediate_count_index);
    assert!(four_service_lists_index < five_service_lists_index);
    assert!(five_service_lists_index < immediate_count_index);
    assert!(scan_helper_index < immediate_count_index);
    assert!(immediate_count_index < service_names_push_index);
    assert!(three_service_lists_index > immediate_count_index);
    assert!(four_service_helper_index > three_service_lists_index);
    assert!(five_service_helper_index > four_service_helper_index);
    assert!(immediate_count_index < two_service_helper_index);
    assert!(registration_source.contains("(true, false, true)"));
    assert!(registration_source.contains("(1, 1, 1) => Arc::<[RegistryName]>::from(["));
    assert!(registration_source.contains("(true, false, false, true)"));
    assert!(registration_source.contains("(1, 2, 1) => Arc::<[RegistryName]>::from(["));
    assert!(registration_source.contains("(true, false, false, true, false)"));
    assert!(registration_source.contains("(1, 2, 2) => Arc::<[RegistryName]>::from(["));
    assert!(registration_behavior_tests_source
        .contains("fn register_exact_three_mixed_kind_services_keep_direct_caches()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_exact_four_mixed_kind_services_keep_direct_caches()"));
    assert!(registration_behavior_tests_source
        .contains("fn register_exact_five_mixed_kind_services_keep_direct_caches()"));
    assert!(!registration_behavior_tests_source
        .contains("fn register_four_service_mixed_startup_uses_scanned_owner_cache()"));
    assert!(registration_source.contains("if scan.immediate_count == 0"));
    assert!(registration_source.contains("fn lazy_multi_service_module_lists("));
    let lazy_multi_service_lists_index = registration_source
        .find("fn lazy_multi_service_module_lists(")
        .expect("lazy multi-service list helper should exist");
    let single_service_lists_index = registration_source
        .find("fn single_service_module_lists(")
        .expect("single-service list helper should exist");
    let lazy_multi_service_lists_source =
        &registration_source[lazy_multi_service_lists_index..single_service_lists_index];
    assert!(lazy_multi_service_lists_source.contains("startup_service_names: Arc::default()"));
    assert!(!lazy_multi_service_lists_source.contains("Vec::with_capacity(0)"));
    assert!(!lazy_multi_service_lists_source
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    assert!(registration_source.contains("fn push_shutdown_service_names("));
    let lazy_multi_service_index = registration_source
        .find("if scan.immediate_count == 0")
        .expect("lazy multi-service modules should bypass startup cache fill");
    let startup_cache_fill_index = registration_source
        .find("let mut startup_service_names = Vec::with_capacity(immediate_count)")
        .expect("immediate multi-service modules should keep exact startup capacity");
    assert!(lazy_multi_service_index < startup_cache_fill_index);
    assert!(registration_source.contains("if scan.immediate_count == pending_services.len()"));
    assert!(registration_source.contains("let startup_service_names = service_names.clone();"));
    assert!(registration_behavior_tests_source.contains(
        "fn register_exact_two_all_immediate_services_reuses_owner_cache_for_startup_cache()"
    ));
    assert!(registration_behavior_tests_source.contains(
        "fn register_all_immediate_multi_service_reuses_owner_cache_for_startup_cache()"
    ));
    assert!(registration_source.contains("if scan.immediate_count == 1"));
    assert!(registration_source.contains("let mut single_immediate_index = 0_usize"));
    assert!(registration_source
        .contains("for (index, (name, entry)) in pending_services.iter().enumerate()"));
    assert!(registration_source.contains("service_names.push(name.clone())"));
    assert!(registration_source.contains("single_immediate_index = index"));
    assert!(registration_source.contains("scan.single_immediate_index"));
    assert!(registration_source.contains("fn single_startup_multi_service_module_lists("));
    assert!(registration_behavior_tests_source
        .contains("fn register_single_startup_multi_service_keeps_direct_startup_cache()"));
    assert!(registration_source.contains("fn mixed_startup_multi_service_module_lists("));
    assert!(registration_source
        .contains("mixed_startup_multi_service_module_lists(\n        scan.service_names,"));
    let all_immediate_service_index = registration_source
        .find("if scan.immediate_count == pending_services.len()")
        .expect("all-immediate multi-service modules should share owner/startup cache");
    let single_startup_service_index = registration_source
        .find("if scan.immediate_count == 1")
        .expect("single-startup multi-service modules should bypass startup Vec construction");
    assert!(lazy_multi_service_index < all_immediate_service_index);
    assert!(all_immediate_service_index < startup_cache_fill_index);
    assert!(all_immediate_service_index < single_startup_service_index);
    assert!(single_startup_service_index < startup_cache_fill_index);
    let single_startup_helper_index = registration_source
        .find("fn single_startup_multi_service_module_lists(")
        .expect("single-startup list helper should own exact-one startup construction");
    let mixed_startup_helper_index = registration_source
        .find("fn mixed_startup_multi_service_module_lists(")
        .expect("mixed-startup list helper should own two-or-more startup construction");
    assert!(single_startup_service_index < single_startup_helper_index);
    assert!(single_startup_helper_index < mixed_startup_helper_index);
    let single_startup_helper_source =
        &registration_source[single_startup_helper_index..mixed_startup_helper_index];
    assert!(single_startup_helper_source.contains("service_names: Arc<[RegistryName]>"));
    assert!(single_startup_helper_source.contains("startup_service_index: usize"));
    assert!(single_startup_helper_source
        .contains("pending_services[startup_service_index].1.startup_mode"));
    assert!(single_startup_helper_source
        .contains("let startup_service_name = pending_services[startup_service_index].0.clone();"));
    assert!(!single_startup_helper_source.contains("for (name, _) in pending_services"));
    assert!(!single_startup_helper_source
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    assert!(single_startup_helper_source
        .contains("Arc::<[RegistryName]>::from([startup_service_name])"));
    assert!(!single_startup_helper_source.contains("Vec::with_capacity(immediate_count)"));
    assert!(!single_startup_helper_source.contains("if entry.startup_mode"));
    let lazy_helper_source =
        &registration_source[lazy_multi_service_lists_index..single_service_lists_index];
    assert!(lazy_helper_source.contains("service_names: Arc<[RegistryName]>"));
    let all_immediate_helper_index = registration_source
        .find("fn all_immediate_multi_service_module_lists(")
        .expect("all-immediate helper should exist");
    let all_immediate_helper_source =
        &registration_source[all_immediate_helper_index..lazy_multi_service_lists_index];
    assert!(all_immediate_helper_source.contains("service_names: Arc<[RegistryName]>"));
    assert!(!all_immediate_helper_source
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    let mixed_startup_helper_source =
        &registration_source[mixed_startup_helper_index..all_immediate_helper_index];
    assert!(mixed_startup_helper_source.contains("service_names: Arc<[RegistryName]>"));
    assert!(!mixed_startup_helper_source
        .contains("let mut service_names = Vec::with_capacity(pending_services.len())"));
    assert!(registration_source.contains("fn shutdown_service_names_or_owner_clone("));
    assert!(registration_source.contains("fn shutdown_order_matches_owner_order("));
    assert!(registration_source.contains("return owner_service_names.clone();"));
    assert!(registration_source.contains("driver_count == service_count"));
    assert!(registration_source.contains("manager_count == service_count"));
    assert!(registration_source.contains("plugin_count == service_count"));
    assert!(registration_behavior_tests_source
        .contains("fn register_same_kind_multi_service_reuses_owner_cache_for_shutdown_cache()"));
    assert!(registration_source.contains("fn dependency_names("));
    assert!(registration_source.contains("if dependencies.is_empty()"));
    assert!(registration_source.contains("return Arc::default();"));
    assert!(registration_source.contains("if let [dependency] = dependencies"));
    assert!(registration_source
        .contains("return Arc::<[RegistryName]>::from([dependency.name.clone()]);"));
    assert!(
        registration_source.contains("if let [first_dependency, second_dependency] = dependencies")
    );
    assert!(registration_source.contains("first_dependency.name.clone()"));
    assert!(registration_source.contains("second_dependency.name.clone()"));
    assert!(registration_source
        .contains("if let [first_dependency, second_dependency, third_dependency] = dependencies"));
    assert!(registration_source.contains("third_dependency.name.clone()"));
    assert!(registration_source.contains(
        "if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies"
    ));
    assert!(registration_source.contains("fourth_dependency.name.clone()"));
    assert!(registration_source.contains("fifth_dependency.name.clone()"));
    assert!(registration_source.contains("Vec::with_capacity(dependencies.len())"));
    assert!(registration_source.contains("names.push(dependency.name.clone())"));
    assert!(registration_source.contains("dependencies: dependency_names(dependencies)"));
    let empty_dependency_fast_path_index = registration_source
        .find("if dependencies.is_empty()")
        .expect("dependency-name materialization should fast-path empty dependency slices");
    let single_dependency_arc_index = registration_source
        .find("if let [dependency] = dependencies")
        .expect("single dependency names should bypass the Vec-backed dependency list");
    let two_dependency_arc_index = registration_source
        .find("if let [first_dependency, second_dependency] = dependencies")
        .expect("two dependency names should bypass the Vec-backed dependency list");
    let three_dependency_arc_index = registration_source
        .find("if let [first_dependency, second_dependency, third_dependency] = dependencies")
        .expect("three dependency names should bypass the Vec-backed dependency list");
    let four_dependency_arc_index = registration_source
        .find("if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies")
        .expect("four dependency names should bypass the Vec-backed dependency list");
    let five_dependency_arc_index = registration_source
        .find("fifth_dependency.name.clone()")
        .expect("five dependency names should bypass the Vec-backed dependency list");
    let dependency_vec_index = registration_source
        .find("let mut names = Vec::with_capacity(dependencies.len())")
        .expect("six-or-more dependency slices should keep pre-sized materialization");
    assert!(empty_dependency_fast_path_index < single_dependency_arc_index);
    assert!(single_dependency_arc_index < two_dependency_arc_index);
    assert!(two_dependency_arc_index < three_dependency_arc_index);
    assert!(three_dependency_arc_index < four_dependency_arc_index);
    assert!(four_dependency_arc_index < five_dependency_arc_index);
    assert!(five_dependency_arc_index < dependency_vec_index);
    let driver_dependency_validation_index = registration_source
        .find("fn validate_driver_dependencies(")
        .expect("driver dependency validation should have a dedicated helper");
    let driver_dependency_kind_helper_index = registration_source
        .find("fn validate_driver_dependency_kind(")
        .expect("driver dependency kind checking should be split from traversal");
    let driver_dependency_validation_source = &registration_source
        [driver_dependency_validation_index..driver_dependency_kind_helper_index];
    assert!(
        driver_dependency_validation_source
            .find("if dependencies.is_empty()")
            .unwrap()
            < driver_dependency_validation_source
                .find("if let [dependency] = dependencies")
                .unwrap()
    );
    assert!(driver_dependency_validation_source
        .contains("return validate_driver_dependency_kind(kind, name, second_dependency);"));
    assert!(driver_dependency_validation_source
        .contains("if let [first_dependency, second_dependency, third_dependency] = dependencies"));
    assert!(driver_dependency_validation_source
        .contains("return validate_driver_dependency_kind(kind, name, third_dependency);"));
    assert!(driver_dependency_validation_source.contains(
        "if let [first_dependency, second_dependency, third_dependency, fourth_dependency] ="
    ));
    assert!(driver_dependency_validation_source
        .contains("return validate_driver_dependency_kind(kind, name, fourth_dependency);"));
    assert!(driver_dependency_validation_source.contains("fifth_dependency"));
    assert!(driver_dependency_validation_source
        .contains("return validate_driver_dependency_kind(kind, name, fifth_dependency);"));
    let single_driver_dependency_index = driver_dependency_validation_source
        .find("if let [dependency] = dependencies")
        .expect("single driver dependency slices should validate directly");
    let two_driver_dependency_index = driver_dependency_validation_source
        .find("if let [first_dependency, second_dependency] = dependencies")
        .expect("two driver dependency slices should validate directly");
    let three_driver_dependency_index = driver_dependency_validation_source
        .find("if let [first_dependency, second_dependency, third_dependency] = dependencies")
        .expect("three driver dependency slices should validate directly");
    let four_driver_dependency_index = driver_dependency_validation_source
        .find("fourth_dependency")
        .expect("four driver dependency slices should validate directly");
    let five_driver_dependency_index = driver_dependency_validation_source
        .find("fifth_dependency")
        .expect("five driver dependency slices should validate directly");
    let driver_dependency_loop_index = driver_dependency_validation_source
        .find("for dependency in dependencies")
        .expect("six-or-more driver dependency slices should retain the loop");
    assert!(single_driver_dependency_index < two_driver_dependency_index);
    assert!(two_driver_dependency_index < three_driver_dependency_index);
    assert!(three_driver_dependency_index < four_driver_dependency_index);
    assert!(four_driver_dependency_index < five_driver_dependency_index);
    assert!(five_driver_dependency_index < driver_dependency_loop_index);
    assert!(registration_source.contains("debug_assert_eq!("));
    assert!(registration_source.contains("let manager_start = driver_count"));
    assert!(registration_source.contains("let plugin_start = driver_count + manager_count"));
    assert!(registration_source.contains("let plugin_end = plugin_start + plugin_count"));
    assert!(registration_source.contains("fn push_service_names("));
    assert!(registration_source.contains("target: &mut Vec<RegistryName>"));
    assert!(registration_source.contains("for (name, _) in services"));
    assert!(registration_source.contains("target.push(name.clone())"));
    assert!(registration_source.contains("push_shutdown_service_names("));
    assert!(registration_source.contains("pending_services[plugin_start..plugin_end]"));
    assert!(registration_source.contains("pending_services[manager_start..plugin_start]"));
    assert!(registration_source.contains("pending_services[..driver_count]"));
    assert!(pending_prepare_index < commit_lock_index);
    assert!(registration_register_module_source.contains("use std::collections::HashSet;"));
    assert!(registration_duplicates_source.contains("use std::collections::HashMap;"));
    assert!(registration_source.contains("if let Some(duplicate_name) ="));
    assert!(registration_source
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(registration_source.contains("fn duplicate_existing_pending_service_name<'a>("));
    assert!(registration_source.contains("services: &HashMap<RegistryName, ServiceEntry>"));
    assert!(registration_source.contains("debug_assert!(pending_services.len() >= 6);"));
    assert!(registration_duplicates_source.contains("for (name, _) in pending_services"));
    assert!(registration_duplicates_source.contains("if services.contains_key(name)"));
    assert!(registration_duplicates_source.contains("return Some(name);"));
    assert!(registration_duplicates_source.contains("None"));
    assert!(!registration_duplicates_source.contains(".find_map("));
    assert!(!registration_duplicates_source.contains(".then_some("));
    let six_or_more_duplicate_check_index = registration_duplicates_source
        .find("fn duplicate_existing_pending_service_name<'a>(")
        .expect("six-or-more pending services should own the generic duplicate helper");
    let duplicate_helper_assert_index = registration_duplicates_source
        .find("debug_assert!(pending_services.len() >= 6);")
        .expect("generic duplicate helper should only be called for six-or-more services");
    let multi_duplicate_check_index = registration_duplicates_source
        .find("for (name, _) in pending_services")
        .expect("six-or-more service registration should scan pending services directly");
    let multi_duplicate_contains_index = registration_duplicates_source
        .find("if services.contains_key(name)")
        .expect("six-or-more service registration should check the existing service table");
    let multi_duplicate_return_index = registration_duplicates_source
        .find("return Some(name);")
        .expect("six-or-more service registration should return the first pending duplicate");
    assert!(six_or_more_duplicate_check_index < duplicate_helper_assert_index);
    assert!(multi_duplicate_check_index < multi_duplicate_contains_index);
    assert!(multi_duplicate_contains_index < multi_duplicate_return_index);
    assert!(registration_behavior_tests_source
        .contains("fn register_single_service_reports_existing_service_table_key()"));
    assert!(registration_behavior_tests_source.contains(
        "fn register_exact_three_services_reports_existing_third_key_without_partial_commit()"
    ));
    assert!(registration_behavior_tests_source.contains(
        "fn register_exact_four_services_reports_existing_fourth_key_without_partial_commit()"
    ));
    assert!(registration_behavior_tests_source.contains(
        "fn register_exact_five_services_reports_existing_fifth_key_without_partial_commit()"
    ));
    assert!(!registration_duplicates_source.contains("if let [(name, _)] = pending_services"));
    assert!(!registration_duplicates_source
        .contains("if let [(first_name, _), (second_name, _)] = pending_services"));
    assert!(!registration_duplicates_source.contains(
        "if let [(first_name, _), (second_name, _), (third_name, _)] = pending_services"
    ));
    assert!(!registration_duplicates_source.contains("fourth_service_name"));
    assert!(!registration_duplicates_source.contains("fifth_service_name"));
    assert!(registration_behavior_tests_source
        .contains("fn register_lazy_multi_service_keeps_empty_startup_cache()"));
    assert!(!runtime_inner_source.contains("HashMap<String, ServiceEntry>"));
    assert!(!service_entry_source.contains("name: RegistryName"));
    assert!(!service_entry_source.contains("owner_module: String"));
    assert!(!service_entry_source.contains("kind: ServiceKind"));
    assert!(!registration_source.contains("existing_services"));
    assert!(!registration_source.contains("services.contains_key(name.as_str())"));
    assert!(!registration_source.contains("pending_keys.contains(name.as_str())"));
    assert!(!registration_source.contains("HashSet::new()"));
    assert!(!registration_source.contains(".map(|dependency| dependency.name.clone())"));
    assert!(!registration_source.contains(".map(|(name, _)| name.clone())"));
    assert!(!registration_source.contains(".collect::<Vec<_>>()"));
    assert!(!registration_source.contains("fn startup_service_names("));
    assert!(!registration_source.contains("fn shutdown_service_names("));
}
