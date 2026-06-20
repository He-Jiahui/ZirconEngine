use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 12 Input stack 镜像文档守卫",
        [
            "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
            "input_stack_boundary",
            "standalone rustc 4/4",
            "Cargo input/action_map/gamepad/app gates pending",
        ],
    ),
    (
        "Runtime 12 Input stack 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 12",
            "missing_behavior_test_anchors = []",
            "standalone input_stack 4/4",
            "input/action_map/gamepad/app Cargo gates pending",
        ],
    ),
    (
        "Runtime 12 input validation window recheck",
        [
            "cargo test -p zircon_runtime --lib input --no-default-features --features core-min",
            "1200s",
            "standalone `input_stack.rs` 4/4",
            "input/action_map/gamepad/app Cargo gates pending",
        ],
    ),
    (
        "Runtime 12 Input stack current audit recheck",
        [
            "input_stack_current_audit_static_passed_cargo_pending",
            "runtime/framework/test owner 12/20/7",
            "standalone `input_stack.rs` 4/4",
            "input/action_map/gamepad/app Cargo gates",
        ],
    ),
];
