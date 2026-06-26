use super::*;

const REGISTRATION_STRUCTURE_TEST_FILES: &[&str] = &[
    "core/runtime/tests/registration/structure/mod.rs",
    "core/runtime/tests/registration/structure/behavior_layout.rs",
    "core/runtime/tests/registration/structure/module_layout.rs",
    "core/runtime/tests/registration/structure/service_count_paths.rs",
    "core/runtime/tests/registration/structure/service_list_caches.rs",
    "core/runtime/tests/registration/structure/dependency_fast_paths.rs",
    "core/runtime/tests/registration/structure/duplicate_detection.rs",
    "core/runtime/tests/registration/structure/cleanup.rs",
];

#[test]
fn runtime_15_core_runtime_registration_structure_tests_are_folder_backed() {
    let registration_parent = read_runtime_src("core/runtime/tests/registration/mod.rs");
    let structure_mod = read_runtime_src("core/runtime/tests/registration/structure/mod.rs");
    let behavior_layout =
        read_runtime_src("core/runtime/tests/registration/structure/behavior_layout.rs");
    let service_count_paths =
        read_runtime_src("core/runtime/tests/registration/structure/service_count_paths.rs");
    let service_list_caches =
        read_runtime_src("core/runtime/tests/registration/structure/service_list_caches.rs");
    let retired_flat = runtime_src_path("core/runtime/tests/registration/structure.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let lifecycle_doc = read_repo("docs/zircon_runtime/core/runtime/lifecycle.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert!(
        !retired_flat.exists(),
        "core runtime registration structure tests should stay folder-backed, not return to {:?}",
        retired_flat
    );
    assert_contains_all(
        "core runtime registration test parent mounts structure owner",
        &registration_parent,
        &["mod behavior;", "mod structure;"],
    );
    assert_contains_all(
        "registration structure parent mounts focused child owners",
        &structure_mod,
        &[
            "mod behavior_layout;",
            "mod cleanup;",
            "mod dependency_fast_paths;",
            "mod duplicate_detection;",
            "mod module_layout;",
            "mod service_count_paths;",
            "mod service_list_caches;",
            "pub(super) fn registration_sources()",
        ],
    );
    assert_contains_all(
        "service-count child owns commit-boundary structure checks",
        &service_count_paths,
        &[
            "fn registration_source_preserves_service_count_fast_paths",
            ".rfind(\"let mut modules = self.lock_modules()\")",
            ".find(\"let modules = self.lock_modules();\")",
        ],
    );
    assert_contains_all(
        "service-list child owns cache-list structure checks",
        &service_list_caches,
        &[
            "fn registration_source_preserves_service_list_cache_paths",
            "fn lazy_multi_service_module_lists(",
            "fn single_startup_multi_service_module_lists(",
        ],
    );
    assert_contains_all(
        "registration behavior layout guard tracks all structure children",
        &behavior_layout,
        &[
            "core/runtime/tests/registration/structure/mod.rs",
            "core/runtime/tests/registration/structure/service_count_paths.rs",
            "core/runtime/tests/registration/structure/service_list_caches.rs",
            "core/runtime/tests/registration/structure/dependency_fast_paths.rs",
            "core/runtime/tests/registration/structure/duplicate_detection.rs",
            "core/runtime/tests/registration/structure/cleanup.rs",
        ],
    );

    for path in REGISTRATION_STRUCTURE_TEST_FILES {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("core runtime lifecycle doc", lifecycle_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 core runtime registration structure owner split",
                "runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred",
                "core/runtime/tests/registration/structure/mod.rs",
                "core/runtime/tests/registration/structure/service_count_paths.rs",
                "core/runtime/tests/registration/structure/service_list_caches.rs",
                "runtime_15_core_runtime_registration_structure_tests_are_folder_backed",
            ],
        );
    }
}
