use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 09 UI architecture Markdown renderer split",
        &[
            "ui_architecture_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "ui_architecture_markdown.py",
            "ui_architecture_boundary.py` remains the 541-line audit/risk owner",
            "Markdown owner is 110 lines",
        ],
    ),
    (
        "Runtime 09 UI architecture 镜像文档守卫",
        &[
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
            "ui_architecture_boundary",
            "standalone rustc 18/18",
            "ui/input/naming_boundary/layout/template Cargo gates pending",
        ],
    ),
    (
        "Runtime 09 UI entry map audit sync",
        &[
            "runtime_09_ui_entry_map_audit_sync_static_passed_cargo_deferred",
            "expected_ui_entry_count = 19",
            "expected_surface_entry_count = 21",
            "platform_input",
            "property_mutation",
            "has_pointer_capture_or_unindexed_fallback_for_owner",
            "risks = []",
        ],
    ),
    (
        "Runtime 09 UI input route authority",
        &[
            "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending",
            "runtime_09_m1_1_ui_input_route_authority",
            "runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers",
            "route_authority.rs",
        ],
    ),
];
