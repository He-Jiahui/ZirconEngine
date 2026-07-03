use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_core_runtime_service_lists_are_folder_backed() {
    let old_flat_path =
        super::runtime_src_path("core/runtime/handle/registration/service_lists.rs");
    let registration_mod = read_runtime_src("core/runtime/handle/registration/mod.rs");
    let register_module = read_runtime_src("core/runtime/handle/registration/register_module.rs");
    let parent = read_runtime_src("core/runtime/handle/registration/service_lists/mod.rs");
    let types = read_runtime_src("core/runtime/handle/registration/service_lists/types.rs");
    let multi = read_runtime_src("core/runtime/handle/registration/service_lists/multi.rs");
    let specialized =
        read_runtime_src("core/runtime/handle/registration/service_lists/specialized.rs");
    let shutdown = read_runtime_src("core/runtime/handle/registration/service_lists/shutdown.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let lifecycle_doc = read_repo("docs/zircon_runtime/core/runtime/lifecycle.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert!(
        !old_flat_path.exists(),
        "core runtime service-list behavior should live in the folder-backed service_lists owner, not the retired flat service_lists.rs"
    );
    assert_contains_all(
        "registration module keeps service-list owner mounted",
        &registration_mod,
        &["mod service_lists;", "mod register_module;"],
    );
    assert_contains_all(
        "register_module keeps the same narrow service-list entry points",
        &register_module,
        &[
            "use super::service_lists::{module_service_lists, single_service_module_lists};",
            "module_service_lists(&pending_services, driver_count, manager_count, plugin_count)",
            "single_service_module_lists(&service_name, &service_entry)",
        ],
    );
    assert_contains_all(
        "service-list parent stays structural",
        &parent,
        &[
            "mod multi;",
            "mod shutdown;",
            "mod specialized;",
            "mod types;",
            "pub(super) fn module_service_lists",
            "pub(super) use self::specialized::single_service_module_lists;",
        ],
    );
    assert_contains_all(
        "service-list types owner owns only the returned name lists",
        &types,
        &[
            "pub(in crate::core::runtime::handle::registration) struct ModuleServiceLists",
            "service_names: Arc<[RegistryName]>",
            "startup_service_names: Arc<[RegistryName]>",
            "shutdown_service_names: Arc<[RegistryName]>",
        ],
    );
    assert_contains_all(
        "multi-service owner owns the generic scan path",
        &multi,
        &[
            "pub(super) struct MultiServiceListScan",
            "pub(super) fn scan_multi_service_module_lists",
            "pub(super) fn single_startup_multi_service_module_lists",
            "pub(super) fn mixed_startup_multi_service_module_lists",
            "shutdown_service_names_or_owner_clone",
        ],
    );
    assert_contains_all(
        "specialized service-list owner owns one-through-five service paths",
        &specialized,
        &[
            "pub(in crate::core::runtime::handle::registration) fn single_service_module_lists",
            "pub(super) fn two_service_module_lists",
            "pub(super) fn three_service_module_lists",
            "pub(super) fn four_service_module_lists",
            "pub(super) fn five_service_module_lists",
        ],
    );
    assert_contains_all(
        "shutdown service-list owner owns shutdown ordering",
        &shutdown,
        &[
            "pub(super) fn shutdown_service_names_or_owner_clone",
            "fn shutdown_order_matches_owner_order",
            "fn push_shutdown_service_names",
            "shutdown keeps the inverse plugin, manager, driver lifecycle order",
        ],
    );

    for (path, source) in [
        (
            "core/runtime/handle/registration/service_lists/mod.rs",
            parent.as_str(),
        ),
        (
            "core/runtime/handle/registration/service_lists/types.rs",
            types.as_str(),
        ),
        (
            "core/runtime/handle/registration/service_lists/multi.rs",
            multi.as_str(),
        ),
        (
            "core/runtime/handle/registration/service_lists/specialized.rs",
            specialized.as_str(),
        ),
        (
            "core/runtime/handle/registration/service_lists/shutdown.rs",
            shutdown.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime lifecycle doc", lifecycle_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 core runtime service-list owner split",
                "runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked",
                "core/runtime/handle/registration/service_lists/mod.rs",
                "core/runtime/handle/registration/service_lists/specialized.rs",
                "runtime_15_core_runtime_service_lists_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 core runtime service-list owner split",
            "runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked",
            "core/runtime/handle/registration/service_lists/mod.rs",
            "core/runtime/handle/registration/service_lists/specialized.rs",
            "runtime_15_core_runtime_service_lists_are_folder_backed",
        ],
    );
}

#[test]
fn runtime_15_production_file_budget_core_runtime_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/production_file_budget.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/production_file_budget/core_runtime_service_lists.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );

    assert_contains_all(
        "production-file budget parent mounts core runtime child owner",
        &parent,
        &[
            "mod core_runtime_service_lists;",
            "mod material_asset;",
            "mod scene_world_project_io;",
            "mod scene_world_property_access;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_core_runtime_service_lists_are_folder_backed"),
        "production_file_budget.rs should mount the core runtime service-list guard instead of defining it"
    );
    assert_contains_all(
        "core runtime child owns production-file budget service-list guards",
        &child,
        &[
            "fn runtime_15_core_runtime_service_lists_are_folder_backed",
            "fn runtime_15_production_file_budget_core_runtime_guard_is_child_owner",
            "core/runtime/handle/registration/service_lists/mod.rs",
            "core/runtime/handle/registration/service_lists/specialized.rs",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/production_file_budget.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/production_file_budget/core_runtime_service_lists.rs",
            child.as_str(),
        ),
    ] {
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
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 production file budget core runtime guard split",
                "runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred",
                "structure_convention/production_file_budget.rs",
                "structure_convention/production_file_budget/core_runtime_service_lists.rs",
                "runtime_15_production_file_budget_core_runtime_guard_is_child_owner",
            ],
        );
    }
}
