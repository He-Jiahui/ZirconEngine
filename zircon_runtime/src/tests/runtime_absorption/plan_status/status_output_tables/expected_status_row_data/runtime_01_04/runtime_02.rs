use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 02 root-surface Markdown renderer split",
        &[
            "runtime_root_surface_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "runtime_root_surface_markdown.py",
            "runtime_root_surface.py` remains the 268-line audit/risk owner",
            "35-line Markdown owner",
        ],
    ),
    (
        "Runtime 02 F6 core resource registry typed errors",
        &[
            "core_resource_registry_typed_errors_coremin_check_passed",
            "review_f6_core_resource_registry_rename_uses_core_error",
            "registry_rename_reports_missing_locator_with_core_error",
            "MissingResourceRecordForLocator",
        ],
    ),
    (
        "Runtime 02 core/root/generated 镜像文档守卫",
        &[
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "core_spine_root_generated_boundary",
            "standalone core_spine_root_generated 1/1",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 generated template count 审计同步",
        &[
            "`template_file_count=10`",
            "generated export templates 10/10",
            "0 migration debt",
            "stale 9/9 scan",
            "Runtime 02 generated/export/app/editor/plugin Cargo gates",
        ],
    ),
    (
        "Runtime 02 generated-code Markdown renderer split",
        &[
            "generated_code_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "generated_code_markdown.py",
            "287-line generated-code audit",
            "standalone generated_code_guard 7/7",
        ],
    ),
    (
        "Runtime 02 guard-test anchors 审计同步",
        &[
            "guard_test_anchor_count = 26",
            "missing_guard_test_anchors = []",
            "standalone core_spine_root_generated 1/1",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 root_entries guard-count current resync",
        &[
            "EXPECTED_ROOT_ENTRIES_TEST_COUNT",
            "root_entries guard tests 13/13",
            "guard_test_anchor_count = 26",
            "standalone core_spine_root_generated 1/1",
        ],
    ),
    (
        "Runtime 02 root graphics alias block removal",
        &[
            "graphics_alias_block_removed_static_passed_cargo_pending",
            "crate_visible_graphics_reexport_count = 0",
            "crate-visible graphics alias debt 0/0",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates",
        ],
    ),
    (
        "Runtime 02 rhi_wgpu root backend private cutover",
        &[
            "rhi_wgpu_root_backend_private_static_passed_cargo_pending",
            "runtime root public modules 19/19",
            "`rhi_wgpu` crate-private backend owner",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 builtin root facade cutover",
        &[
            "builtin_root_facade_removed_static_passed_cargo_pending",
            "public `pub use` sites 2/2",
            "root-surface M1 gate `classified-and-clear`",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 core/root/generated current audit recheck",
        &[
            "core_root_generated_current_audit_static_passed_cargo_pending",
            "core root entries 6/6",
            "standalone `generated_code_guard.rs` 7/7",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 core/root/generated 2026-07-01 current audit recheck",
        &[
            "core_root_generated_20260701_current_audit_static_passed_cargo_deferred",
            "runtime root public modules 19/19",
            "guard_test_anchor_count = 26",
            "full `audit_runtime_structure.py --json` 风险汇总为 `{}`",
        ],
    ),
    (
        "Runtime 02 core/root/generated Markdown renderer split",
        &[
            "core_root_generated_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "core_spine_root_generated_markdown.py",
            "315-line core/root/generated audit",
            "standalone core_spine_root_generated 1/1",
        ],
    ),
];
