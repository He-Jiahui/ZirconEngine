use super::registration_sources;

#[test]
fn registration_source_preserves_duplicate_detection_boundaries() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();
    let duplicates = sources.duplicates;
    let behavior_tests = sources.behavior_tests.as_str();

    assert!(sources
        .register_module
        .contains("use std::collections::HashSet;"));
    assert!(duplicates.contains("use std::collections::HashMap;"));
    assert!(registration.contains("if let Some(duplicate_name) ="));
    assert!(registration
        .contains("duplicate_existing_pending_service_name(&services, &pending_services)"));
    assert!(registration.contains("fn duplicate_existing_pending_service_name<'a>("));
    assert!(registration.contains("services: &HashMap<RegistryName, ServiceEntry>"));
    assert!(registration.contains("debug_assert!(pending_services.len() >= 6);"));
    assert!(duplicates.contains("for (name, _) in pending_services"));
    assert!(duplicates.contains("if services.contains_key(name)"));
    assert!(duplicates.contains("return Some(name);"));
    assert!(duplicates.contains("None"));
    assert!(!duplicates.contains(".find_map("));
    assert!(!duplicates.contains(".then_some("));
    let six_or_more_duplicate_check_index = duplicates
        .find("fn duplicate_existing_pending_service_name<'a>(")
        .expect("six-or-more pending services should own the generic duplicate helper");
    let duplicate_helper_assert_index = duplicates
        .find("debug_assert!(pending_services.len() >= 6);")
        .expect("generic duplicate helper should only be called for six-or-more services");
    let multi_duplicate_check_index = duplicates
        .find("for (name, _) in pending_services")
        .expect("six-or-more service registration should scan pending services directly");
    let multi_duplicate_contains_index = duplicates
        .find("if services.contains_key(name)")
        .expect("six-or-more service registration should check the existing service table");
    let multi_duplicate_return_index = duplicates
        .find("return Some(name);")
        .expect("six-or-more service registration should return the first pending duplicate");
    assert!(six_or_more_duplicate_check_index < duplicate_helper_assert_index);
    assert!(multi_duplicate_check_index < multi_duplicate_contains_index);
    assert!(multi_duplicate_contains_index < multi_duplicate_return_index);
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
    assert!(!duplicates.contains("if let [(name, _)] = pending_services"));
    assert!(!duplicates.contains("if let [(first_name, _), (second_name, _)] = pending_services"));
    assert!(!duplicates.contains(
        "if let [(first_name, _), (second_name, _), (third_name, _)] = pending_services"
    ));
    assert!(!duplicates.contains("fourth_service_name"));
    assert!(!duplicates.contains("fifth_service_name"));
    assert!(behavior_tests.contains("fn register_lazy_multi_service_keeps_empty_startup_cache()"));
}
