---
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs
  - tools/tests/test_runtime_render_legacy_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs
  - tools/tests/test_runtime_render_legacy_naming.py
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - python -m unittest tools.tests.test_runtime_render_legacy_naming -v
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py
  - git diff --check -- tools/tests/test_runtime_render_legacy_naming.py zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs zircon_runtime/src/core/framework/render/scene_extract.rs zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs docs/zircon_runtime/core/framework/render/advanced_lighting/material-features.md docs/zircon_runtime/core/framework/render/scene_extract.md docs/zircon_runtime/graphics/scene/scene_renderer/environment/lightmap-binding.md docs/zircon_runtime/core/framework/render/post_process.md docs/plans/zircon_runtime/runtime/15/2026-07-16-render-legacy-naming-hard-cut.md docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
doc_type: milestone-detail
status_anchor: runtime_15_render_legacy_naming_hard_cut_static_passed_shared_cargo_check_passed
---

# Runtime 15 Render Legacy Naming Hard Cut

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2 | Render owner 遗留命名硬切 | `runtime_15_render_legacy_naming_hard_cut_static_passed_shared_cargo_check_passed` | 2026-07-16 | TDD 红态精确命中 runtime naming 8 处和 hard-cut 2 处；实现后聚焦 Python 回归 3/3 通过，完整结构审计的 module-convention gate 为 `classified-and-clear`、migration debt 0；共享 HEAD 的 managed `cargo check -p zircon_runtime --lib --locked` 通过。 |

## 完成项

- Standard PBR 零 lobe 合同改用 `baseline`，直接表达默认 shader/pipeline identity，不保留迁移术语。
- Hybrid GI 反序列化 fixture 改用 `pre_m4`，明确这是 M4 字段出现前的 schema 输入；运行时默认语义未变。
- 单页 lightmap 通用纹理 view 硬切为 `page_zero_bind_group_view` / `lightmap_page_zero_bind_group_view_descriptor`；测试同名迁移，没有旧函数别名、re-export 或 forwarding shim。
- 全屏 baked ambient 参数改为 `retired`，并在模块文档中固定逐 surface lightmap 是唯一 baked-indirect owner。
- 新增聚焦审计回归，既锁定当前 owner 名称，也要求 runtime naming 与 hard-cut migration debt 同时归零。

## 验证状态

- 红态：`python -m unittest tools.tests.test_runtime_render_legacy_naming -v` 为 0/3，通过失败信息确认 `legacy-runtime-graphics-debt` 分别为 8 处与 2 处。
- 绿态：同一命令为 3/3，通过；`runtime_naming_boundary_audit` 的 `legacy.migration_debt_count=0`，`hard_cutover_migration_smells_audit` 的 `hard_cutover_migration_debt_count=0` 且 `risks=[]`。
- 完整 `audit_runtime_structure.py --json` 通过，`module_convention_gate.m1_gate_status=classified-and-clear`、`migration_debt_count=0`、`risk_count=0`；审计同时保留其他当前计划 owner 的 root/tech-stack 等未关闭项，未把它们误记为本切片完成。
- 输出记录审计、scoped rustfmt、旧符号扫描与 scoped `git diff --check` 通过；差异检查只报告仓库既有 LF/CRLF 转换提醒。
- 本切片请求的 managed `validate-matrix -Package zircon_runtime -SkipBuild` 在 Test lane 排队约 15 分钟，外部等待器超时后协调器按进程身份标记该请求 `orphaned`；其 Cargo 子进程未启动，因此没有把该请求误记为通过，随后仅通过协调器释放该 orphaned job。
- 同一共享 HEAD 上，Frameworks05 owner 的 Windows managed check job `b1345befab644601be7b3544f06ec981` 执行 `cargo check -p zircon_runtime --lib --locked`，于 2026-07-16 02:00 CST 以 exit 0 完成并释放；输出为 508 个既有 warning，无 error。该门禁覆盖本切片五个 Rust 文件的编译，但不冒充 `--tests` 或聚焦 runtime test execution。

## 父计划状态

Runtime15 的 module-convention 命名债已在聚焦门禁中归零，但父计划仍保持 `in_progress`，直到完整聚合结构审计、受管 Cargo 门禁和其他计划完成定义全部满足；不以本切片局部通过冒充父计划完成。
