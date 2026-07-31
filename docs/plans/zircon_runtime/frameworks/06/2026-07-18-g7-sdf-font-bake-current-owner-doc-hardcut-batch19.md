---
related_code:
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/distance_field.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
---

# Frameworks06 G7 SDF Font Bake 当前 Owner 文档硬切 Batch 19

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-sdf-font-bake-current-owner-doc-hardcut-batch19-20260718`

## 完成项目

- 将 EditorLayout17 front matter 中已删除的 scene-renderer `ui/sdf_font_bake.rs` 硬切到 current Runtime Text `text/sdf/font_bake.rs` owner。
- 将正文的实现权让渡说明同步到 current `ui/text/layout_engine.rs` / `text/sdf/font_bake.rs` 路径，并明确历史短名不代表兼容入口。
- 保持 EditorLayout17 为编辑器侧排版验收规范，Runtime Text 继续持有 SDF bake/cache/offline-source 实现权；不恢复旧 owner、转发层、alias、shim 或兼容重导出。

## Fresh Testing Evidence

- 修改前 fresh G7：所选文档 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选文档 `0` violations；共享 current-source 全局快照为 `470` violations / `123` documents，G7 继续保持 RED。
- EditorLayout17 的 current SDF owner 机器路径全部存在；退役 scene-renderer SDF bake front-matter 路径为 `0`。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮为 `Critical 0 / Important 1 / Minor 0`：reviewer 在 fresh G7 与记录更新交错期间读到旧的“待运行”证据。Fresh Testing Evidence 随后同步为 focused `0`、global `470/123`、retired front-matter `0` 与 diff-check/staged_total `0`；current SDF root/child wiring、Editor 验收权与 Runtime Text 实现权边界未发现问题。最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批只关闭选定 current-owner 文档漂移。Frameworks06 M1、全局 G7、EditorLayout17 与计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
