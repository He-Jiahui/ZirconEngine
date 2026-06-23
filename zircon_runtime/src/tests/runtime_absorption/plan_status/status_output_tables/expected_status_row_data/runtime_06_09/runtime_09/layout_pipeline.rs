use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 09 taffy bridge pass order",
        &[
            "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending",
            "UI_LAYOUT_PASS_ORDER",
            "compute_taffy_child_frames",
            "cargo check -p zircon_runtime",
        ],
    ),
    (
        "Runtime 09 virtualization scroll boundary",
        &[
            "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
            "UiScrollVirtualizationPlan",
            "plan_scrollable_virtual_window",
            "scroll_virtualization.rs",
        ],
    ),
    (
        "Runtime 09 template pipeline boundary",
        &[
            "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
            "UiTemplateRuntimePipeline",
            "UI_TEMPLATE_RUNTIME_PIPELINE_STAGES",
            "template_pipeline.rs",
        ],
    ),
];
