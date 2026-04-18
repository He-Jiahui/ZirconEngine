---
related_code:
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_gpu_resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_gpu_resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 milestones without waiting for confirmation
  - user: 2026-04-17 Virtual Geometry should continue from refine and uploader baseline into cluster streaming or indirect raster consumption
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-indirect-raster-consumption.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_merges_gpu_completed_assignments_into_page_table_snapshot --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_resident_slot_changes_fallback_raster_output --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_render --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Page Table Indirection

**Goal:** 把 `Virtual Geometry` 的 page-table ownership 再往 renderer/GPU 消费面推进一步，不再只让 GPU readback 提供 `completed_page_assignments`，而要让同一帧 readback 就包含 post-upload page-table snapshot，并让 renderer fallback raster 真正消费 `resident_slot` 这类 page-table indirection 数据。

**Non-Goal:** 本轮仍然不实现真正的 page residency manager、GPU-generated indirect command compaction、Nanite-like cluster raster、page streaming copy queue 或统一的 visibility-owned indirect args buffer。

## Delivered Slice

- `VirtualGeometryGpuResources::execute_prepare(...)` 现在把 page table 变成真正的 GPU write-back 资源：
  - page table buffer 先用当前 resident `(page_id, slot)` 初始化
  - uploader shader 在完成 upload 时，会在同一帧把新完成的 `(page_id, slot)` 合并回 page table buffer
  - 如果 assignment 复用了 evictable resident slot，page table 会直接覆盖旧页，而不是把重复 slot 追加成脏快照
- `VirtualGeometryGpuPendingReadback::collect(...)` 现在不再把 `page_table_entries` 当成“上传前 resident snapshot”的回显，而是按 completed assignments 计算最终有效 entry 数，从 post-uploader buffer 里还原当前页表视图。
- `build_mesh_draws(...)` 现在不再把 `resident_slot` 当成 prepare 里的闲置字段：
  - renderer 会从 `prepare.visible_clusters` 取回 `(entity, cluster_id) -> resident_slot`
  - resident cluster 的 fallback tint 现在会带入 slot phase
  - 同一 cluster draw segment 在不同 resident slot 下会产生不同的 fallback raster 输出，说明 page-table indirection 已经真正影响 renderer 消费面

## Why This Slice Exists

- 上一轮 slot assignment ownership 已经把“GPU 决定页落在哪个 slot”固定下来，但 `page_table_entries` 读回仍然只是上传前 resident snapshot，renderer 侧也还没有真正消费 `resident_slot`。
- 这会导致 page-table ownership 虽然在 runtime host 内部成立，却无法形成 renderer-side 的可观测、可继续扩展的 indirection baseline。
- 本轮把“post-upload page-table snapshot + resident-slot-aware fallback raster”补齐后，后续 page residency manager、cluster streaming consumption、GPU-generated indirect compaction 才有一个可信的替换点。

## Runtime And Renderer Contract

- runtime host 仍然优先消费 `completed_page_assignments` 推进 resident page table；这条 contract 没有被新 slice 改坏。
- renderer readback 现在额外保证：
  - `page_table_entries` 反映的是本帧 uploader 运行后的页表快照
  - reused slot 会替换旧页
  - newly available slot 会把新页追加到页表快照末尾
- fallback raster 仍然只是一条 renderer-local placeholder baseline，但它已经显式依赖 `resident_slot`，不再把 page-table data 当成 runtime-only bookkeeping。

## Validation Summary

- `virtual_geometry_gpu_uploader_readback_merges_gpu_completed_assignments_into_page_table_snapshot`
  - 证明同一帧 GPU readback 的 `page_table_entries` 已经包含新完成的 `(page_id, slot)`
- `virtual_geometry_gpu_uploader_readback_reports_completed_page_ids_from_prepare_snapshot`
  - 证明复用 evictable slot 时，page table snapshot 会反映“旧页被替换、新页入驻”的结果，而不是继续回显旧 resident 页
- `virtual_geometry_prepare_resident_slot_changes_fallback_raster_output`
  - 证明 renderer fallback raster 已经真正消费 `resident_slot`
- `cargo test -p zircon_graphics virtual_geometry --locked`
  - 证明 page-table snapshot uplift 与 slot-aware fallback 没有破坏已有 runtime host、GPU uploader、refine frontier 与 indirect raster 基线
- `cargo test -p zircon_graphics --lib --locked` 与 `validate-matrix.ps1 -Package zircon_graphics`
  - 证明本轮改动没有回归 `zircon_graphics` 其余 M4/M5 功能族

## Remaining Route

- page-table indirection 的更真实 raster / material / cluster-data consumption，而不是当前 slot-phase placeholder tint
- GPU-generated indirect command compaction 与 visibility-owned indirect args buffer
- 真正的 page streaming / residency manager / async copy queue orchestration
- Nanite-like cluster raster 与更深层 split-merge hierarchy refinement
