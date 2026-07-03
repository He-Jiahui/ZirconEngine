use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 12 Input stack 镜像文档守卫",
        &[
            "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
            "input_stack_boundary",
            "standalone rustc 4/4",
            "Cargo input/action_map/gamepad/app gates pending",
        ],
    ),
    (
        "Runtime 12 Input stack 行为测试锚审计同步",
        &[
            "behavior_test_anchor_count = 12",
            "missing_behavior_test_anchors = []",
            "standalone input_stack 4/4",
            "input/action_map/gamepad/app Cargo gates pending",
        ],
    ),
    (
        "Runtime 12 input validation window recheck",
        &[
            "cargo test -p zircon_runtime --lib input --no-default-features --features core-min",
            "1200s",
            "standalone `input_stack.rs` 4/4",
            "input/action_map/gamepad/app Cargo gates pending",
        ],
    ),
    (
        "Runtime 12 Input stack current audit recheck",
        &[
            "input_stack_current_audit_static_passed_cargo_pending",
            "runtime/framework/test owner 12/20/7",
            "standalone `input_stack.rs` 4/4",
            "input/action_map/gamepad/app Cargo gates",
        ],
    ),
    (
        "Runtime 12 Input stack 2026-07-01 current audit recheck",
        &[
            "input_stack_20260701_current_audit_static_passed_cargo_deferred",
            "runtime/framework/test owner 12/20/7",
            "behavior-test anchors 15/15",
            "full `audit_runtime_structure.py --json` 风险汇总为 `{}`",
        ],
    ),
    (
        "Runtime 12 Input stack inventory split",
        &[
            "input_stack_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "input_stack_source_inventory.py",
            "input_stack_anchor_inventory.py",
            "input_stack_boundary.py",
        ],
    ),
    (
        "Runtime 12 Input stack Markdown renderer split",
        &[
            "input_stack_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "input_stack_markdown.py",
            "input_stack_boundary.py` now owns audit read, missing-anchor calculation, and risk aggregation at 337 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 12 input boundary grouped manager import guard repair",
        &[
            "input_boundary_grouped_manager_import_guard_passed_broader_input_failures_pending",
            "input_protocol_types_live_in_runtime_input_surface",
            "1 passed; 0 failed; 4704 filtered out",
            "342 passed; 11 failed; 1 ignored",
        ],
    ),
    (
        "Runtime 12 input_manager child test owner audit sync",
        &[
            "runtime_12_input_manager_child_test_owner_audit_sync_static_passed_cargo_deferred",
            "input/tests/input_manager/{frame_state,host_requests}.rs",
            "missing_behavior_test_anchors = []",
            "targeted `plan_status.rs` status-table/index/last_refined checks 3/3",
        ],
    ),
];
