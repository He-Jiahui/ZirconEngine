use super::*;

const SLICE: &str = "Runtime 15 M3 root entries guard child-owner split";
const STATUS: &str = "runtime_15_root_entries_guard_child_owner_split_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_root_entries_guard_child_owners_are_folder_backed";

#[test]
fn runtime_15_root_entries_guard_child_owners_are_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/root_entries.rs");
    let runtime_root = read_runtime_src("tests/runtime_absorption/root_entries/runtime_root.rs");
    let core_spine = read_runtime_src("tests/runtime_absorption/root_entries/core_spine.rs");
    let module_families =
        read_runtime_src("tests/runtime_absorption/root_entries/module_families.rs");
    let module_family_navigation =
        read_runtime_src("tests/runtime_absorption/root_entries/module_families/navigation.rs");
    let module_family_animation_backlog = read_runtime_src(
        "tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
    );
    let module_family_animation_status_json = read_runtime_src(
        "tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
    );
    let module_family_root_seats =
        read_runtime_src("tests/runtime_absorption/root_entries/module_families/root_seats.rs");
    let module_family_mirror_docs =
        read_runtime_src("tests/runtime_absorption/root_entries/module_families/mirror_docs.rs");
    let module_family_split_layout =
        read_runtime_src("tests/runtime_absorption/root_entries/module_families/split_layout.rs");
    let core_spine_mirror =
        read_runtime_src("tests/runtime_absorption/core_spine_root_generated/mirror_docs.rs");
    let core_spine_audit = read_repo(
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py",
    );
    let module_family_audit = read_repo(
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation/lock_poison.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation/lock_poison.rs",
    );

    assert_contains_all(
        "root_entries parent stays structural",
        &parent,
        &[
            "#[path = \"root_entries/core_spine.rs\"]",
            "mod core_spine;",
            "#[path = \"root_entries/module_families.rs\"]",
            "mod module_families;",
            "#[path = \"root_entries/runtime_root.rs\"]",
            "mod runtime_root;",
        ],
    );

    for moved_guard in [
        "builtin_root_stays_structural_after_runtime_module_split",
        "core_root_retires_channel_and_service_alias_fragments",
        "runtime_navigation_boundary_file_set_requires_doc_update",
        "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "root_entries.rs should only mount child owners; moved guard `{moved_guard}` should not return"
        );
    }

    assert_contains_all(
        "runtime-root child keeps root-surface guards",
        &runtime_root,
        &[
            "builtin_root_stays_structural_after_runtime_module_split",
            "runtime_crate_root_does_not_flatten_plugin_surface",
            "runtime_crate_root_does_not_flatten_builtin_module_assembly_functions",
        ],
    );
    assert_contains_all(
        "core-spine child keeps core root guards",
        &core_spine,
        &[
            "core_root_retires_channel_and_service_alias_fragments",
            "core_root_retires_runtime_kernel_fragment_files",
            "core_root_splits_event_dto_from_runtime_event_bus",
            "core_root_reexports_runtime_diagnostics_without_root_directory",
            "core_module_tree_matches_decided_spine_shape",
        ],
    );
    assert_contains_all(
        "module-family child keeps Runtime 14 guards",
        &format!(
            "{module_families}\n{module_family_navigation}\n{module_family_animation_backlog}\n{module_family_animation_status_json}\n{module_family_root_seats}\n{module_family_mirror_docs}\n{module_family_split_layout}"
        ),
        &[
            "runtime_navigation_boundary_file_set_requires_doc_update",
            "runtime_animation_backlog_boundary_requires_doc_update",
            "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
            "runtime_14_module_family_root_seats_match_documented_judgements",
            "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
            "runtime_15_root_entries_module_families_guard_is_folder_backed",
        ],
    );

    for (path, source) in [
        ("tests/runtime_absorption/root_entries.rs", parent.as_str()),
        (
            "tests/runtime_absorption/root_entries/runtime_root.rs",
            runtime_root.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/core_spine.rs",
            core_spine.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families.rs",
            module_families.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families/navigation.rs",
            module_family_navigation.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
            module_family_animation_backlog.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
            module_family_animation_status_json.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families/root_seats.rs",
            module_family_root_seats.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families/mirror_docs.rs",
            module_family_mirror_docs.as_str(),
        ),
        (
            "tests/runtime_absorption/root_entries/module_families/split_layout.rs",
            module_family_split_layout.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the root_entries test guard owner budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Rust Runtime 02 mirror aggregates root_entries children",
        &core_spine_mirror,
        &[
            "root_entries/core_spine.rs",
            "root_entries/module_families.rs",
            "root_entries/runtime_root.rs",
            "rust_test_count_in_files",
        ],
    );
    assert_contains_all(
        "Python Runtime 02 audit aggregates root_entries children",
        &core_spine_audit,
        &[
            "ROOT_ENTRIES_GUARD_RELATIVES",
            "root_entries/core_spine.rs",
            "root_entries/module_families.rs",
            "root_entries/runtime_root.rs",
            "root_entries_sources",
        ],
    );
    assert_contains_all(
        "Python Runtime 14 audit aggregates root_entries children",
        &module_family_audit,
        &[
            "ROOT_ENTRIES_GUARD_RELATIVES",
            "root_entries/core_spine.rs",
            "root_entries/module_families.rs",
            "root_entries/runtime_root.rs",
            "root_entries_files",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status row data", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD]);
    }
    assert_contains_all("status map", &status_map, &[SLICE, STATUS]);
    assert_contains_all("date map", &date_map, &[SLICE, "2026-06-24"]);
}
