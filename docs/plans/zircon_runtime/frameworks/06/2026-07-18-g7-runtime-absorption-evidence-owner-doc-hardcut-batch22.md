---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/current_source_fixture.rs
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_runtime/render/05/fixed-2026-07-16-runtime-absorption-missing-session-fixture.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md docs/plans/zircon_runtime/frameworks/06/2026-07-18-g7-runtime-absorption-evidence-owner-doc-hardcut-batch22.md docs/plans/zircon_runtime/render/05/fixed-2026-07-16-runtime-absorption-missing-session-fixture.md
---

# Frameworks06 G7 Runtime Absorption Evidence Owner 文档硬切 Batch 22

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-runtime-absorption-evidence-owner-doc-hardcut-batch22-20260718`

## 完成项目

- 将 Render05 fixed 记录 front matter 中已删除的 `.codex/sessions/20260612-0847-runtime-architecture-implementation.md` 硬切到 current Runtime15 tracked archive 与 `runtime_absorption/current_source_fixture.rs`。
- 明确退役 session 路径只保留在失败现象/历史证据正文，不再伪装成 current `related_code` machine owner。
- 不恢复旧 session fixture，不引入 alias、shim、兼容路径或第二份 evidence owner。

## Fresh Testing Evidence

- 修改前 fresh G7：所选 Render05 fixed 记录有 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选 Render05 fixed 记录 `0` violations；共享 current-source 全局快照为 `472` violations / `123` documents / `66,940` checked paths，G7 继续保持 RED。
- 所选记录 front matter 中退役 `.codex/sessions/20260612-0847-runtime-architecture-implementation.md` 为 `0`；Runtime15 archive 与 `runtime_absorption/current_source_fixture.rs` 两个 current owner 均存在。
- exact-scope `git diff --check` 通过；staged_total 为 `0`。

## Review

独立只读首轮复审确认 owner 与 fixed lifecycle 事实准确，发现 **Important 1**：记录中的 pending 证据未同步 fresh 结果。该状态矛盾修正后，终轮复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批只关闭所选 Runtime absorption evidence current-owner 文档漂移。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
