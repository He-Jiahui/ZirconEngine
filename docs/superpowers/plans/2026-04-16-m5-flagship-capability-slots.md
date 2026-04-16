---
related_code:
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/extract/history.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/culling/mod.rs
  - zircon_graphics/src/visibility/culling/is_mesh_visible.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_runtime_feature_flags.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/extract/history.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/culling/mod.rs
  - zircon_graphics/src/visibility/culling/is_mesh_visible.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_runtime_feature_flags.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
plan_sources:
  - user: 2026-04-16 continue next step after M4
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
tests:
  - zircon_render_server/src/tests.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_scene --locked
  - cargo test -p zircon_graphics compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features --locked
  - cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_render_server --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Flagship Capability Slots Plan

**Goal:** 在不假装实现 Nanite/Lumen 本体的前提下，把 `Virtual Geometry` 与 `Hybrid GI` 作为真正的 opt-in `RenderFeature` 家族接进当前 SRP/RHI/RenderServer 架构，让它们拥有稳定的 extract 合同、quality profile 开关、capability gate、pipeline compile 行为与可观测 stats。

**Architecture:** 本轮只做旗舰路径的第一层“边界预埋”，不做真实 page table、cluster streaming、radiance cache 或 RT hybrid lighting。`Virtual Geometry` 与 `Hybrid GI` 会：

- 在 `zircon_scene` 里拿到默认关闭的 extract 槽位
- 在 `zircon_render_server` 里拿到默认关闭的 quality/profile 开关
- 在 `zircon_graphics` 里拿到 built-in feature descriptor、history slot、opt-in compile path
- 在 `WgpuRenderServer` 里被 capability gate 强制关闭，证明纯 `wgpu` 基线下无法被 profile 强行打开

## Status

当前状态：M5 第一段 capability-slot 切片已完成并通过验证。

- `Virtual Geometry` 与 `Hybrid GI` 已经成为 compile/profile/stats 空间里的真实 opt-in feature
- 当前 headless `wgpu` 仍然会把两者 cleanly gate 为关闭
- 真实 page table、cluster streaming、radiance cache、screen probe、RT hybrid lighting 仍未开始
- 其中 `Virtual Geometry` 的下一条 preprocess/runtime-stats 切片已经转入 `docs/superpowers/plans/2026-04-16-m5-virtual-geometry-preprocess.md`

## Task 1: Lock M5 Opt-In Behavior With Tests

- [x] Step 1: Add red tests for extract default slots.
  - `RenderFrameExtract::from_snapshot(...)` must initialize `geometry.virtual_geometry = None`
  - `lighting.hybrid_global_illumination = None`

- [x] Step 2: Add red tests for compile opt-in.
  - `RenderPipelineCompileOptions::with_feature_enabled(...)` should be able to opt in `VirtualGeometry` and `GlobalIllumination`
  - default Forward+ compile must keep them absent

- [x] Step 3: Add red tests for façade capability gate.
  - headless `wgpu` stats should expose `virtual_geometry_supported = false`
  - headless `wgpu` stats should expose `hybrid_global_illumination_supported = false`
  - quality profile opt-in must not force these features into `last_effective_features`

## Task 2: Extend Public Contracts

- [x] Step 1: Add `RenderVirtualGeometryExtract`
- [x] Step 2: Add `RenderHybridGiExtract`
- [x] Step 3: Thread them into `GeometryExtract` and `LightingExtract`
- [x] Step 4: Add `RenderFeatureQualitySettings.virtual_geometry`
- [x] Step 5: Add `RenderFeatureQualitySettings.hybrid_global_illumination`
- [x] Step 6: Add builder methods on `RenderQualityProfile`
- [x] Step 7: Extend `RenderCapabilitySummary` with flagship support booleans

## Task 3: Wire Feature And History Skeleton

- [x] Step 1: Add `FrameHistorySlot::GlobalIllumination`
- [x] Step 2: Add `BuiltinRenderFeature::VirtualGeometry` pass descriptor
- [x] Step 3: Add `BuiltinRenderFeature::GlobalIllumination` pass descriptor + GI history binding
- [x] Step 4: Mark flagship features as explicit opt-in in pipeline compile logic

## Task 4: Connect Capability Gate

- [x] Step 1: Extend `RenderPipelineCompileOptions` with explicit opt-in support
- [x] Step 2: Let `compile_options_for_profile(...)` enable M5 features only when:
  - profile flag is on
  - capability summary says support is available
- [x] Step 3: Keep built-in default pipeline compile results unchanged when no opt-in is requested

## Task 5: Validate And Document

- [x] Step 1: Update rendering architecture docs with M5 first-slice boundaries
- [x] Step 2: Run:
  - `cargo test -p zircon_scene --locked`
  - `cargo test -p zircon_graphics compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features --locked`
  - `cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked`
  - `cargo test -p zircon_graphics pipeline_compile --locked`
  - `cargo test -p zircon_graphics render_server_bridge --locked`
  - `cargo test -p zircon_graphics --lib --locked`
  - `cargo test -p zircon_render_server --locked`
- [x] Step 3: Run validator:
  - `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`

## Validation Notes

- 新增红测已经证明：
  - 默认 `RenderFrameExtract::from_snapshot(...)` 会把两条 flagship extract 槽位保持为 `None`
  - `RenderPipelineCompileOptions::with_feature_enabled(...)` 可以显式编译 `virtual-geometry-prepare` 与 `hybrid-gi-resolve`
  - headless `wgpu` 的 `RenderStats.capabilities` 会把 `virtual_geometry_supported` 与 `hybrid_global_illumination_supported` 报告为 `false`
  - quality profile 即使请求开启，也不会把两条旗舰 feature 写进 `last_effective_features`
- 全量 `cargo test -p zircon_graphics --lib --locked` 在本轮还暴露了 `visibility/culling` 与 `visibility/planning` 的模块可见性问题。该问题已通过显式模块路径修复，并顺手把 `is_mesh_visible(...)` 改成 `transform_point3(...)`，从而恢复全量回归。

## Remaining M5 Route

- `Virtual Geometry`
  - cluster/page 数据结构
  - streaming residency 与 feedback
  - visibility/LOD/page table runtime
- `Hybrid GI`
  - scene representation
  - radiance cache / screen probe gather
  - RT hybrid lighting 与 capability-tier fallback

## Completion Gate

- `Virtual Geometry` and `Hybrid GI` are real opt-in `RenderFeature` families in compile/profile space
- `RenderServer` stats expose whether the current backend can support them
- pure `wgpu` baseline keeps them disabled even when quality profile tries to opt in
- extract, pipeline, render server, docs, and tests all agree on the same capability-gated boundary
