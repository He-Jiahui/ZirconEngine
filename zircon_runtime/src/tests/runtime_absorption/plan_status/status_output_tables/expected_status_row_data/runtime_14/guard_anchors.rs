use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 14 Module family guard anchors 审计同步",
        [
            "module_family_guard_anchor_count = 7",
            "missing_module_family_guard_anchors = []",
            "standalone root_entries 13/13",
            "module-family Cargo/rustc gates pending",
        ],
    ),
    (
        "Runtime 14 animation runtime-status JSON 边界守卫",
        [
            "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
            "animation_status_json_guard_present = true",
            "animation_status_json_anchor_count = 8",
            "missing_animation_status_json_anchors = []",
        ],
    ),
];
