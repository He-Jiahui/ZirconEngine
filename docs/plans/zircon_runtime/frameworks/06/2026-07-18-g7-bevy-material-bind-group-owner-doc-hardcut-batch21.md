---
related_code:
  - dev/bevy/crates/bevy_render/src/material_bind_groups.rs
  - dev/bevy/crates/bevy_pbr/src/material.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_runtime/render/19-gpu-capability-optimizations.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md docs/plans/zircon_runtime/frameworks/06/2026-07-18-g7-bevy-material-bind-group-owner-doc-hardcut-batch21.md docs/plans/zircon_runtime/render/19-gpu-capability-optimizations.md
---

# Frameworks06 G7 Bevy Material Bind Group Owner 文档硬切 Batch 21

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-bevy-material-bind-group-owner-doc-hardcut-batch21-20260718`

## 完成项目

- 将 Render19 中已删除的 `dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs` 硬切到 current `dev/bevy/crates/bevy_render/src/material_bind_groups.rs`。
- 同步责任边界：通用 `MaterialBindGroupAllocator`、bindless/non-bindless 双形态、slab 分配回收和 fallback resource 填充归 `bevy_render`；`bevy_pbr/src/material.rs` 只消费该通用 owner。
- 不恢复旧 PBR 私有文件，不引入 alias、shim、兼容路径或双 owner。

## Fresh Testing Evidence

- 修改前 fresh G7：所选 Render19 文档有 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选 Render19 文档 `0` violations；共享 current-source 全局快照为 `473` violations / `124` documents / `66,924` checked paths，G7 继续保持 RED。
- 退役 `dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs` 在所选文档中为 `0`；current `dev/bevy/crates/bevy_render/src/material_bind_groups.rs` 存在。
- exact-scope `git diff --check` 通过；staged_total 为 `0`。

## Review

独立只读首轮复审确认技术 owner/consumer 事实准确，发现 **Important 1**：记录中的 pending 证据未同步 fresh 结果。该状态矛盾修正后，终轮复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批只关闭所选 Bevy reference current-owner 文档漂移。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
