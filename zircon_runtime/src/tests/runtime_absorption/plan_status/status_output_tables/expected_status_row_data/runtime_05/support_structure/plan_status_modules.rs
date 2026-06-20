use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 plan-status output-anchor module split",
        [
            "runtime_plan_status_output_anchors.py",
            "runtime_plan_status_boundary.py` remains orchestration",
            "761 lines",
            "direct `runtime_plan_status_boundary_audit` risks=0",
        ],
    ),
    (
        "Runtime 05 plan-status output-anchor budget guard",
        [
            "plan_status_boundary_line_count = 855",
            "max_plan_status_boundary_lines = 900",
            "status_output_anchor_module_present = true",
            "status_output_anchor_module_line_count = 143",
        ],
    ),
    (
        "Runtime 05 status-output status/date helper split",
        [
            "status_output_tables/expected_slices.rs",
            "expected_status_for_slice",
            "expected_date_for_slice",
            "status_output_tables.rs",
        ],
    ),
    (
        "Runtime 05 status-output expected anchor split",
        [
            "status_output_tables/expected_status_rows.rs",
            "EXPECTED_STATUS_OUTPUT_SLICES",
            "status_output_tables.rs",
            "plan-status support files 21/21",
        ],
    ),
    (
        "Runtime 05 plan-status root module split",
        [
            "plan_status/support.rs",
            "plan_status/index_tables.rs",
            "plan_status.rs",
            "plan-status support files 26/26",
        ],
    ),
    (
        "Runtime 05 plan-status support inventory split",
        [
            "runtime_plan_status_support_inventory.py",
            "PLAN_STATUS_SUPPORT_FILES",
            "plan_status_boundary_line_count = 842",
            "support 26/26",
        ],
    ),
    (
        "Runtime 05 plan-status anchor inventory split",
        [
            "runtime_plan_status_anchor_inventory.py",
            "CORE_GUARD_ANCHORS",
            "PENDING_GATE_ANCHORS",
            "plan_status_boundary_line_count = 789",
        ],
    ),
    (
        "Runtime 05 plan-status markdown renderer split",
        [
            "runtime_plan_status_markdown.py",
            "render_runtime_plan_status_boundary_markdown",
            "plan_status_boundary_line_count = 559",
            "support 26/26",
        ],
    ),
    (
        "Runtime 05 plan-status source helper split",
        [
            "runtime_plan_status_sources.py",
            "runtime_subplans",
            "status_rows",
            "plan_status_boundary_line_count = 454",
        ],
    ),
    (
        "Runtime 05 status-output expected row data split",
        [
            "expected_status_row_data.rs",
            "EXPECTED_STATUS_OUTPUT_SLICES",
            "expected_status_rows.rs",
            "plan-status support files 27/27",
        ],
    ),
];
