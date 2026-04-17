---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 route without waiting for confirmation
  - user: 2026-04-17 Virtual Geometry still needs visibility-owned unified indirect / deeper cluster raster / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-generated-indirect-args.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-page-table-residency-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_resident_slot_changes_fallback_raster_output
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_gpu_generated_indirect_args_change_when_resident_slot_changes
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Slot-Aware Cluster Raster Consumption

**Goal:** 把 `resident_slot` 从 page-table / stats 侧信息继续推进到真实的 GPU indirect args 与 cluster-raster consumption，让不同 slot ownership 不只改变调试 tint，而会改变实际的 fallback raster 子范围。

**Non-Goal:** 本轮仍然不实现真正的 visibility-owned unified indirect buffer、multi-draw indirect、Nanite-like cluster rasterizer 或完整 residency manager。

## Delivered Slice

- `VirtualGeometryIndirectArgsInput` 现在会显式携带 `resident_slot`，不再只上传 segment span/state。
- `virtual_geometry_indirect_args.wgsl` 新增 resident-slot-aware trim 规则：
  - 只对 `Resident` 状态生效
  - 只在 segment 至少覆盖多个 triangle 且 slot 落入更高 band 时裁剪
  - 最终通过改变 `first_index / index_count` 让高 slot ownership 的 cluster fallback 消费不同的 mesh 子范围
- 离屏回归不再只检查颜色亮度，而是同时验证 raster coverage / output 差异，确保 slot ownership 已进入真正的 draw consumption。

## Why This Slice Exists

- 之前的 page-table / slot assignment 路线已经让 `resident_slot` 成为 runtime host 与 GPU uploader 的真实状态，但 renderer fallback 仍主要把它当成 tint/brightness 的附属信号。
- 如果 slot ownership 不参与真实的 indirect args 生成，后续 unified indirect ownership、cluster raster 与 residency-manager 仍然缺少可替换的真实消费点。
- 本轮先把 slot 带进 GPU-generated args，后续再继续推进 unified indirect / deeper cluster raster 时，已经有了“slot changes -> raster changes”的可验证边界。

## Validation Summary

- `virtual_geometry_prepare_resident_slot_changes_fallback_raster_output`
  - 证明不同 `resident_slot` 会改变最终 fallback raster 输出与 coverage，而不是只改变调试色偏
- `virtual_geometry_prepare_gpu_generated_indirect_args_change_when_resident_slot_changes`
  - 直接读回 GPU indirect args，证明高 slot ownership 会改变 `first_index / index_count`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render`
  - 证明 slot-aware cluster-raster consumption 没有破坏已有的 filtering、segment override、visible-cluster routing、shared buffer reuse 与 streaming coverage
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明这条 renderer-side slice 与 runtime host、GPU uploader、page-table snapshot 主链兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 renderer / shader / tests 的边界仍然闭合

## Remaining Route

- 把 slot-aware raster consumption 继续推进到真正的 visibility-owned unified indirect ownership，而不是 renderer build step 的末端收口
- 把 page residency / slot truth 与更深层 cluster raster / split-merge frontier 连接起来
- 继续走向 GPU-driven indirect compaction、multi-draw submission 与 Nanite-like cluster raster execution
