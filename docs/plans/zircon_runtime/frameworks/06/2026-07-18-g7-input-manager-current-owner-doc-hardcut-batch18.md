---
related_code:
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/event_buffer/frame.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_plugins/09/failure-2026-07-17-export-host-high-frequency-input-dispatch.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_plugins/09/failure-2026-07-17-export-host-high-frequency-input-dispatch.md
---

# Frameworks06 G7 InputManager 当前 Owner 文档硬切 Batch 18

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-input-manager-current-owner-doc-hardcut-batch18-20260718`

## 完成项目

- 将 Plugins09 开放 failure front matter 中已删除的 flat `zircon_runtime/src/input/input_manager.rs` 硬切到 current 中立 trait、生产 manager 与 frame event-buffer owners。
- 同步当前责任边界：Runtime `FrameEventBuffer` 已合并相邻 cursor latest-position 与 raw mouse delta，但 export browser/JNI/Swift host 的 ABI 前逐事件调用仍未关闭。
- 保持 failure `open`、fixing plan 与验收条件不变；不恢复 flat owner、转发层、alias、shim 或兼容重导出。

## Fresh Testing Evidence

- 修改前 fresh G7：所选文档 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选文档 `0` violations；共享 current-source 全局快照为 `471` violations / `124` documents，G7 继续保持 RED。
- 所选 failure 的 current owner 机器路径全部存在；退役 flat input-manager front-matter 路径为 `0`。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮为 `Critical 0 / Important 1 / Minor 0`：reviewer 在 fresh G7 与记录更新交错期间读到旧的“待运行”证据。Fresh Testing Evidence 随后同步为 focused `0`、global `471/124`、retired front-matter `0` 与 diff-check/staged_total `0`；源码 owner、Runtime 相邻事件合并及 host ABI 前 failure 责任边界未发现问题。最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批只关闭选定 current-owner 文档漂移，不关闭 Plugins09 高频输入 failure。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
