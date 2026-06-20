use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 status-output Runtime 02 generated template row",
        [
            "Runtime 02 generated template count 审计同步",
            "template_file_count=10",
            "generated export templates 10/10",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 Runtime 02 generated status 审计元数据",
        [
            "runtime_02_generated_status_index_anchor_count = 6",
            "runtime_02_generated_status_guard_anchor_count = 5",
            "runtime_02_generated_status_guard_present = true",
            "index 6/6",
        ],
    ),
    (
        "Runtime 05 Runtime 02 root_entries count 状态表闭环",
        [
            "Runtime 02 root_entries guard-count current resync",
            "EXPECTED_ROOT_ENTRIES_TEST_COUNT",
            "root_entries guard tests 13/13",
            "standalone plan-status 32/32",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 03 module-doc row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 03 Schedule/frame-loop module-doc 镜像元数据",
            "frame schedule module-doc anchors 3/3",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 03 behavior-test row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 03 Schedule/frame-loop 行为测试锚审计同步",
            "behavior_test_anchor_count = 13",
            "standalone status-output 2/2",
        ],
    ),
];
