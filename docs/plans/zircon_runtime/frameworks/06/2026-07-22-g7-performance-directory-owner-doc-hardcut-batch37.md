---
related_code:
  - docs/plans/performance/01/2026-07-17-editor-host-window-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-material-primitives-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-mui-x-primitives-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-native-pointer-button-dispatch-static-review.md
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B tools/check_conventions.py --repo-root E:\Git\ZirconEngine --only docs
  - git diff --check -- docs/plans/performance/01/2026-07-17-editor-host-window-static-review.md docs/plans/performance/01/2026-07-17-editor-material-primitives-static-review.md docs/plans/performance/01/2026-07-17-editor-mui-x-primitives-static-review.md docs/plans/performance/01/2026-07-17-editor-native-pointer-button-dispatch-static-review.md docs/plans/zircon_runtime/frameworks/06/2026-07-22-g7-performance-directory-owner-doc-hardcut-batch37.md
---

# Frameworks06 G7 Performance Directory Owner 文档硬切 Batch 37

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: []
Date: 2026-07-22
Session: `frameworks06-g7-performance-directory-owner-batch37-20260722`

## Scope Delivered

- 将 4 份 performance implementation-evidence 文档中伪装成 machine path 的 `**/*.rs` glob 硬切到 4 个真实存在的 folder-backed directory owners。
- 保留 sibling module root 与 directory tree 的双 owner 口径，不把 glob、测试函数锚点或历史扫描表达式继续放入 `related_code`。
- 正文范围表达同步改为 directory tree，不恢复 glob 兼容解析、不放宽 G7，也不改变原性能发现或动态验收 pending 状态。
- 4 份目标 implementation-evidence 归属 `docs/plans/performance/01`，由其 owning Performance01 milestone 提交；本文件只作为 Frameworks06 的治理状态记录，不把跨 child-plan 输出伪装成本 child 的 `implementation_files`。

## Fresh Testing Evidence

- 修改前 4 份目标文档各有 `1` 个 missing-path violation；修改后每份 focused violations 均为 `0`，当前切片 `4 → 0`。
- fresh G7 全局观测由上一批后的 `569` 降至 `565`；并发文档变化可能改变全局计数，当前切片的目标 `4 → 0` 为验收边界。
- scoped `git diff --check` 通过，仅有工作树既有 LF/CRLF 提示；4 个目标 metadata 中 `**/*.rs` glob 均为 `0`。
- coordinator snapshot `827` 锁定 4 份目标文档：host-window SHA-256 `1c66cb10ee9027f45d3de8774b1c5e93961d8257519217abeead795b2b8d41df`、material-primitives `d60f3af125ffc87f0bc401464306ec6946f6c4030d4a0a6f03992cae2e03587d`、mui-x `c35655e6c257c3c705a6903b2133006c29bf5ff97cfb30c8cecc2be8959e9d44`、button-dispatch `ebeac2468ef5d8d855e68e9d9fdca30fe47d204d0c417d8573fd7f525417f497`。

## Review

- snapshot 827 首轮独立复审返回 `Critical 0 / Important 0 / Minor 1 — NOT READY`；唯一 finding 是 canonical record 缺 immutable snapshot/hash 锚点，已按上述四份目标 SHA-256 补齐，等待终审。
- snapshot 829 终审确认四份 target hash 未漂移，返回 `Critical 0 / Important 0 / Minor 0 — READY`。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | M1-T G7 performance directory-owner docs testing | 通过 | 2026-07-22 | fresh G7 目标 `4 → 0`、全局观测 `569 → 565`；snapshot 829；终审 `0/0/0`；scoped diff-check 通过。 |

## Milestone Decision

本批 focused G7、scoped diff-check 与独立复审均通过，状态为 `accepted`；目标文档与本治理记录按各自 owning plan 分离进入协调器管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`；其余 current-source violations 继续由后续独立批次收敛。
