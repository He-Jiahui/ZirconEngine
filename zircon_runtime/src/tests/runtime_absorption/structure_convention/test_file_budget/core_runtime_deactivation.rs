use super::*;

#[test]
fn runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed() {
    let parent = read_runtime_src("core/runtime/tests/activation/behavior/deactivation/blocked.rs");
    let external_dependents = read_runtime_src(
        "core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs",
    );
    let exact_two_three = read_runtime_src(
        "core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs",
    );
    let shutdown_order = read_runtime_src(
        "core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs",
    );
    let exact_four = read_runtime_src(
        "core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs",
    );
    let exact_five_without_index_map = read_runtime_src(
        "core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs",
    );
    let exact_five_dependency_matcher = read_runtime_src(
        "core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs",
    );

    assert_contains_all(
        "core runtime deactivation blocked parent mounts folder-backed children",
        &parent,
        &[
            "mod exact_five_dependency_matcher;",
            "mod exact_five_without_index_map;",
            "mod exact_four_dependency_matcher;",
            "mod exact_two_three_dependency_matcher;",
            "mod external_dependents;",
            "mod shutdown_order;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "blocked.rs should only mount child deactivation-blocked test owners"
    );
    for moved_test in [
        "deactivate_single_service_module_reports_external_dependent",
        "deactivate_exact_two_services_reports_first_shutdown_service_when_dependent_names_both",
        "deactivate_reports_first_blocked_service_in_shutdown_order",
        "deactivate_exact_four_services_reports_first_blocked_without_index_map",
        "deactivate_exact_five_services_reports_first_blocked_without_index_map",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved deactivation blocked test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "external-dependents child owns running dependent blockers",
        &external_dependents,
        &[
            "fn deactivate_single_service_module_reports_external_dependent",
            "fn deactivate_blocks_when_external_dependents_are_alive",
            "LifecycleState::Running",
        ],
    );
    assert_contains_all(
        "exact two/three child owns dependency matcher coverage",
        &exact_two_three,
        &[
            "fn deactivate_exact_two_services_reports_first_shutdown_service_when_dependent_names_both",
            "fn deactivate_exact_three_services_reports_first_shutdown_service_when_dependent_names_all",
            "fn deactivate_exact_three_services_reports_first_blocked_without_index_map",
        ],
    );
    assert_contains_all(
        "shutdown-order child owns first blocked service coverage",
        &shutdown_order,
        &["fn deactivate_reports_first_blocked_service_in_shutdown_order"],
    );
    assert_contains_all(
        "exact-four child owns four-service dependency matcher coverage",
        &exact_four,
        &[
            "fn deactivate_exact_four_services_reports_first_shutdown_service_when_dependent_names_all",
            "fn deactivate_exact_four_services_reports_first_blocked_without_index_map",
        ],
    );
    assert_contains_all(
        "exact-five without-index-map child owns five-service fallback coverage",
        &exact_five_without_index_map,
        &["fn deactivate_exact_five_services_reports_first_blocked_without_index_map"],
    );
    assert_contains_all(
        "existing exact-five child remains mounted",
        &exact_five_dependency_matcher,
        &[
            "fn deactivate_exact_five_services_reports_first_shutdown_service_when_dependent_names_all",
        ],
    );

    let child_test_total = [
        external_dependents.as_str(),
        exact_two_three.as_str(),
        shutdown_order.as_str(),
        exact_four.as_str(),
        exact_five_without_index_map.as_str(),
        exact_five_dependency_matcher.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 10,
        "deactivation blocked children should preserve all 10 tests, including existing exact-five matcher coverage"
    );

    for (path, source) in [
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked.rs",
            parent.as_str(),
        ),
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs",
            external_dependents.as_str(),
        ),
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs",
            exact_two_three.as_str(),
        ),
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs",
            shutdown_order.as_str(),
        ),
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs",
            exact_four.as_str(),
        ),
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs",
            exact_five_without_index_map.as_str(),
        ),
        (
            "core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs",
            exact_five_dependency_matcher.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let lifecycle_doc = read_repo("docs/zircon_runtime/core/runtime/lifecycle.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime lifecycle doc", lifecycle_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 core runtime deactivation blocked test folder split",
                "runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred",
                "core/runtime/tests/activation/behavior/deactivation/blocked.rs",
                "core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs",
                "core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs",
                "runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed",
            ],
        );
    }
}
