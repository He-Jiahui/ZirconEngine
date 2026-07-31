---
related_code:
  - docs/assets-and-rendering/runtime-physics-animation-assets.md
  - docs/zircon_runtime/asset/scene.md
  - docs/zircon_runtime/core/framework/render/camera.md
  - docs/zircon_runtime/core/framework/render/core_pipeline.md
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
implementation_files:
  - docs/assets-and-rendering/runtime-physics-animation-assets.md
  - docs/zircon_runtime/asset/scene.md
  - docs/zircon_runtime/core/framework/render/camera.md
  - docs/zircon_runtime/core/framework/render/core_pipeline.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/assets-and-rendering/runtime-physics-animation-assets.md docs/zircon_runtime/asset/scene.md docs/zircon_runtime/core/framework/render/camera.md docs/zircon_runtime/core/framework/render/core_pipeline.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-runtime-scene-contract-doc-owner-hardcut-batch26.md
---

# Frameworks06 G7 Runtime Scene Contract 文档 Owner 硬切 Batch 26

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/assets-and-rendering/runtime-physics-animation-assets.md", "docs/zircon_runtime/asset/scene.md", "docs/zircon_runtime/core/framework/render/camera.md", "docs/zircon_runtime/core/framework/render/core_pipeline.md"]
Date: 2026-07-19
Session: `frameworks06-g7-runtime-scene-contract-doc-owner-hardcut-batch26-20260719`

## Scope Delivered

- 将四份 Runtime scene contract 文档中指向已删除 `zircon_runtime/src/scene/components/scene.rs` 的机器路径硬切到 folder-backed current owners。
- Scene asset 文档按正文持久化契约列出 curated facade 与 transform、hierarchy、activation、camera、mesh renderer、lighting、physics、animation、mobility 的真实声明责任。
- 物理/动画资产文档只记录 `physics.rs` 与 `animation.rs`；Camera 与 CorePipeline 文档只记录 `camera.rs`，不再用聚合 flat 文件冒充所有者。
- 旧 flat owner 保持物理删除，不创建 alias、shim、兼容 include 或重复文档真相。

## Fresh Testing Evidence

- 修改前 fresh G7：四份目标文档共 `7` 个 missing-path violations，均指向已删除 `components/scene.rs`；同一时序全局 `600` violations / `145` documents / `67,332` checked paths。
- 修改后 fresh G7：四份目标文档与本记录 focused `0` violations；同一时序全局降到 `593` violations / `141` documents / `67,362` checked paths，继续保持 RED。
- 四份目标文档的机器字段中退役 flat owner 计数为 `0`；10 个 curated facade/current owner 文件全部存在；exact-scope `git diff --check` 通过，仅输出工作树既有 LF/CRLF 提示。

## Review

- coordinator snapshot633 exact5 独立双遍复审为 `Critical 0 / Important 0 / Minor 0`、`Ready`；ordinal fingerprint `ff3b1da6dbf14769281bde2cd33a004d87c263e63a83533e445855d776b6e509`，首遍、末遍与父会话复算均一致，drift 为 `none`。

## Milestone Decision

本批 focused G7 与独立复审已通过，等待受管 milestone commit。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`，不得以局部路径迁移冒充计划完成。
