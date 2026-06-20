use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 recent-static Runtime 02/07 status metadata guard",
        [
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_02_generated_status_guard_present = true",
            "runtime_07_owner_budget_status_guard_present = true",
            "standalone recent_static 1/1",
        ],
    ),
    (
        "Runtime 05 status-output recent-static metadata row",
        [
            "Runtime 05 recent-static Runtime 02/07 status metadata guard",
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_02_generated_status_guard_present = true",
            "standalone recent_static 1/1",
        ],
    ),
    (
        "Runtime 05 non-network server UI sortingMode allowlist",
        [
            "sortingMode = \"server\"",
            "allowed_context_count 99",
            "unclassified_location_count 0",
            "aggregate `audit_runtime_structure.py --json` non-network assertions",
        ],
    ),
    (
        "Runtime 05 status-output non-network server allowlist row",
        [
            "Runtime 05 non-network server UI sortingMode allowlist",
            "sortingMode = \"server\"",
            "allowed_context_count 99",
            "unclassified_location_count 0",
        ],
    ),
    (
        "Runtime 05 naming_boundary non-network server Rust guard",
        [
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime-ui-component-catalog-editor-controls",
            "standalone naming_boundary 2/2",
            "sortingMode = \"server\"",
        ],
    ),
    (
        "Runtime 05 texture importer DDS caps policy wording",
        [
            "DDSCAPS2_CUBEMAP caps2 policy",
            "legacy_reference_count = 148",
            "hard_cutover_migration_debt_count = 5",
            "DDS debt bucket absent",
        ],
    ),
];
