use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 01 Tech-stack 镜像文档守卫",
        &[
            "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
            "tech_stack_boundary",
            "standalone rustc 1/1",
            "tech_stack/extensions/text_shaper/plugin physics Cargo gates pending",
        ],
    ),
    (
        "Runtime 01 Tech-stack 行为测试锚审计同步",
        &[
            "behavior_test_anchor_count = 4",
            "missing_behavior_test_anchors = []",
            "standalone tech_stack 1/1",
            "tech_stack/extensions/text_shaper/plugin physics Cargo gates pending",
        ],
    ),
    (
        "Runtime 01 Tech-stack current audit recheck",
        &[
            "manifest files 5/5",
            "tech-stack guard anchors 12/12",
            "behavior-test anchors 4/4",
            "standalone `plan_status.rs` 32/32",
        ],
    ),
    (
        "Runtime 01 Tech-stack inventory split",
        &[
            "tech_stack_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "tech_stack_source_inventory.py",
            "tech_stack_anchor_inventory.py",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 01 Tech-stack Markdown renderer split",
        &[
            "tech_stack_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "tech_stack_markdown.py",
            "tech_stack_boundary.py` now owns audit read, dependency scanning, missing-anchor calculation, and risk aggregation at 341 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 01 export build-plan directory materialization boundary",
        &[
            "materialize/{mod,generated,paths,native,package_lookup,copy,report}.rs",
            "path traversal guard",
            "不引入 `zip` / `tar`",
            "Cargo 与 focused behavior tests",
        ],
    ),
    (
        "Runtime 01 NativeDynamic materialization symlink boundary",
        &[
            "只遍历真实目录",
            "只读取真实 `plugin.toml`",
            "跳过 symlinked package top-level payload",
            "directory-first materialization",
        ],
    ),
    (
        "Runtime 01 export materialization dry-run preview",
        &[
            "preview_materialize",
            "planned generated file paths",
            "不创建目录、不写文件、不复制 payload",
            "Cargo 与 focused behavior tests",
        ],
    ),
    (
        "Runtime 01 export materialization fatal preflight gate",
        &[
            "effective_fatal_diagnostics()",
            "空 `generated_files` / `copied_packages`",
            "materialization-blocked diagnostic",
            "write_generated_files(...)",
        ],
    ),
    (
        "Runtime 01 editor native-aware fatal export early exit",
        &[
            "plan.has_fatal_diagnostics()",
            "runtime no-op `materialize(...)` report",
            "空 generated/copied/native-cargo/source-cargo 结果",
            "NativeDynamic staging/build",
        ],
    ),
    (
        "Runtime 01 editor native-aware discovery reuse",
        &[
            "complete_project_plugin_manifest_with_native_report",
            "NativePluginLoadReport",
            "重复扫描 plugin directory",
            "generate_native_aware_export_plan(...)",
        ],
    ),
    (
        "Runtime 01 export ZIP archive materialization",
        &[
            "materialize_zip_archive",
            "zip = { version = \"9.0.0-pre2\"",
            "ExportMaterializeReport.archive_file",
            "native_dynamic_zip_archive_materialization_writes_generated_files_and_runtime_payloads",
        ],
    ),
];
