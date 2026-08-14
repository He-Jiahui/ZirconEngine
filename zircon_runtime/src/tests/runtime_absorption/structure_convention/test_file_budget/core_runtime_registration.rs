use super::super::support::assert_contains_all_exact;
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
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-17-registration-filter-plan-anchor-current-owner.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let lifecycle_doc = read_repo("docs/zircon_runtime/core/runtime/lifecycle.md");

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

    assert_contains_all_exact(
        "Runtime 15 registration-filter current child owner",
        &current_anchor_owner,
        &[
            "Runtime 15 M3 core runtime registration structure owner split",
            "runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred",
            "core/runtime/tests/registration/structure/mod.rs",
            "core/runtime/tests/registration/structure/service_count_paths.rs",
            "core/runtime/tests/registration/structure/service_list_caches.rs",
            "runtime_15_core_runtime_registration_structure_tests_are_folder_backed",
            "2026-06-24",
        ],
    );
    assert_contains_all_exact(
        "module convention keeps the core runtime registration folder contract",
        &module_doc,
        &[
            "Runtime 15 M3 core runtime registration structure owner split",
            "runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred",
            "core/runtime/tests/registration/structure/mod.rs",
        ],
    );
    assert_contains_all_exact(
        "core runtime lifecycle keeps the registration structure contract",
        &lifecycle_doc,
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
