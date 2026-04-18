---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 milestones without waiting for confirmation
  - user: 2026-04-17 continue next task
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-page-table-indirection.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-indirect-raster-consumption.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_prepare_compacts_resident_segments_sharing_page_slot_into_fewer_indirect_draws --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_render --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Slot-Aware Indirect Compaction

**Goal:** 在已经落地的 `resident_slot` / page-table indirection / indirect raster baseline 之上，把 `Virtual Geometry` 的 renderer-local draw submission 再往前推进一步，让共享同一 resident page slot 的相邻 resident cluster segments 在 raster 前压成更少的 indirect draw。

**Non-Goal:** 本轮仍然不实现 GPU-generated indirect args compaction、multi-draw indirect、真正的 cluster raster、visibility-owned indirect buffer 或 page residency manager。

## Delivered Slice

- `build_virtual_geometry_cluster_raster_draws(...)` 不再把每条 prepare draw segment 直接映射成一个 raster draw；它现在会先构造带 `page_id / resident_slot / start_ordinal / span_count / total_count` 的中间表示。
- renderer 会对这些中间表示做一次 slot-aware resident compaction：
  - 只压缩 `Resident` 状态的 cluster
  - 只在 `page_id / resident_slot / lod_level / total_count` 一致时压缩
  - 只在 ordinal 连续时压缩，避免把不相邻 segment 错误拼成一条 draw
- `virtual_geometry_draw_range(...)` 现在支持用 `start_ordinal + span_count` 计算一个更大的 draw range，因此 compaction 后仍然保持同一页内的连续 cluster 覆盖。
- 不同 resident slot 的 cluster 仍然保持拆分，从而让 page-table indirection 继续成为 draw submission 的边界条件，而不是只停在 tint 或 runtime bookkeeping。

## Why This Slice Exists

- 上一轮 page-table indirection 已经让 `resident_slot` 真正进入 renderer fallback 消费面，但 draw submission 仍然是“一条 prepare segment -> 一条 indirect draw”。
- 这意味着 page-table mapping 虽然可见，却还没有真正影响 indirect submission 的结构。
- 本轮把“shared slot -> fewer indirect draws”固定下来后，后续 GPU-generated indirect compaction 或 cluster raster 才有一个更可信的替换点。

## Validation Summary

- `virtual_geometry_prepare_compacts_resident_segments_sharing_page_slot_into_fewer_indirect_draws`
  - 证明共享同一 resident slot 的相邻 resident segments 会被压成 1 条 indirect draw，而不同 slot 仍保持 2 条
- `virtual_geometry_prepare_render`
  - 证明 slot-aware compaction 没有破坏既有的 fallback filtering、slot tint、segment override、streaming coverage 与 visible-cluster spatial routing
- `cargo test -p zircon_graphics virtual_geometry --locked`
  - 证明 compaction 与 runtime host、GPU uploader、page-table snapshot readback 一致工作
- `cargo test -p zircon_graphics --lib --locked` 与 `validate-matrix.ps1 -Package zircon_graphics`
  - 证明没有回归 `zircon_graphics` 其他 M4/M5 功能族

## Remaining Route

- GPU-generated indirect args compaction，而不是当前 CPU-side renderer compaction
- multi-draw indirect / visibility-owned indirect buffers
- 更真实的 page-table indirection consumption 与 cluster raster
- 与 occlusion、BVH、Hybrid GI scene representation 的更深层联合路径
