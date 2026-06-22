use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 14 Module family 镜像文档守卫",
        [
            "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
            "module_family_boundary",
            "standalone rustc 13/13",
            "module-family Cargo/rustc gates pending",
        ],
    ),
    (
        "Runtime 14 animation family 28-file audit sync",
        [
            "animation = 28",
            "navigation = 9",
            "module_family_boundary",
            "module_family_source_count_static_passed_cargo_pending",
        ],
    ),
    (
        "Runtime 14 navigation fallback runtime owner split",
        [
            "navigation_runtime_owner_split_static_passed_cargo_pending",
            "folder-backed runtime owner split",
            "navigation = 9",
            "runtime/avoidance.rs",
        ],
    ),
    (
        "Runtime 14 module family current audit recheck",
        [
            "module_family_current_audit_static_passed_cargo_pending",
            "standalone `root_entries.rs` 13/13",
            "missing_cargo_gate_anchors = []",
            "full lib Cargo gates pending",
        ],
    ),
    (
        "Runtime 14 module family markdown renderer split",
        [
            "module_family_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "module_family_markdown.py",
            "module_family_boundary.py` now owns only audit data/risk aggregation at 305 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
];
