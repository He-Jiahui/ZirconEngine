use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 02 core/root/generated 镜像文档守卫",
        [
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "core_spine_root_generated_boundary",
            "standalone core_spine_root_generated 1/1",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 generated template count 审计同步",
        [
            "`template_file_count=10`",
            "generated export templates 10/10",
            "0 migration debt",
            "stale 9/9 scan",
        ],
    ),
    (
        "Runtime 02 guard-test anchors 审计同步",
        [
            "guard_test_anchor_count = 26",
            "missing_guard_test_anchors = []",
            "standalone core_spine_root_generated 1/1",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 root_entries guard-count current resync",
        [
            "EXPECTED_ROOT_ENTRIES_TEST_COUNT",
            "root_entries guard tests 13/13",
            "guard_test_anchor_count = 26",
            "standalone core_spine_root_generated 1/1",
        ],
    ),
    (
        "Runtime 02 root graphics alias block removal",
        [
            "graphics_alias_block_removed_static_passed_cargo_pending",
            "crate_visible_graphics_reexport_count = 0",
            "crate-visible graphics alias debt 0/0",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates",
        ],
    ),
    (
        "Runtime 02 rhi_wgpu root backend private cutover",
        [
            "rhi_wgpu_root_backend_private_static_passed_cargo_pending",
            "runtime root public modules 19/19",
            "`rhi_wgpu` crate-private backend owner",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 builtin root facade cutover",
        [
            "builtin_root_facade_removed_static_passed_cargo_pending",
            "public `pub use` sites 2/2",
            "root-surface M1 gate `classified-and-clear`",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 core/root/generated current audit recheck",
        [
            "core_root_generated_current_audit_static_passed_cargo_pending",
            "core root entries 6/6",
            "standalone `generated_code_guard.rs` 7/7",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
];
