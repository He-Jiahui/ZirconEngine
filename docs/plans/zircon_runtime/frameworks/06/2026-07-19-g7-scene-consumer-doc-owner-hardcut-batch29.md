---
related_code:
  - docs/zircon_editor/scene/viewport/edit_mode_projection.md
  - docs/zircon_plugins/rendering-plugin-options.md
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
  - docs/zircon_editor/scene/viewport/edit_mode_projection.md
  - docs/zircon_plugins/rendering-plugin-options.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/zircon_editor/scene/viewport/edit_mode_projection.md docs/zircon_plugins/rendering-plugin-options.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-scene-consumer-doc-owner-hardcut-batch29.md
---

# Frameworks06 G7 Scene Consumer 文档 Owner 硬切 Batch 29

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/zircon_editor/scene/viewport/edit_mode_projection.md", "docs/zircon_plugins/rendering-plugin-options.md"]
Date: 2026-07-19
Session: `frameworks06-g7-scene-consumer-doc-owner-hardcut-batch29-20260719`

## Scope Delivered

- 将 Editor scene projection 与 rendering-plugin options 两份 durable consumer 文档中的 `3` 条已删除 `zircon_runtime/src/scene/components/scene.rs` 机器路径硬切到 folder-backed current owners。
- Editor projection 文档按其 hierarchy、transform、activation、camera、mesh/light、physics、animation 与 mobility 查询契约记录 curated scene facade/current leaves；rendering-plugin options 只记录其 Mesh LOD scene data 实际依赖的 `mesh_renderer.rs`。
- 旧 flat owner 保持物理删除，不创建 alias、shim、兼容 include 或重复聚合 owner；本批不修改 Runtime、Render 或 Editor 行为。

## Fresh Testing Evidence

- 修改前 fresh G7：两份目标文档共 `3` 个 missing-path violations，均指向已删除 `components/scene.rs`；同一时序全局 `590` violations / `139` documents / `67,485` checked paths。
- 修改后 fresh G7：两份目标文档与本记录 focused `0` violations；同一时序全局降到 `587` violations / `137` documents / `67,508` checked paths，继续保持 RED。
- 两份目标文档机器字段中退役 flat owner 计数为 `0`；10 个 curated facade/current owner 文件全部存在；exact-scope `git diff --check` 通过，仅输出工作树既有 LF/CRLF 提示。

## Review

- coordinator snapshot `656` exact3 独立双遍复审为 `C0 / I0 / M0 — Ready`；record `f1ce75ad9666a3912ddc4acdc33f854146c82a411abf28ccefc74f65f0b1a3c1`、Editor doc `24c1a8d7f15dd8db4bed932f6de4a88649eaa525da77ba620294dc83e8a63eff`、plugin doc `d42cb69381c99c86ea6056d6e65a62e191ea4a8b84f2bf4a804fabfbe57d8be0` 与 ordinal fingerprint `034ae368693ce38d50cc68e74ea3d3059e6a3e3f2f7b850858140b79f0dbaeb0` 两遍稳定无漂移。
- 复审确认 Editor 10/10 facade/leaf owner 与 projection 字段契约一致，`mesh_renderer.rs` 是 Mesh LOD scene data 的最窄 current owner；旧 flat 文件保持不存在，未引入 alias、shim、兼容 include 或 legacy forwarding。

## Milestone Decision

本批 focused G7 与独立复审已通过，状态记为 `accepted`，等待通过协调器纳入 Frameworks06 管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`，不得以局部文档路径迁移冒充里程碑完成。
