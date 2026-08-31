use super::registration_sources;

#[test]
fn registration_source_preserves_duplicate_detection_boundaries() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();
    let duplicates = sources.duplicates;
    let behavior_tests = sources.behavior_tests.as_str();

    assert!(!sources.register_module.contains("HashSet"));
    assert!(!sources.descriptor_entries.contains("pending_keys"));
    assert!(sources
        .register_module
        .contains("duplicate_pending_service_name(pending_services.as_mut())"));
    assert!(sources
        .commit
        .contains("duplicate_existing_pending_service_name("));
    assert!(duplicates.contains("use std::collections::{HashMap, HashSet};"));
    assert!(registration.contains("if let Some(duplicate_name) ="));
    assert!(registration
        .contains("duplicate_existing_pending_service_name(services, pending_services.as_mut())"));
    assert!(registration.contains("fn duplicate_pending_service_name("));
    assert!(registration.contains("fn duplicate_existing_pending_service_name<'a>("));
    assert!(registration.contains("services: &HashMap<RegistryName, ServiceEntry>"));
    assert!(registration.contains("debug_assert!(!pending_services.is_empty());"));
    assert!(duplicates.contains("const SMALL_PENDING_SERVICE_BATCH: usize = 5;"));
    assert!(duplicates.contains("if pending_services.len() <= SMALL_PENDING_SERVICE_BATCH"));
    assert!(duplicates
        .contains("for (left_index, (left_name, _)) in pending_services.iter().enumerate()"));
    assert!(
        duplicates.contains("for (right_name, _) in pending_services.iter().skip(left_index + 1)")
    );
    assert!(duplicates.contains("let mut seen = HashSet::with_capacity(pending_services.len())"));
    assert!(duplicates.contains("if !seen.insert(name)"));
    assert!(duplicates.contains("for (name, _) in pending_services"));
    assert!(duplicates.contains("if services.contains_key(name)"));
    assert!(duplicates.contains("return Some(name);"));
    assert!(duplicates.contains("None"));
    assert!(!duplicates.contains(".find_map("));
    assert!(!duplicates.contains(".then_some("));
    let pending_duplicate_check_index = sources
        .register_module
        .find("duplicate_pending_service_name(pending_services.as_mut())")
        .expect("the common registration entry should validate the pending batch");
    let module_lock_index = sources
        .register_module
        .rfind("let mut modules = self.lock_modules()")
        .expect("registration should lock the module table after batch validation");
    assert!(pending_duplicate_check_index < module_lock_index);

    let existing_duplicate_check_index = duplicates
        .find("fn duplicate_existing_pending_service_name<'a>(")
        .expect("pending service commits should own one duplicate helper");
    let existing_duplicate_source = &duplicates[existing_duplicate_check_index..];
    let duplicate_helper_assert_index = existing_duplicate_source
        .find("debug_assert!(!pending_services.is_empty());")
        .expect("the duplicate helper should reject empty commit input in debug builds");
    let existing_pending_scan_index = existing_duplicate_source
        .find("for (name, _) in pending_services")
        .expect("service registration should scan pending services directly");
    let pending_duplicate_contains_index = existing_duplicate_source
        .find("if services.contains_key(name)")
        .expect("service registration should check the existing service table");
    let pending_duplicate_return_index = existing_duplicate_source
        .find("return Some(name);")
        .expect("service registration should return the first pending duplicate");
    assert!(duplicate_helper_assert_index < existing_pending_scan_index);
    assert!(existing_pending_scan_index < pending_duplicate_contains_index);
    assert!(pending_duplicate_contains_index < pending_duplicate_return_index);
    assert!(
        behavior_tests.contains("fn register_single_service_reports_existing_service_table_key()")
    );
    assert!(behavior_tests.contains(
        "fn register_exact_three_services_reports_existing_third_key_without_partial_commit()"
    ));
    assert!(behavior_tests.contains(
        "fn register_exact_four_services_reports_existing_fourth_key_without_partial_commit()"
    ));
    assert!(behavior_tests.contains(
        "fn register_exact_five_services_reports_existing_fifth_key_without_partial_commit()"
    ));
    assert!(behavior_tests.contains(
        "fn register_exact_two_services_rejects_duplicate_batch_key_without_partial_commit()"
    ));
    assert!(behavior_tests.contains(
        "fn register_exact_five_services_rejects_first_last_duplicate_without_partial_commit()"
    ));
    assert!(behavior_tests.contains(
        "fn register_six_services_rejects_first_last_duplicate_without_partial_commit()"
    ));
    assert!(behavior_tests
        .contains("fn register_small_batch_reports_the_first_duplicate_key_deterministically()"));
    assert!(!duplicates.contains("if let [(name, _)] = pending_services"));
    assert!(!duplicates.contains("if let [(first_name, _), (second_name, _)] = pending_services"));
    assert!(!duplicates.contains(
        "if let [(first_name, _), (second_name, _), (third_name, _)] = pending_services"
    ));
    assert!(!duplicates.contains("fourth_service_name"));
    assert!(!duplicates.contains("fifth_service_name"));
    assert!(behavior_tests.contains("fn register_lazy_multi_service_keeps_empty_startup_cache()"));
}
