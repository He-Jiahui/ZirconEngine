---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-17 continue M5
  - user: 2026-04-17 Virtual Geometry still needs visibility-owned unified indirect / deeper cluster raster / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-aware-indirect-compaction.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-page-table-residency-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_keeps_different_pages_in_same_slot_split_into_separate_indirect_draws
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Page-Aware Indirect Ownership

**Goal:** 把 `Virtual Geometry` 的 unified indirect draw compaction 从“entity + slot + ordinal”推进到真正保留 `page_id` 边界，避免不同 resident page 因为暂时共享同一 slot 且 ordinal 连续而被错误压成一条 indirect draw。

**Non-Goal:** 本轮仍然不实现真正的 visibility-owned unified indirect buffer 资产、GPU multi-draw compaction、cluster rasterizer 或完整 residency manager 级联。

## Delivered Slice

- `VirtualGeometryPrepareIndirectDraw` 现在显式保留 `page_id`，不再在 `cluster_draw_segments -> unified_indirect_draws()` 过程中把页级所有权丢掉。
- `VirtualGeometryPrepareFrame::unified_indirect_draws()` 的 compaction 边界新增 `page_id`：
  - 只有同 `entity`
  - 同 `page_id`
  - 同 `cluster_total_count / lod_level / resident_slot / state`
  - 并且 ordinal 连续
  的 segment 才允许继续压成一条 indirect draw
- 为了兼容旧 helper/测试里使用 `page_id = 0` 的 prepare segment，unified compaction 还会从 `visible_clusters` 回填 page ownership 作为兜底。
- renderer-local `VirtualGeometryClusterRasterDraw` 也同步保留 `page_id`，把页级边界继续传给 mesh draw build 层，而不是重新退化成 slot-only 视图。

## Why This Slice Exists

- slot-aware indirect compaction 把 `resident_slot` 正式带进了 draw submission 结构，但它仍然可能把“不同 page 只是暂时映射到同一 slot”的 segment 当成同一 ownership 区间。
- 这会在 residency churn、page-table rewrite 或 future residency-manager 继续推进时制造错误的 unified draw 边界。
- 如果 page ownership 在 unified indirect 之前就丢掉，后续所谓 visibility-owned unified indirect buffer 就仍然是伪命题，因为 renderer 已经看不见页级 authority。
- 本轮先把 `page_id` 补回 compaction contract，给后续 unified indirect / deeper cluster raster 留下可信的 ownership 边界。

## Validation Summary

- `virtual_geometry_prepare_keeps_different_pages_in_same_slot_split_into_separate_indirect_draws`
  - 证明不同 resident page 即使暂时共享同一 slot，也会保持两条独立 indirect draw
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render`
  - 证明新增 page-aware boundary 没有破坏 filtering、streaming coverage、segment override、resident-slot raster consumption 与 shared-buffer 复用
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明 unified compaction contract 扩展与 runtime host、GPU uploader、page-table snapshot、hierarchy refine 主链兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 `prepare -> renderer` 这条 ownership 扩展没有留下 crate 编译缺口

## Remaining Route

- 把当前 page-aware compaction 继续推进到真正的 visibility-owned unified indirect buffer，而不是仍由 renderer build step 做最后收口
- 把 page ownership 与更深层 cluster raster / split-merge frontier / residency-manager cascade 连接起来
- 继续朝 GPU-driven indirect compaction、cluster streaming 和 Nanite-like execution 迈进
