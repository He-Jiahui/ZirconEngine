---
related_code:
  - docs/zircon_runtime/plugin/bridge.md
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/plugin_activation/native.rs
  - zircon_editor/src/core/play/plugin_activation/report.rs
  - zircon_editor/src/core/play/transition_report.rs
implementation_files:
  - docs/zircon_runtime/plugin/bridge.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B tools/check_conventions.py --repo-root E:\Git\ZirconEngine --only docs
  - git diff --check -- docs/zircon_runtime/plugin/bridge.md docs/plans/zircon_runtime/frameworks/06/2026-07-22-g7-editor-play-plugin-bridge-owner-doc-hardcut-batch34.md
---

# Frameworks06 G7 Editor Play Plugin Bridge Owner 文档硬切 Batch 34

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: ["docs/zircon_runtime/plugin/bridge.md"]
Date: 2026-07-22
Session: `frameworks06-g7-editor-play-plugin-bridge-owner-batch34-20260722`

## Scope Delivered

- 将 Plugin Bridge 文档中已删除的 `zircon_editor/src/core/play/bridge.rs` owner 硬切到当前 `PlaySessionController`、`NativePluginBridgeActivation` 与 `PluginBridgeActivationReport` owners。
- 删除 `EditorRuntimePlayModeBackendReport` 与 `NativePluginEditorRuntimePlayModeBackend` 旧语义，改为当前 `PluginBridgeActivationReport.bridge_diagnostics` 经 `PlayTransitionReport.activation` 到 workbench snapshot 的数据链。
- 明确 `PlaySessionController` 持有整条 play transition 顺序和 backend-start 失败回滚；native plugin activation 负责 native live-host/可选 bridge-lifecycle 激活，并在内部通过 transition gate 串行保存 active snapshot。不恢复旧 bridge 文件，不增加 alias、shim 或兼容 re-export。

## Fresh Testing Evidence

- 修改前 fresh G7：全局 `583` violations；目标文档正好 `2` 个 missing-path violations，均指向删除的 `zircon_editor/src/core/play/bridge.rs`。
- 修改后 fresh G7：全局 `581` violations；目标文档 focused violations 为 `0`，当前切片 `2 → 0`。
- scoped `git diff --check` 通过，仅有工作树既有 LF/CRLF 提示；目标文档内旧路径及两个旧类型名计数均为 `0`。
- 首轮独立复审的 `Important 1 / Minor 1` 已修复：两处 owner 清单补齐 `transition_report.rs`，清零证据限定为目标文档。coordinator snapshot `797` 锁定修正后目标文档 SHA-256 `b0ff70162546a0b7288c85ed05b06519b7f598fe78d681c3a278f00a5e53a10c`。

## Review

- 独立 reviewer 对修正后磁盘 exact2 终审返回 `Critical 0 / Important 0 / Minor 0 — READY`。
- 终审核对 `PlaySessionController` 与 native activation 的职责、transition gate/active snapshot、snapshot 797、两处 current-owner metadata、旧 token 清零及无 alias/shim/compat，确认记录没有过度陈述。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | M1-T G7 editor play/plugin bridge current-owner docs testing | 通过 | 2026-07-22 | fresh G7 目标 `2 → 0`、全局 `583 → 581`；snapshot 797；独立终审 `0/0/0`；scoped diff-check 通过。 |

## Milestone Decision

本批 focused G7、scoped diff-check 与独立终审均通过，状态为 `accepted`，等待协调器管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`；其余 current-source violations 继续由后续独立批次收敛。
