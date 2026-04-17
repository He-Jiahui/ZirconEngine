---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry unified indirect ownership downshift or wider split-merge policy
  - user: 2026-04-18 continue M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-aware-indirect-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_frame_preserves_explicit_draw_segment_boundaries_in_unified_indirect_draws
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_builds_visibility_owned_compacted_draw_segments
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_preserves_explicit_segment_boundaries_even_when_segments_share_page_slot
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model
doc_type: milestone-detail
---

# M5 Virtual Geometry Prepare-Owned Unified Indirect Boundaries

**Goal:** 把 `Virtual Geometry` 的 unified indirect ownership 真正下沉到 `prepare` snapshot，使 `cluster_draw_segments` 成为最终 submission authority，而不是再让 renderer 在 `unified_indirect_draws()` 里做一轮隐式 regroup。

**Non-Goal:** 本轮仍然不实现真正的 visibility-owned unified indirect buffer 资产、GPU multi-draw compaction、cluster streaming residency manager 或 Nanite-like cluster rasterizer。

## Delivered Slice

- `VirtualGeometryPrepareFrame::unified_indirect_draws()` 不再尝试把相邻 prepare segments 重新合并。
- 这条转换现在只做两件事：
  - 过滤 `Missing` segment
  - 在旧 helper 仍然写 `page_id = 0 / resident_slot = None` 时，从 `visible_clusters` 回填 page/slot ownership
- 真正的 compaction authority 被固定在 `runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs`：
  - runtime host 仍然会把连续、同 ownership 的 visible cluster 收口成更少的 `cluster_draw_segments`
  - 但如果 prepare 明确给出两条 segment，renderer 就必须保持两条 submission
- renderer 侧的 `build_virtual_geometry_cluster_raster_draws(...)` 继续消费 `prepare.unified_indirect_draws()`，但这一步现在已经是单纯的 prepare snapshot 投影，而不是 renderer-local policy。

## Why This Slice Exists

- 之前的 `page-aware indirect ownership` 只把 `page_id` 补回 unified draw compaction 边界，但 renderer 仍然保留“看见两个相邻 segment 就可以再合并一次”的权力。
- 这会让 `cluster_draw_segments` 变成不稳定 contract：
  - upstream visibility/runtime 想显式 split submission
  - renderer 却还可能因为 page/slot/state 连续而把它重新压回一条
- 对真正的 visibility-owned unified indirect 路线来说，这种二次 regroup 会继续把 authority 留在 renderer，而不是留在 `prepare` 前处理阶段。
- 本轮把 authority 固定在 prepare snapshot 后，后续更深的 cluster raster / streaming / residency policy 可以只改 `cluster_draw_segments` 的生成规则，不需要再改 renderer submission 逻辑。

## Validation Summary

- `virtual_geometry_prepare_frame_preserves_explicit_draw_segment_boundaries_in_unified_indirect_draws`
  - 证明相邻、同 page/slot 的显式 prepare segments 不会再被 `unified_indirect_draws()` 二次合并
- `virtual_geometry_runtime_state_builds_visibility_owned_compacted_draw_segments`
  - 证明真正的 compaction 仍然留在 runtime prepare 阶段，而没有丢掉原本的 visibility-owned compact baseline
- `virtual_geometry_prepare_preserves_explicit_segment_boundaries_even_when_segments_share_page_slot`
  - 证明 renderer integration 现在会尊重 prepare 显式 segment 边界
- `virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model`
  - 证明 multi-primitive shared indirect buffer 仍然会复用同一条 prepare-owned segment，而不是退化成 per-primitive ownership

## Remaining Route

- 把 prepare-owned unified ownership 继续推进到真正的 unified indirect buffer 资产，而不是仍靠 renderer build step 转译成 pending draw refs
- 把更宽的 split-merge policy、cluster streaming / residency-manager frontier 继续压到 visibility/runtime 前处理层
- 继续朝 GPU-driven indirect compaction、cluster raster consumption 与 Nanite-like execution 推进
