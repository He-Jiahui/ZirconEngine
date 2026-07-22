---
related_code:
  - docs/zircon_editor/ui/performance-timeline.md
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
implementation_files:
  - docs/zircon_editor/ui/performance-timeline.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B tools/check_conventions.py --repo-root E:\Git\ZirconEngine --only docs
  - git diff --check -- docs/zircon_editor/ui/performance-timeline.md docs/plans/zircon_runtime/frameworks/06/2026-07-22-g7-editor-runtime-gateway-owner-doc-hardcut-batch33.md
---

# Frameworks06 G7 Editor Runtime Gateway Owner 文档硬切 Batch 33

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: ["docs/zircon_editor/ui/performance-timeline.md"]
Date: 2026-07-22
Session: `frameworks06-g7-editor-runtime-gateway-owner-batch33-20260722`

## Scope Delivered

- 将 Performance Timeline 文档中两条已删除的 `zircon_editor/src/ui/host/editor_runtime_client.rs` 机器路径硬切到 `core/gateway/contract.rs` 与 `core/gateway/session.rs` 当前 owners。
- 同步删除旧 `EditorRuntimeClient` 语义：中立契约现在由 `EditorRuntimeGateway` 持有，默认路径报告 `runtime.profile.control` capability 缺失；`SessionGateway` 仅在验证后的 API table 未提供可选 hook 时返回 `Ok(None)`。
- 明确 `zircon_app::RuntimeSession::editor_gateway` 使用 runtime API table、session handle 与 capabilities 构造动态 gateway；请求编码、ABI 调用、响应解码与 owned buffer 释放由 `SessionGateway` 负责。
- 不恢复旧 client 文件，不增加 alias、shim、兼容 re-export 或并行 transport owner。

## Fresh Testing Evidence

- 修改前 fresh G7：全局 `581` violations；目标文档正好 `2` 个 missing-path violations，均指向删除的 `ui/host/editor_runtime_client.rs`。
- 修改后 fresh G7：全局 `579` violations；目标文档 focused violations 为 `0`。并发文档变化使全局 document/path 计数不作为本批固定验收锚，目标 2 → 0 为当前切片边界。
- coordinator snapshot `779` 锁定目标文档 SHA-256 `8b0583e9ed95f7d04ff46a1429cdc76295515820398f64f9f70769c13e4bec3f`；旧路径与旧类型名计数均为 `0`。

## Review

- 独立 reviewer Session `frameworks06-g7-editor-runtime-gateway-owner-batch33-review-20260722` 对 snapshot `779` 返回 `Critical 0 / Important 0 / Minor 0 — ready`。
- 复审逐项核对 contract 默认错误、SessionGateway 可选 hook/ABI buffer 生命周期及 RuntimeSession 构造路径，确认正文与当前源码一致且无 alias/shim/兼容实现。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | M1-T G7 Editor Runtime Gateway current-owner docs testing | 通过 | 2026-07-22 | fresh G7 目标 `2 → 0`、全局 `581 → 579`；snapshot 779 独立复审 `0/0/0`；scoped diff-check 通过。 |

## Milestone Decision

本批 focused G7、scoped diff-check 与独立复审均已通过，状态为 `accepted`，等待协调器管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`；其余 current-source violations 继续由后续独立批次收敛。
