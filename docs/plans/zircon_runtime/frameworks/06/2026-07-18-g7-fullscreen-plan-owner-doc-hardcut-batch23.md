---
related_code:
  - zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_editor/editor/02/fixed-2026-07-14-compute-fullscreen-descriptor-compile-regression.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md docs/plans/zircon_runtime/frameworks/06/2026-07-18-g7-fullscreen-plan-owner-doc-hardcut-batch23.md docs/plans/zircon_editor/editor/02/fixed-2026-07-14-compute-fullscreen-descriptor-compile-regression.md
---

# Frameworks06 G7 Fullscreen Plan Owner 文档硬切 Batch 23

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-fullscreen-plan-owner-doc-hardcut-batch23-20260718`

## 完成项目

- 将 Editor02 的 Shader04 fixed 记录中已删除的 `feature_descriptors/fullscreen_pass.rs` 硬切到 current `graphics/shader/builtin_global_shader_contracts.rs`。
- 同步 consuming-builder 责任：builtin global shader contracts 是 fullscreen plan 的 current owner；旧 descriptor helper 保持物理删除。
- 不恢复旧 fullscreen owner，不引入 alias、shim、兼容路径或重复 plan 真相。

## Fresh Testing Evidence

- 修改前 fresh G7：所选 fixed 记录有 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选 fixed 记录 `0` violations；共享 current-source 全局快照为 `471` violations / `122` documents / `66,953` checked paths，G7 继续保持 RED。
- 所选记录 front matter 中退役 `feature_descriptors/fullscreen_pass.rs` 为 `0`；current `graphics/shader/builtin_global_shader_contracts.rs` owner 存在。
- exact-scope `git diff --check` 通过；staged_total 为 `0`。

## Review

独立只读首轮复审确认 fullscreen/consuming-builder 与 fixed lifecycle 事实准确，发现 **Important 1**：记录中的 pending 证据未同步 fresh 结果。该状态矛盾修正后，终轮复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批只关闭所选 fullscreen plan current-owner 文档漂移。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
