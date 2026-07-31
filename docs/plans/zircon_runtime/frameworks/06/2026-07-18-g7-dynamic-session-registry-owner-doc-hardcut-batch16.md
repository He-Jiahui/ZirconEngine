---
related_code:
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/registry/mod.rs
  - zircon_runtime/src/dynamic_api/session/registry/action_guard.rs
  - zircon_runtime/src/dynamic_api/session/registry/frame_activity.rs
  - zircon_runtime/src/dynamic_api/session/registry/frame_demand.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_slot.rs
  - zircon_runtime/src/dynamic_api/session/registry/wake_registration.rs
  - tools/check_conventions.py
implementation_files:
  - docs/zircon_runtime/structure/module-convention.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/zircon_runtime/structure/module-convention.md
---

# Frameworks06 G7 Dynamic Session Registry 当前 Owner 文档硬切 Batch 16

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-dynamic-session-registry-owner-doc-hardcut-batch16-20260718`

## 完成项目

- 将结构规范 front matter 中两处已删除的 flat `dynamic_api/session/registry.rs` 硬切到唯一 folder-backed route owner `registry/mod.rs`。
- 将正文从过时的 682/69 行双文件描述同步为 current 51 行 session route、140 行 registry route 与五个职责 child owners。
- 不恢复 flat owner、alias、shim、兼容重导出或第二套 registry。

## Fresh Testing Evidence

- 修改前 fresh G7：所选文档 `2` 个 missing-path violations。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选文档 `0` violations；共享 current-source 全局快照为 `473` violations / `126` documents，G7 继续保持 RED。
- 所选文档内退役 flat registry 机器路径为 `0`，folder-backed route 与五个 child owners 均存在。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮为 `Critical 0 / Important 1 / Minor 0`：正文后续段仍把 registry API 误写为 flat parent imports。现已按 current source 改为 `session.rs` route-only mount，以及 `ffi.rs`、`linked_session.rs`、`operation.rs` 和 lock-poison tests 的直接消费拓扑；最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批 focused G7 与独立复审已通过。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
