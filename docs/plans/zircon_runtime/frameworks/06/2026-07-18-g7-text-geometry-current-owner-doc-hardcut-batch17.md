---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
---

# Frameworks06 G7 Text Geometry 当前 Owner 文档硬切 Batch 17

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-text-geometry-current-owner-doc-hardcut-batch17-r2-20260718`

## 完成项目

- 将 Text03 front matter 中已删除的 flat `zircon_runtime_interface/src/ui/surface/render/text_geometry.rs` 硬切到 folder-backed `text_geometry/mod.rs` 与 `text_geometry/source_map.rs`。
- 将正文同步为 current topology：`mod.rs` 负责 editable selection/composition/caret decoration 协调，`source_map.rs` 负责 resolved line 的 source-byte 到 visual-cluster 唯一映射。
- 补齐消费链：`command.rs` 直接调用 `editable_text_decorations(...)`；source-map DTO 经 interface `render/mod.rs`、`surface/mod.rs` 重导出后，由 Runtime `ui/text/geometry.rs` 与 `ui/text/hit_test.rs` 消费，不在消费者中重建映射策略。
- 不恢复 flat owner、转发层、alias、shim 或兼容重导出，也不删除 G7 字段规避审计。

## Fresh Testing Evidence

- 修改前 fresh G7：所选文档 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选文档 `0` violations；共享 current-source 全局快照为 `472` violations / `125` documents，G7 继续保持 RED。
- Text03 front matter 的 G7 机器路径全部存在；退役 flat interface owner 的 front-matter 机器路径为 `0`。不固化会随 current owner 扩展而漂移的路径总数。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮为 `Critical 0 / Important 1 / Minor 0`：记录漏列 `command.rs` decoration 直连与 interface re-export → Runtime geometry/hit-test 消费链，导致“同步 current topology”证据不足。补齐后复审又发现路径总数固化为扩展前旧值，现已改为不固化易漂移总数。最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批只关闭选定 current-owner 文档漂移。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
