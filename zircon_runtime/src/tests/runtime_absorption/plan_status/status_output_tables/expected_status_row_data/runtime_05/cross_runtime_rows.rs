use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 status-output Runtime 12 gamepad event-owner row",
        &[
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 12 gamepad event-owner 漂移同步",
            "missing_gamepad_abi_anchors = []",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 12 behavior-test row",
        &[
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 12 Input stack 行为测试锚审计同步",
            "behavior_test_anchor_count = 6",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 04 behavior-test row",
        &[
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 04 Asset pipeline 行为测试锚审计同步",
            "behavior_test_anchor_count = 20",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 08 behavior-test row",
        &[
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 08 ECS 行为测试锚审计同步",
            "behavior_test_anchor_count = 16",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 10 behavior-test row",
        &[
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 10 Dynamic API 行为测试锚审计同步",
            "behavior_test_anchor_count = 16",
            "standalone status-output 2/2",
        ],
    ),
];
