const SLICE: &str = "Runtime 15 M3 input-stack absorption guard folder-backed split";
const STATUS: &str =
    "runtime_15_input_stack_absorption_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_input_stack_absorption_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_input_stack_absorption_guard_is_folder_backed";

const PARENT_PATH: &str = "input_stack.rs";
const CHILD_PATHS: &[&str] = &[
    "input_stack/action_mapping.rs",
    "input_stack/contracts.rs",
    "input_stack/gamepad_bridge.rs",
    "input_stack/inventory.rs",
    "input_stack/split_layout.rs",
    "input_stack/support.rs",
    "input_stack/inventory/module_sets.rs",
    "input_stack/inventory/public_surface.rs",
    "input_stack/inventory/guard_anchors.rs",
    "input_stack/inventory/behavior_anchors.rs",
    "input_stack/inventory/cursor_host_requests.rs",
    "input_stack/inventory/mirror_docs.rs",
    "input_stack/inventory/split_layout.rs",
];

#[test]
fn runtime_15_input_stack_absorption_guard_is_folder_backed() {
    let parent = include_str!("../input_stack.rs");
    let children = [
        include_str!("action_mapping.rs"),
        include_str!("contracts.rs"),
        include_str!("gamepad_bridge.rs"),
        include_str!("inventory.rs"),
        include_str!("split_layout.rs"),
        include_str!("support.rs"),
        include_str!("inventory/module_sets.rs"),
        include_str!("inventory/public_surface.rs"),
        include_str!("inventory/guard_anchors.rs"),
        include_str!("inventory/behavior_anchors.rs"),
        include_str!("inventory/cursor_host_requests.rs"),
        include_str!("inventory/mirror_docs.rs"),
        include_str!("inventory/split_layout.rs"),
    ];

    assert_contains_all(
        "input-stack parent routes child owners",
        parent,
        &[
            "mod action_mapping;",
            "mod contracts;",
            "mod gamepad_bridge;",
            "mod inventory;",
            "mod split_layout;",
            "mod support;",
        ],
    );

    for moved_anchor in [
        "EXPECTED_INPUT_RUNTIME_MODULES",
        "runtime_12_input_stack_contracts_stay_documented_and_exported",
        "runtime_12_action_mapping_keeps_ui_filtered_evaluation_path",
        "runtime_12_gamepad_bridge_keeps_runtime_abi_path",
        "assert_owner_files",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "input-stack parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.iter().any(|source| source.contains(moved_anchor)),
            "input-stack children should own moved owner `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 30usize),
        (CHILD_PATHS[0], children[0], 180),
        (CHILD_PATHS[1], children[1], 90),
        (CHILD_PATHS[2], children[2], 110),
        (CHILD_PATHS[3], children[3], 40),
        (CHILD_PATHS[4], children[4], 180),
        (CHILD_PATHS[5], children[5], 50),
        (CHILD_PATHS[6], children[6], 110),
        (CHILD_PATHS[7], children[7], 70),
        (CHILD_PATHS[8], children[8], 60),
        (CHILD_PATHS[9], children[9], 75),
        (CHILD_PATHS[10], children[10], 75),
        (CHILD_PATHS[11], children[11], 90),
        (CHILD_PATHS[12], children[12], 190),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/structure_guard_rows.rs"
    );
    assert_contains_all(
        "module-convention row data records input-stack guard split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[4],
            GUARD,
        ],
    );

    let status_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs"
    );
    assert_contains_all("structure route status map", status_map, &[SLICE, STATUS]);

    let date_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs"
    );
    assert_contains_all("structure route date map", date_map, &[SLICE, "2026-07-05"]);

    for (label, source) in [
        (
            "Runtime 15 subplan",
            crate::tests::runtime_absorption::current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT,
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
        ),
        (
            "engine code review findings",
            include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        ),
        (
            "module convention doc",
            include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[4]]);
    }

    let frameworks = include_str!(
        "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
    );
    assert_contains_all(
        "frameworks plan records input-stack guard split",
        frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
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
