---
related_code:
  - docs/engine-architecture/plugin-optional-feature-bundles.md
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution/ordered_ready_set.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project.rs
implementation_files:
  - docs/engine-architecture/plugin-optional-feature-bundles.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B tools/check_conventions.py --repo-root E:\Git\ZirconEngine --only docs
  - git diff --check -- docs/engine-architecture/plugin-optional-feature-bundles.md docs/plans/zircon_runtime/frameworks/06/2026-07-22-g7-plugin-feature-projection-owner-doc-hardcut-batch35.md
---

# Frameworks06 G7 Plugin Feature Projection Owner 文档硬切 Batch 35

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: ["docs/engine-architecture/plugin-optional-feature-bundles.md"]
Date: 2026-07-22
Session: `frameworks06-g7-plugin-feature-projection-owner-batch35-20260722`

## Scope Delivered

- 将 Optional Feature Bundles 文档中 5 个已删除 owner（capability base/declaration、availability/outcome、project lookup）硬切到 current derived projection、incremental resolver/ready queue 与 projection index lookup owners。
- 删除旧的 registration rescan、feature declaration scan helper 与 repeated fixed-point availability 设计叙述，改为 generation-scoped indexes 和按 missing capability 唤醒受影响行。
- `project_manifest/` 仅保留 completion/default hydration，package selection lookup 由 `project.rs` 通过 projection registration index 持有；不恢复旧文件，不增加 alias、shim 或兼容 re-export。

## Fresh Testing Evidence

- 修改前目标文档有 `10` 个 missing-path violations（5 个删除路径在 related/implementation 两组各一处）；修改后目标文档 focused violations 为 `0`，当前切片 `10 → 0`。
- fresh G7 全局观测由上一批后的 `581` 降至 `571`；并发文档变化可能改变全局计数，当前切片的目标 `10 → 0` 为验收边界。
- scoped `git diff --check` 通过，仅有工作树既有 LF/CRLF 提示；目标文档内 5 个删除路径 token 均为 `0`。
- coordinator snapshot `804` 锁定目标文档 SHA-256 `76ba85d26772b5c280f69f1cab7238e64ced2dba08e257c2524e992d2efc751a`。

## Review

- 独立 reviewer 对 exact2 返回 `Critical 0 / Important 0 / Minor 0 — READY`。
- 复审核对 generation rebuild、按 target 的 base/feature provider indexes、单次 feature status evaluation、missing-capability waiter 唤醒、current/next deterministic ready queue、blocking handoff 与 projection-index project lookup，确认正文与当前源码一致且无 alias/shim/compat。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | M1-T G7 plugin feature projection current-owner docs testing | 通过 | 2026-07-22 | fresh G7 目标 `10 → 0`、全局观测 `581 → 571`；snapshot 804；独立复审 `0/0/0`；scoped diff-check 通过。 |

## Milestone Decision

本批 focused G7、scoped diff-check 与独立复审均通过，状态为 `accepted`，等待协调器管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`；其余 current-source violations 继续由后续独立批次收敛。
