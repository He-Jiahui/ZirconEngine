#[test]
fn registration_behavior_tests_stay_folder_backed() {
    let behavior_mod_source = include_str!("../behavior.rs");
    let behavior_tests_source = [
        include_str!("../behavior/validation.rs"),
        include_str!("../behavior/cache_lists.rs"),
        include_str!("../behavior/commit.rs"),
        include_str!("../behavior/canonical_keys.rs"),
    ]
    .join("\n");

    assert!(behavior_mod_source.contains("mod validation;"));
    assert!(behavior_mod_source.contains("mod cache_lists;"));
    assert!(behavior_mod_source.contains("mod commit;"));
    assert!(behavior_mod_source.contains("mod canonical_keys;"));
    assert!(!behavior_mod_source.contains("#[test]"));
    assert!(!behavior_mod_source.contains("use "));
    assert!(
        behavior_tests_source.contains("fn register_module_rejects_noncanonical_module_names()")
    );
    assert!(behavior_tests_source
        .contains("fn register_single_immediate_service_keeps_exact_cached_service_lists()"));
    assert!(behavior_tests_source
        .contains("fn register_single_service_reports_existing_service_table_key()"));
    assert!(
        behavior_tests_source.contains("fn service_table_is_keyed_by_canonical_registry_names()")
    );
    assert!(behavior_tests_source
        .contains("fn register_exact_four_dependencies_keeps_direct_dependency_name_cache()"));
    assert!(behavior_tests_source
        .contains("fn register_module_rejects_fourth_driver_dependency_on_manager()"));
    assert!(behavior_tests_source
        .contains("fn register_exact_five_dependencies_keeps_direct_dependency_name_cache()"));
    assert!(behavior_tests_source
        .contains("fn register_module_rejects_fifth_driver_dependency_on_manager()"));
    assert!(behavior_tests_source.contains(
        "fn register_exact_four_services_reports_existing_fourth_key_without_partial_commit()"
    ));
    assert!(behavior_tests_source.contains(
        "fn register_exact_five_services_reports_existing_fifth_key_without_partial_commit()"
    ));

    for (path, source) in [
        (
            "core/runtime/tests/registration/structure/mod.rs",
            include_str!("mod.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/module_layout.rs",
            include_str!("module_layout.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/service_count_paths.rs",
            include_str!("service_count_paths.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/service_list_caches.rs",
            include_str!("service_list_caches.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/dependency_fast_paths.rs",
            include_str!("dependency_fast_paths.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/duplicate_detection.rs",
            include_str!("duplicate_detection.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/cleanup.rs",
            include_str!("cleanup.rs"),
        ),
        (
            "core/runtime/tests/registration/structure/behavior_layout.rs",
            include_str!("behavior_layout.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
