---
related_code:
  - docs/zircon_runtime/scene/ecs-to-render-workflow.md
  - docs/zircon_runtime/scene/inspection.md
  - docs/zircon_runtime/scene/reflect.md
  - docs/zircon_runtime/scene/render_extract.md
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/components/scene/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
implementation_files:
  - docs/zircon_runtime/scene/ecs-to-render-workflow.md
  - docs/zircon_runtime/scene/inspection.md
  - docs/zircon_runtime/scene/reflect.md
  - docs/zircon_runtime/scene/render_extract.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/zircon_runtime/scene/ecs-to-render-workflow.md docs/zircon_runtime/scene/inspection.md docs/zircon_runtime/scene/reflect.md docs/zircon_runtime/scene/render_extract.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-runtime-scene-doc-owner-hardcut-batch25.md
---

# Frameworks06 G7 Runtime Scene 文档 Owner 硬切 Batch 25

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/zircon_runtime/scene/ecs-to-render-workflow.md", "docs/zircon_runtime/scene/inspection.md", "docs/zircon_runtime/scene/reflect.md", "docs/zircon_runtime/scene/render_extract.md"]
Date: 2026-07-19
Session: `frameworks06-g7-runtime-scene-doc-owner-hardcut-batch25-20260719`

## Scope Delivered

- 将四份 Runtime scene 文档中 `7` 条指向已删除 `zircon_runtime/src/scene/components/scene.rs` 的机器路径硬切到 folder-backed current owners。
- `scene/mod.rs` 只记录 curated facade；`transform.rs`、`activation.rs`、`identity.rs`、`hierarchy.rs`、`camera.rs`、`mesh_renderer.rs`、`lighting.rs`、`post_process.rs` 与 `core/framework/scene/mobility.rs` 分别记录实际声明责任。
- 每份文档按正文语义选择 owner：ECS-to-render 覆盖 transform/activation、mesh mutation、post-process 与 mobility；inspection 覆盖 inspection-visible identity/hierarchy/transform/activation/mesh 以及 `NodeRecord`；reflection 补齐相同 fixed-component owners、camera 与 Mobility；render extract 覆盖参与 frame projection 的 scene component owners。
- 旧 flat owner 保持物理删除，不创建 alias、shim、兼容 include 或重复文档真相。

## Fresh Testing Evidence

- 修改前 fresh G7：四份目标文档共 `7` 个 missing-path violations，均指向已删除 `components/scene.rs`；同一时序全局 `513` violations / `146` documents / `67,177` checked paths。
- 修改后 fresh G7：四份目标文档与本记录 focused `0` violations；同一时序全局降到 `506` violations / `142` documents / `67,244` checked paths，继续保持 RED。
- 四份目标文档的机器字段和正文 current facts 中退役 flat owner 计数为 `0`；逐文档 required-owner 检查 `4/4` 通过，union `11/11` facade/owner 文件存在；exact-scope `git diff --check` 通过。

## Review

- snapshot599 首轮独立复审为 `Critical 0 / Important 3 / Minor 0`、`Reject`，两遍 drift 为 `none`；两处正文旧 owner、逐文档 owner 遗漏与 GREEN 记录缺口均已修正，等待 exact5 successor snapshot 复审。
- snapshot601 exact5 修正版独立双遍复审为 `Critical 0 / Important 0 / Minor 0`、`Ready`，ordinal fingerprint `0ce39118fc381464774f62607653e17a99a5e582ee4bb5493cdfbd79ccdd8151`，两遍 drift 均为 `none`。

## Milestone Decision

本批只关闭四份 Runtime scene 文档的 current-owner 漂移。focused G7 与独立复审已通过；Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`，本批等待受管 milestone commit，不以局部收敛冒充计划完成。
