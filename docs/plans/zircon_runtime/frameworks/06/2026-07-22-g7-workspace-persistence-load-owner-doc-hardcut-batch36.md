---
related_code:
  - docs/zircon_editor/ui/workbench/project/workspace_persistence.md
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
implementation_files:
  - docs/zircon_editor/ui/workbench/project/workspace_persistence.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B tools/check_conventions.py --repo-root E:\Git\ZirconEngine --only docs
  - git diff --check -- docs/zircon_editor/ui/workbench/project/workspace_persistence.md docs/plans/zircon_runtime/frameworks/06/2026-07-22-g7-workspace-persistence-load-owner-doc-hardcut-batch36.md
---

# Frameworks06 G7 Workspace Persistence Load Owner 文档硬切 Batch 36

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: ["docs/zircon_editor/ui/workbench/project/workspace_persistence.md"]
Date: 2026-07-22
Session: `frameworks06-g7-workspace-persistence-load-owner-batch36-20260722`

## Scope Delivered

- 将 workspace persistence 文档中已删除的 `editor_project_document_load_from_path.rs` owner 硬切到 current `editor_project_document_load.rs`，同步 related/implementation 两组机器路径。
- 将旧的 path-based project reopen 叙述改为 current `ProjectAuthority -> AssetManager -> retained ProjectManager -> EditorProjectDocument::load_from_project` 单一代际流程，并将实际 orchestration owner `ui/host/project_access.rs` 纳入两组 metadata。
- 保留 workspace restore 的 recoverable diagnostic 与 default-layout fallback 语义，明确 missing file 是无诊断的正常 first-open 分支，只有 unreadable/corrupt/unsupported version 才产生 path+message diagnostic；不恢复旧文件，不增加 alias、shim 或兼容 re-export。

## Fresh Testing Evidence

- 修改前目标文档有 `2` 个 missing-path violations（同一删除 owner 在 related/implementation 两组各一处）；修改后目标文档 focused violations 为 `0`，当前切片 `2 → 0`。
- fresh G7 全局观测由上一批后的 `571` 降至 `569`；并发文档变化可能改变全局计数，当前切片的目标 `2 → 0` 为验收边界。
- scoped `git diff --check` 通过，仅有工作树既有 LF/CRLF 提示；目标文档内删除的 `editor_project_document_load_from_path.rs` 与 `load_from_path` token 均为 `0`。
- coordinator snapshot `813` 锁定首轮 exact2：目标文档 SHA-256 `c8633547e236f698868917904ee13c2a5364f18a52fe82cfb38faaec6a730016`，记录 SHA-256 `d6fe9a8c20ee3d2fbf98fa8f42be0ae72323b4192527f0a24a11986da4d741af`。
- coordinator snapshot `814` 锁定修正后目标文档 SHA-256 `1ede58be027e863997df35b07ccd10521acf9631c3168b9c20b13120fa06271b`。

## Review

- snapshot 813 首轮独立复审返回 `Critical 0 / Important 2 / Minor 1 — NOT READY`：指出 missing-file diagnostic 叙述与源码不符、`project_access.rs` orchestration owner 未进入 canonical metadata，且记录缺 immutable snapshot anchor。
- 上述三项已按 current source 修正；snapshot 814 终审返回 `Critical 0 / Important 0 / Minor 0 — READY`。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | M1-T G7 workspace persistence current-owner docs testing | 通过 | 2026-07-22 | fresh G7 目标 `2 → 0`、全局观测 `571 → 569`；snapshot 814；终审 `0/0/0`；scoped diff-check 通过。 |

## Milestone Decision

本批 focused G7、scoped diff-check 与独立复审均通过，状态为 `accepted`，等待协调器管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`；其余 current-source violations 继续由后续独立批次收敛。
