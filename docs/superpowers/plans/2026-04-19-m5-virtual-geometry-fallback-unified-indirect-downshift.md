---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-19 继续把这套 fallback/full-mesh authority 从 CPU mesh draws 绑定 authoritative key 压进更真实的 visibility-owned / GPU-generated args compaction
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-fallback-full-mesh-cluster-authority.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-prepare-owned-args-source-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect_synthesizes_fallback_cluster_slices_when_segments_are_absent -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Fallback Unified-Indirect Downshift

## Goal

把 “没有显式 `cluster_draw_segments` 的 full-mesh fallback” 从 renderer 末端的 CPU fallback side-path 继续下沉到 `VirtualGeometryPrepareFrame::unified_indirect_draws()` 自身，让同一份 prepare-owned unified-indirect truth 同时喂给：

- `build_virtual_geometry_cluster_raster_draws(...)`
- shared `segment_buffer / draw_ref_buffer / indirect args`
- 最终 `MeshDraw -> draw_indexed_indirect(...)` 提交顺序

## Delivered Slice

### 1. `unified_indirect_draws()` 现在会为 missing-segment entity 合成 fallback cluster slices

当 entity 出现在 `visible_entities` 中、拥有 `visible_clusters`，但当前帧没有显式 `cluster_draw_segments` 时，`VirtualGeometryPrepareFrame::unified_indirect_draws()` 不再返回空集。

它现在会：

- 读取 `visible_clusters`
- 结合 `resident_pages + pending_page_requests`
- 收束出 `submission_slot / frontier_rank / page_id / lod / state`
- 按 authoritative cluster order 生成一条或多条 `VirtualGeometryPrepareIndirectDraw`

这条排序与 renderer 侧已有 fallback authority 一致：

- 先按 `submission_slot`
- 再按 `frontier_rank`
- 再按 visible/entity/cluster ordinal 稳定 tie-break

### 2. fallback full-mesh authority 现在进入了 prepare-owned mainline，而不是只停在 mesh-build fallback key

这意味着 missing-segment fallback 不再主要依赖：

- `build_mesh_draws(...)` 内部的 `authoritative_fallback_segment_keys`
- `extend_pending_draws_for_mesh_instance(...)` 的 later CPU fallback expansion

相反，当前主链已经变成：

- `prepare.unified_indirect_draws()`
- `build_virtual_geometry_cluster_raster_draws(...)`
- `build_context.virtual_geometry_cluster_draws`
- authoritative `segment_key / draw_ref / indirect args`
- actual submission

CPU fallback side-path 现在只剩下更窄的兜底职责，例如当前帧根本没有可复用的 prepare cluster truth 时。

### 3. fallback cluster slices 继续和真实 unified-indirect contract 对齐

新合成的 fallback draw 现在显式保留：

- `cluster_start_ordinal`
- `cluster_total_count`
- `submission_slot`
- `frontier_rank`
- `page_id`
- `state`

因此缺少显式 `cluster_draw_segments` 时，fallback 也会像显式 prepare segment 一样被 GPU-generated indirect args 和真实 cluster-raster consumption 消费，而不再是 renderer 末端自造的一次性 key。

## Why This Slice Matters

上一刀已经做到两件事：

- fallback full-mesh 不再退回 `page 0 / slot 0 / Resident`
- shared `segment_buffer / draw_ref_buffer / indirect args` 会保留 per-cluster fallback truth

但那条 truth 主要仍是 renderer mesh-build 阶段临时合成出来的。

这会留下一个 execution leak：

- prepare / visibility 有 unified-indirect authority
- 缺失显式 segments 的 fallback 仍然要等 renderer 末端补一遍

本轮把 fallback cluster slices 继续前移到 `unified_indirect_draws()` 之后，`Virtual Geometry` 才真正更接近“visibility-owned unified indirect authority”：

- prepare 先给出 truth
- cluster-raster draws 直接消费 truth
- renderer 末端不再发明第一份 fallback indirect ownership

## Validation Summary

- red -> green
  - `virtual_geometry_unified_indirect_synthesizes_fallback_cluster_slices_when_segments_are_absent`
- focused regressions
  - `virtual_geometry_args_source_authority`
  - `virtual_geometry_unified_indirect`
  - `virtual_geometry_submission_execution_order`
  - `virtual_geometry_prepare_render`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 把这条 downshift 继续推进到更真实的 visibility-owned / GPU-generated args compaction source，而不只是 “prepare 已经给出 draw truth，renderer 继续消费它”
- 继续推进 deeper cluster-raster consumption，让同一套 fallback/unified-indirect truth 更完整地进入 GPU-driven cluster raster execution
- 再往下就是更深的 residency-manager cascade / split-merge frontier policy
