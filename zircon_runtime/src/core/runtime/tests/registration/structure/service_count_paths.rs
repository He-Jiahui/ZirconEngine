use super::registration_sources;

#[test]
fn registration_source_preserves_service_count_fast_paths() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();
    let register_module = sources.register_module;
    let commit = sources.commit;

    assert!(register_module.contains("let driver_count = descriptor.drivers.len()"));
    assert!(register_module.contains("let manager_count = descriptor.managers.len()"));
    assert!(register_module.contains("let plugin_count = descriptor.plugins.len()"));
    assert!(
        register_module.contains("let service_count = driver_count + manager_count + plugin_count")
    );

    let service_count_index = register_module
        .find("let service_count = driver_count + manager_count + plugin_count")
        .expect("registration should derive service count before choosing a path");
    let empty_module_fast_path_index = register_module
        .find("if service_count == 0")
        .expect("registration should fast-path empty module descriptors");
    let non_empty_duplicate_precheck_index = register_module
        .find("let modules = self.lock_modules();")
        .expect("non-empty registration should keep its duplicate-module precheck");
    let single_service_fast_path_index = register_module
        .find("if service_count == 1")
        .expect("single-service registration should bypass pending duplicate storage");
    let two_service_fast_path_index = register_module
        .find("if service_count == 2")
        .expect("two-service registration should bypass pending duplicate storage");
    let three_service_fast_path_index = register_module
        .find("if service_count == 3")
        .expect("three-service registration should bypass pending duplicate storage");
    let four_service_fast_path_index = register_module
        .find("if service_count == 4")
        .expect("four-service registration should bypass pending duplicate storage");
    let five_service_fast_path_index = register_module
        .find("if service_count == 5")
        .expect("five-service registration should bypass pending duplicate storage");
    let pending_services_index = register_module
        .find("let mut pending_services = Vec::with_capacity(service_count)")
        .expect("six-or-more registration should pre-size pending service entries");

    assert!(service_count_index < empty_module_fast_path_index);
    assert!(empty_module_fast_path_index < non_empty_duplicate_precheck_index);
    assert!(non_empty_duplicate_precheck_index < single_service_fast_path_index);
    assert!(single_service_fast_path_index < two_service_fast_path_index);
    assert!(two_service_fast_path_index < three_service_fast_path_index);
    assert!(three_service_fast_path_index < four_service_fast_path_index);
    assert!(four_service_fast_path_index < five_service_fast_path_index);
    assert!(five_service_fast_path_index < pending_services_index);
    assert!(!register_module.contains("pending_keys"));
    assert!(!register_module.contains("HashSet"));

    assert!(registration.contains("fn register_empty_module("));
    assert!(registration.contains("return self.register_empty_module(module_name, descriptor);"));
    assert!(registration.contains("fn register_single_service_module("));
    assert!(registration
        .contains("return self.register_single_service_module(module_name, descriptor);"));
    assert!(registration.contains("fn register_two_service_module("));
    assert!(registration.contains("return self.register_two_service_module("));
    assert!(registration.contains("fn register_three_service_module("));
    assert!(registration.contains("return self.register_three_service_module("));
    assert!(registration.contains("fn register_four_service_module("));
    assert!(registration.contains("return self.register_four_service_module("));
    assert!(registration.contains("fn register_five_service_module("));
    assert!(registration.contains("return self.register_five_service_module("));
    assert!(registration.contains("fn prepare_single_descriptor_service_entry("));
    assert!(registration.contains("fn prepare_two_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_three_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_four_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_five_descriptor_service_entries("));
    assert!(registration.contains("fn prepare_single_service_entry("));
    assert!(registration.contains("fn validate_service_descriptor("));
    assert!(registration.contains("fn service_entry("));
    assert!(register_module.contains("let pending_services = ["));
    assert!(register_module.contains("duplicate_pending_service_name(pending_services.as_mut())"));

    let register_empty_module_start = register_module
        .find("fn register_empty_module(")
        .expect("empty-module registration should have a dedicated commit helper");
    let single_service_registration_start = register_module
        .find("fn register_single_service_module(")
        .expect("single-service registration should have a dedicated setup helper");
    let two_service_registration_start = register_module
        .find("fn register_two_service_module(")
        .expect("two-service registration should have a dedicated setup helper");
    let three_service_registration_start = register_module
        .find("fn register_three_service_module(")
        .expect("three-service registration should have a dedicated setup helper");
    let four_service_registration_start = register_module
        .find("fn register_four_service_module(")
        .expect("four-service registration should have a dedicated setup helper");
    let five_service_registration_start = register_module
        .find("fn register_five_service_module(")
        .expect("five-service registration should have a dedicated setup helper");

    let generic_registration =
        &register_module[pending_services_index..register_empty_module_start];
    assert!(generic_registration.contains("self.commit_module_registration("));
    assert!(!generic_registration.contains("services.insert("));

    let empty_registration =
        &register_module[register_empty_module_start..single_service_registration_start];
    assert!(!empty_registration.contains("self.commit_module_registration("));
    assert!(empty_registration.contains("modules.insert("));

    let single_service_registration =
        &register_module[single_service_registration_start..two_service_registration_start];
    assert!(!single_service_registration.contains("HashSet"));
    assert!(!single_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(single_service_registration.contains("self.commit_module_registration("));

    let two_service_registration =
        &register_module[two_service_registration_start..three_service_registration_start];
    assert!(!two_service_registration.contains("HashSet"));
    assert!(!two_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(two_service_registration.contains("self.commit_module_registration("));

    let three_service_registration =
        &register_module[three_service_registration_start..four_service_registration_start];
    assert!(!three_service_registration.contains("HashSet"));
    assert!(!three_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(three_service_registration.contains("self.commit_module_registration("));

    let four_service_registration =
        &register_module[four_service_registration_start..five_service_registration_start];
    assert!(!four_service_registration.contains("HashSet"));
    assert!(!four_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(four_service_registration.contains("prepare_four_descriptor_service_entries("));
    assert!(four_service_registration.contains("self.commit_module_registration("));

    let five_service_registration = &register_module[five_service_registration_start..];
    assert!(!five_service_registration.contains("HashSet"));
    assert!(!five_service_registration.contains("Vec::with_capacity(service_count)"));
    assert!(five_service_registration.contains("prepare_five_descriptor_service_entries("));
    assert!(five_service_registration.contains("self.commit_module_registration("));

    assert_eq!(
        register_module
            .matches("self.commit_module_registration(")
            .count(),
        6,
        "all non-empty cardinality paths should share one commit owner"
    );

    let module_lock_index = register_module
        .rfind("let mut modules = self.lock_modules()")
        .expect("commit should lock the module table first");
    let module_duplicate_check_index = register_module
        .rfind("if modules.contains_key(&module_name)")
        .expect("commit should recheck the module key while holding the module table lock");
    let service_lock_index = register_module
        .find("let mut services = self.lock_services()")
        .expect("commit should lock the service table second");
    let commit_delegate_index = register_module
        .rfind("commit_prepared_module(")
        .expect("the locked registry tables should be delegated to the commit owner");
    let duplicate_check_index = commit
        .find("duplicate_existing_pending_service_name(")
        .expect("commit should reject existing service names before mutation");
    let identity_assignment_index = commit
        .find("assign_service_indices(")
        .expect("commit should assign identities after duplicate checks");
    let service_insert_index = commit
        .find("for (key, entry) in pending_services")
        .expect("commit should insert every prepared service");
    let module_insert_index = commit
        .find("modules.insert(")
        .expect("commit should publish the module after its services");
    assert!(module_lock_index < module_duplicate_check_index);
    assert!(module_duplicate_check_index < service_lock_index);
    assert!(service_lock_index < commit_delegate_index);
    assert!(duplicate_check_index < identity_assignment_index);
    assert!(identity_assignment_index < service_insert_index);
    assert!(service_insert_index < module_insert_index);
}
