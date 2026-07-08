use super::{
    INVENTORY_CHILD_PATHS, INVENTORY_FRAMEWORKS_STATUS, INVENTORY_GUARD, INVENTORY_PARENT_PATH,
    INVENTORY_SLICE, INVENTORY_STATUS,
};

#[test]
fn runtime_15_input_stack_inventory_guard_is_folder_backed() {
    let parent = include_str!("../inventory.rs");
    let module_sets = include_str!("module_sets.rs");
    let public_surface = include_str!("public_surface.rs");
    let guard_anchors = include_str!("guard_anchors.rs");
    let behavior_anchors = include_str!("behavior_anchors.rs");
    let cursor_host_requests = include_str!("cursor_host_requests.rs");
    let mirror_docs = include_str!("mirror_docs.rs");
    let split_layout = include_str!("split_layout.rs");
    let children = format!(
        "{module_sets}\n{public_surface}\n{guard_anchors}\n{behavior_anchors}\n{cursor_host_requests}\n{mirror_docs}\n{split_layout}"
    );

    assert_contains_all(
        "input-stack inventory parent routes child owners",
        parent,
        &[
            "mod module_sets;",
            "mod public_surface;",
            "mod guard_anchors;",
            "mod behavior_anchors;",
            "mod cursor_host_requests;",
            "mod mirror_docs;",
            "mod split_layout;",
        ],
    );

    for moved_anchor in [
        "EXPECTED_INPUT_RUNTIME_MODULES",
        "CursorGrabMode",
        "runtime_12_input_stack_contracts_stay_documented_and_exported",
        "EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS",
        "CursorHostRequest",
        "input_stack_boundary",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "input-stack inventory parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.contains(moved_anchor),
            "input-stack inventory children should own moved owner `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (INVENTORY_PARENT_PATH, parent, 40usize),
        (INVENTORY_CHILD_PATHS[0], module_sets, 110),
        (INVENTORY_CHILD_PATHS[1], public_surface, 70),
        (INVENTORY_CHILD_PATHS[2], guard_anchors, 60),
        (INVENTORY_CHILD_PATHS[3], behavior_anchors, 75),
        (INVENTORY_CHILD_PATHS[4], cursor_host_requests, 75),
        (INVENTORY_CHILD_PATHS[5], mirror_docs, 90),
        (INVENTORY_CHILD_PATHS[6], split_layout, 190),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs"
    );
    assert_contains_all(
        "module-convention row data records input-stack inventory split",
        row_data,
        &[
            INVENTORY_SLICE,
            INVENTORY_STATUS,
            INVENTORY_PARENT_PATH,
            INVENTORY_CHILD_PATHS[0],
            INVENTORY_CHILD_PATHS[6],
            INVENTORY_GUARD,
        ],
    );

    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs"
    );
    assert_contains_all(
        "structure route status map records input-stack inventory split",
        status_map,
        &[INVENTORY_SLICE, INVENTORY_STATUS],
    );

    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs"
    );
    assert_contains_all(
        "structure route date map records input-stack inventory split",
        date_map,
        &[INVENTORY_SLICE, "2026-07-06"],
    );

    for (label, source) in [
        (
            "Runtime 15 subplan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "frameworks plan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
            ),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!(
                "../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
            ),
        ),
        (
            "module convention doc",
            include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!(
                "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                INVENTORY_SLICE,
                INVENTORY_STATUS,
                INVENTORY_FRAMEWORKS_STATUS,
                INVENTORY_GUARD,
                INVENTORY_PARENT_PATH,
                INVENTORY_CHILD_PATHS[0],
                INVENTORY_CHILD_PATHS[6],
            ],
        );
    }
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .copied()
        .filter(|needle| !source.contains(needle))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing expected anchors:\n{}",
        missing.join("\n")
    );
}
