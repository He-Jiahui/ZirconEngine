---
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_runtime/frameworks/01/2026-07-17-m1-resource-error-owner-dag-prerequisite.md
  - docs/plans/zircon_runtime/frameworks/02/fixed-2026-07-16-text-raster-pool-zircon-error-consumer.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_runtime/frameworks/01/2026-07-17-m1-resource-error-owner-dag-prerequisite.md docs/plans/zircon_runtime/frameworks/02/fixed-2026-07-16-text-raster-pool-zircon-error-consumer.md
---

# Frameworks06 G7 CoreError 当前 Owner 文档硬切 Batch 14

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-core-error-current-owner-doc-hardcut-batch14-20260718`

## 完成项目

- 将 Frameworks01 resource-error prerequisite 与 Frameworks02 fixed return 的 front matter 从已删除的 `core/framework/error.rs` 硬切到唯一现存 `core/runtime/error.rs`。
- 将 fixed return 中关于首次 RED 的历史叙述改为语义说明，不再把退役 owner 写成当前机器路径。
- 不恢复旧 error owner、alias、shim、兼容重导出或第二套错误枚举。

## Fresh Testing Evidence

- 修改前 fresh G7：所选两份文档共 `3` 个 missing-path violations。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选两份文档 `0` violations；共享 current-source 全局快照为 `475` violations / `127` documents，G7 继续保持 RED。
- 两份文档内退役 `core/framework/error.rs` 机器路径为 `0`，唯一 current owner `core/runtime/error.rs` 存在。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮为 `Critical 0 / Important 1 / Minor 0`：fixed return 的叙述把 current `core/runtime/error.rs` owner 误归因到 2026-07-16 历史切片。Git 历史确认当时提交仍修改当时的 framework error owner；现已删除错误时序归因，同时保留语义化 RED 说明与 current front-matter owner。最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批 focused G7 与独立复审已通过。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
