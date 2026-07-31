---
related_code:
  - docs/zircon_runtime/asset/assets/mesh.md
  - docs/zircon_runtime/asset/assets/scene.md
  - docs/zircon_runtime/asset/render-assets.md
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
  - docs/zircon_runtime/asset/assets/mesh.md
  - docs/zircon_runtime/asset/assets/scene.md
  - docs/zircon_runtime/asset/render-assets.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/zircon_runtime/asset/assets/mesh.md docs/zircon_runtime/asset/assets/scene.md docs/zircon_runtime/asset/render-assets.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-runtime-asset-doc-owner-hardcut-batch27.md
---

# Frameworks06 G7 Runtime Asset 文档 Owner 硬切 Batch 27

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/zircon_runtime/asset/assets/mesh.md", "docs/zircon_runtime/asset/assets/scene.md", "docs/zircon_runtime/asset/render-assets.md"]
Date: 2026-07-19
Session: `frameworks06-g7-runtime-asset-doc-owner-hardcut-batch27-20260719`

## Scope Delivered

- 将三份 Runtime asset 文档中 `6` 条指向已删除 `zircon_runtime/src/scene/components/scene.rs` 的机器路径硬切到 folder-backed current owners。
- Mesh 与 render-assets 文档只记录实际声明 MeshRenderer、primitive binding 与 LOD binding 的 `mesh_renderer.rs`。
- Scene asset 概览按正文持久化/投影契约记录 curated facade 与 transform、hierarchy、activation、camera、mesh renderer、lighting、physics、animation、mobility 的真实声明责任。
- 旧 flat owner 保持物理删除，不创建 alias、shim、兼容 include 或重复文档真相。

## Fresh Testing Evidence

- 修改前 fresh G7：三份目标文档共 `6` 个 missing-path violations，均指向已删除 `components/scene.rs`；同一时序全局 `593` violations / `141` documents / `67,362` checked paths。
- 修改后 fresh G7：三份目标文档与本记录 focused `0` violations；同一时序全局降到 `587` violations / `138` documents / `67,403` checked paths，继续保持 RED。
- 三份目标文档机器字段中退役 flat owner 计数为 `0`；10 个 curated facade/current owner 文件全部存在；exact-scope `git diff --check` 通过，仅输出工作树既有 LF/CRLF 提示。

## Review

- coordinator snapshot637 exact4 独立双遍复审为 `Critical 0 / Important 0 / Minor 0`、`Ready`；ordinal fingerprint `b3fba9692f83c70aa4e12c53b7c61ef3618c2d23f42577d36b209bdb337173a6`，首遍、末遍与父会话复算均一致，drift 为 `none`。

## Milestone Decision

本批 focused G7 与独立复审已通过，等待受管 milestone commit。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`，不得以局部路径迁移冒充计划完成。
