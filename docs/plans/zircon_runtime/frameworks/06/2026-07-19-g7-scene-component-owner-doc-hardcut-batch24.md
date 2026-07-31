---
related_code:
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/scene/components/scene/node.rs
implementation_files:
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-scene-component-owner-doc-hardcut-batch24.md
---

# Frameworks06 G7 Scene Component Owner 文档硬切 Batch 24

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md"]
Date: 2026-07-19
Session: `frameworks06-g7-scene-component-owner-doc-hardcut-batch24-20260719`

## Scope Delivered

- 将 runtime foundation/scene authority 架构文档中 `related_code` 与 `implementation_files` 的两条已删除 `zircon_runtime/src/scene/components/scene.rs` 机器路径硬切到 folder-backed current owners。
- `scene/mod.rs` 只作为 curated component facade；`transform.rs` 拥有 `LocalTransform`/`WorldMatrix`，`activation.rs` 拥有 `ActiveSelf`/`ActiveInHierarchy`/`RenderLayerMask`，`core/framework/scene/mobility.rs` 拥有 `Mobility`，`node.rs` 拥有 `SceneNode`/`NodeRecord` 及其持久化字段，与正文 authority 清单逐项对应。
- 旧 flat owner 保持物理删除，不创建 alias、shim、兼容 include 或重复文档真相。

## Fresh Testing Evidence

- 修改前 fresh G7：所选架构文档有 `2` 个 missing-path violations，均指向已删除 `components/scene.rs`。
- 修改后 fresh G7：所选架构文档与本记录 focused `0` violations；同一时序全局从 `515` 降到 `513` violations / `146` documents / `67,170` checked paths，继续保持 RED。
- 退役 `zircon_runtime/src/scene/components/scene.rs` 在所选架构文档中计数为 `0`；5 个 current owner 均为现存文件；exact-scope `git diff --check` 通过。

## Review

- 独立双遍复审基于 coordinator snapshot597 exact2，ordinal fingerprint `8894cad2cabab0f4b7d456502694afdca87a110ae2e31abf03a9e6d462efe622`，结论 `Critical 0 / Important 0 / Minor 0`、`Ready`，两遍 drift 均为 `none`。

## Milestone Decision

本批只关闭 scene component hard-cut 遗留的 current-owner 文档漂移。focused G7 与独立复审已通过；Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`，本批等待受管 milestone commit，不以局部收敛冒充计划完成。
