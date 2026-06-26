use super::registration_sources;

#[test]
fn registration_source_preserves_service_count_fast_paths() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();
    let register_module = sources.register_module;

    let pending_prepare_index = registration
        .find("let mut pending_services = Vec::with_capacity(")
        .expect("registration should prepare pending service entries before commit");
    let commit_lock_index = registration
        .rfind("let mut modules = self.lock_modules()")
        .expect("registration should retain a final transactional module commit lock");
    assert!(registration.contains("let driver_count = descriptor.drivers.len()"));
    assert!(registration.contains("let manager_count = descriptor.managers.len()"));
    assert!(registration.contains("let plugin_count = descriptor.plugins.len()"));
    assert!(
        registration.contains("let service_count = driver_count + manager_count + plugin_count")
    );
    let service_count_index = registration
        .find("let service_count = driver_count + manager_count + plugin_count")
        .expect("registration should derive service count before choosing registration path");
    let empty_module_fast_path_index = registration
        .find("if service_count == 0")
        .expect("registration should fast-path empty module descriptors");
    let non_empty_duplicate_precheck_index = registration
        .find("let modules = self.lock_modules();")
        .expect("non-empty registration should keep its duplicate-module precheck");
    let pending_services_index = registration
        .find("let mut pending_services = Vec::with_capacity(service_count)")
        .expect("non-empty registration should pre-size pending service entries");
    let pending_keys_index = registration
        .find("let mut pending_keys = HashSet::with_capacity(service_count)")
        .expect("non-empty registration should pre-size pending duplicate keys");
    let single_service_fast_path_index = registration
        .find("if service_count == 1")
        .expect("single-service registration should bypass pending duplicate storage");
    let two_service_fast_path_index = registration
        .find("if service_count == 2")
        .expect("two-service registration should bypass pending duplicate storage");
    let three_service_fast_path_index = registration
        .find("if service_count == 3")
        .expect("three-service registration should bypass pending duplicate storage");
    let four_service_fast_path_index = registration
        .find("if service_count == 4")
        .expect("four-service registration should bypass pending duplicate storage");
    let five_service_fast_path_index = registration
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
    assert!(registration.contains("fn register_empty_module("));
    assert!(registration.contains("return self.register_empty_module(module_name, descriptor);"));
    assert!(registration
        .contains("return self.register_single_service_module(module_name, descriptor);"));
    assert!(registration.contains("fn register_single_service_module("));
    assert!(registration.contains("return self.register_two_service_module("));
    assert!(registration.contains("fn register_two_service_module("));
    assert!(registration.contains("return self.register_three_service_module("));
    assert!(registration.contains("fn register_three_service_module("));
    assert!(registration.contains("return self.register_four_service_module("));
    assert!(registration.contains("fn register_four_service_module("));
    assert!(registration.contains("return self.register_five_service_module("));
    assert!(registration.contains("fn register_five_service_module("));
    assert!(registration.contains("fn prepare_single_descriptor_service_entry("));
    assert!(registration.contains("fn prepare_two_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_three_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_four_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_five_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_single_service_entry("));
    assert!(registration.contains("fn validate_service_descriptor("));
    assert!(registration.contains("fn service_entry("));
    assert!(registration.contains("if services.contains_key(&service_name)"));
    assert!(registration.contains("services.insert(service_name, service_entry);"));
    assert!(registration.contains("let pending_services = ["));
    assert!(registration.contains("if first_service_name == second_service_name"));

    let two_service_registration_start = register_module
        .find("fn register_two_service_module(")
        .expect("two-service registration should have a dedicated setup helper");
    let register_empty_module_start = registration
        .find("fn register_empty_module(")
        .expect("empty-module registration should have a dedicated commit helper");
    let three_service_registration_start = register_module
        .find("fn register_three_service_module(")
        .expect("three-service registration should have a dedicated setup helper");
    let two_service_registration =
        &register_module[two_service_registration_start..three_service_registration_start];
    assert!(!two_service_registration.contains("HashSet"));
    assert!(!two_service_registration.contains("Vec::with_capacity(service_count)"));
    let four_service_registration_start = register_module
        .find("fn register_four_service_module(")
        .expect("four-service registration should have a dedicated setup helper");
    let five_service_registration_start = register_module
        .find("fn register_five_service_module(")
        .expect("five-service registration should have a dedicated setup helper");
    let three_service_registration =
        &register_module[three_service_registration_start..four_service_registration_start];
    assert!(!three_service_registration.contains("HashSet"));
    assert!(!three_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(!three_service_registration
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(!three_service_registration.contains("for (key, entry) in pending_services"));
    assert!(three_service_registration
        .contains("services.insert(first_service_name, first_service_entry);"));
    assert!(three_service_registration
        .contains("services.insert(second_service_name, second_service_entry);"));
    assert!(three_service_registration
        .contains("services.insert(third_service_name, third_service_entry);"));
    let four_service_registration =
        &register_module[four_service_registration_start..five_service_registration_start];
    assert!(!four_service_registration.contains("HashSet"));
    assert!(!four_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(!four_service_registration
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(!four_service_registration.contains("for (key, entry) in pending_services"));
    assert!(four_service_registration.contains("prepare_four_descriptor_service_entries("));
    assert!(four_service_registration
        .contains("services.insert(fourth_service_name, fourth_service_entry);"));
    let five_service_registration = &register_module[five_service_registration_start..];
    assert!(!five_service_registration.contains("HashSet"));
    assert!(!five_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(!five_service_registration
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(!five_service_registration.contains("for (key, entry) in pending_services"));
    assert!(five_service_registration.contains("prepare_five_descriptor_service_entries("));
    assert!(five_service_registration
        .contains("services.insert(fifth_service_name, fifth_service_entry);"));
    let generic_registration = &registration[pending_services_index..register_empty_module_start];
    assert!(generic_registration
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(generic_registration.contains("for (key, entry) in pending_services"));
    assert!(
        !generic_registration.contains("services.insert(first_service_name, first_service_entry);")
    );
    assert!(pending_prepare_index < commit_lock_index);
}
