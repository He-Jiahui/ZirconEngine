use super::{FRAMEWORKS_STATUS, GUARD, SLICE, STATUS};

#[test]
fn runtime_15_root_entries_module_families_guard_is_folder_backed() {
    let route = include_str!("../module_families.rs");
    let navigation = include_str!("navigation.rs");
    let animation_backlog = include_str!("animation_backlog.rs");
    let animation_status_json = include_str!("animation_status_json.rs");
    let root_seats = include_str!("root_seats.rs");
    let mirror_docs = include_str!("mirror_docs.rs");
    let split_layout = include_str!("split_layout.rs");
    let children = [
        navigation,
        animation_backlog,
        animation_status_json,
        root_seats,
        mirror_docs,
        split_layout,
    ]
    .join("\n");

    assert_contains_all(
        "module_families route mounts children",
        route,
        &[
            "#[path = \"module_families/animation_backlog.rs\"]",
            "#[path = \"module_families/animation_status_json.rs\"]",
            "#[path = \"module_families/mirror_docs.rs\"]",
            "#[path = \"module_families/navigation.rs\"]",
            "#[path = \"module_families/root_seats.rs\"]",
            "#[path = \"module_families/split_layout.rs\"]",
        ],
    );

    for moved_guard in [
        "runtime_navigation_boundary_file_set_requires_doc_update",
        "runtime_animation_backlog_boundary_requires_doc_update",
        "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
        "runtime_14_module_family_root_seats_match_documented_judgements",
        "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
    ] {
        assert!(
            !route.contains(&format!("fn {moved_guard}")),
            "module_families route should not retain `{moved_guard}`"
        );
        assert!(
            children.contains(&format!("fn {moved_guard}")),
            "module_families children should retain `{moved_guard}`"
        );
    }

    for (label, source, max_lines) in [
        ("module_families route", route, 40usize),
        ("navigation child", navigation, 90),
        ("animation backlog child", animation_backlog, 60),
        ("animation status JSON child", animation_status_json, 90),
        ("root seats child", root_seats, 80),
        ("mirror docs child", mirror_docs, 130),
        ("split layout child", split_layout, 220),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected <= {max_lines}"
        );
    }

    assert_contains_all(
        "core spine generated mirror docs routes module family children",
        include_str!("../../core_spine_root_generated/mirror_docs.rs"),
        &[
            "root_entries/module_families/navigation.rs",
            "root_entries/module_families/animation_backlog.rs",
            "root_entries/module_families/animation_status_json.rs",
            "root_entries/module_families/root_seats.rs",
            "root_entries/module_families/mirror_docs.rs",
        ],
    );
    assert_contains_all(
        "Runtime 02 audit script routes module family children",
        include_str!(
            "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py"
        ),
        &[
            "root_entries/module_families/navigation.rs",
            "root_entries/module_families/animation_backlog.rs",
            "root_entries/module_families/animation_status_json.rs",
            "root_entries/module_families/root_seats.rs",
            "root_entries/module_families/mirror_docs.rs",
        ],
    );
    assert_contains_all(
        "Runtime 14 audit script routes module family children",
        include_str!(
            "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py"
        ),
        &[
            "root_entries/module_families/navigation.rs",
            "root_entries/module_families/animation_backlog.rs",
            "root_entries/module_families/animation_status_json.rs",
            "root_entries/module_families/root_seats.rs",
            "root_entries/module_families/mirror_docs.rs",
        ],
    );

    assert_status_docs_mirror_split();
}

fn assert_status_docs_mirror_split() {
    for (label, source) in [
        (
            "Runtime 15 plan",
            crate::tests::runtime_absorption::current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT,
        ),
        (
            "runtime index",
            include_str!(
                "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
            ),
        ),
        (
            "Frameworks 02",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
            ),
        ),
        (
            "structure convention",
            include_str!(
                "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
            ),
        ),
        (
            "review findings",
            include_str!(
                "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
            ),
        ),
        (
            "module convention",
            include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "status row data",
            include_str!(
                "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests/root_route_rows.rs"
            ),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD]);
    }
    assert_contains_all(
        "Frameworks 02",
        include_str!(
            "../../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
        ),
        &[FRAMEWORKS_STATUS],
    );
    assert_contains_all(
        "status map",
        include_str!(
            "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation/lock_poison.rs"
        ),
        &[SLICE, STATUS],
    );
    assert_contains_all(
        "date map",
        include_str!(
            "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation/lock_poison.rs"
        ),
        &[SLICE, "2026-07-06"],
    );
}

fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(source.contains(anchor), "{label} should contain `{anchor}`");
    }
}
