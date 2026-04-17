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
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-aware-cluster-raster-consumption.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-aware-indirect-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_gpu_generated_indirect_args_change_when_page_id_changes_inside_same_resident_slot
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_gpu_generated_indirect_args
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Page-Owned Cluster Raster Consumption

**Goal:** 把 `Virtual Geometry` 的 `page_id` 从 “只参与 unified indirect ownership 边界” 再推进到真实 GPU indirect args consumption，让不同 resident page 即使暂时共享同一 slot，也会消费不同的 cluster-raster 子范围。

**Non-Goal:** 本轮仍然不实现真正的 visibility-owned indirect buffer 资产、GPU multi-draw compaction、Nanite-like cluster rasterizer 或完整 residency-manager 级联。

## Delivered Slice

- `VirtualGeometryIndirectSegmentInput` 现在会显式携带 `page_id`，不再把页级所有权截断在 renderer-side segment dedup 之前。
- `virtual_geometry_indirect_args.wgsl` 新增 page-owned offset 规则：
  - 只对 `Resident` 状态生效
  - 只在更高 `resident_slot` band 的真实 cluster-raster path 上参与
  - 会在 resident-slot trim 之后继续按 `page_id` 调整 `first_index / index_count`
- 这意味着：
  - `page_id` 现在不仅决定 “是不是两条 draw”
  - 还决定 “即使 slot 一样，这条 draw 真正消费 mesh 的哪一段”

## Why This Slice Exists

- `page-aware indirect ownership` 已经保证不同 resident page 不会被错误压成同一条 indirect draw，但那还只是 submission ownership。
- 如果 GPU indirect args 仍然完全不看 `page_id`，那么 “不同 page 同 slot” 只是在 draw 数量上分离，实际 raster consumption 仍然是同一段 mesh 子范围。
- 这会让更深的 cluster raster / residency-manager / split-merge 路线继续停留在伪 page ownership 上。
- 本轮把 `page_id` 接进真实 GPU args 后，renderer 已经具备了 “page ownership changes -> raster subrange changes” 的可验证消费点。

## Validation Summary

- `virtual_geometry_prepare_gpu_generated_indirect_args_change_when_page_id_changes_inside_same_resident_slot`
  - 证明不同 resident page 即使共享同一 slot，也会生成不同的 GPU indirect args
- `virtual_geometry_prepare_gpu_generated_indirect_args`
  - 证明新的 page-owned offset 没有破坏既有的 visibility-owned span 与 resident-slot routing 回归
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明这条 deeper cluster-raster consumption 与 runtime host、page-table residency、hierarchy refine、shared indirect buffer 主链兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 segment input / shader / test contract 仍然闭合

## Remaining Route

- 把当前 page-owned GPU args 再继续推进到真正的 visibility-owned unified indirect buffer，而不是仍由 renderer build step 做最后聚合
- 给 `Virtual Geometry` 增加更完整的 split-merge hysteresis，而不是只停在 upload-completion split hold
- 继续走向 GPU-driven indirect compaction、cluster streaming 与 Nanite-like cluster raster execution
