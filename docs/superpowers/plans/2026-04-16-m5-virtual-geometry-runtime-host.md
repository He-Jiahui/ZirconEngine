---
related_code:
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/runtime/server/create_viewport.rs
  - zircon_graphics/src/runtime/server/viewport_record.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/runtime/server/create_viewport.rs
  - zircon_graphics/src/runtime/server/viewport_record.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_render_server/src/types.rs
plan_sources:
  - user: 2026-04-16 continue M5 Virtual Geometry runtime host slice
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-preprocess.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/tests.rs
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_tracks_page_table_and_request_sink --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_deduplicates_requests_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_render_server --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Runtime Host Plan

**Goal:** 把 `Virtual Geometry` 从“只有前处理 page 计划”推进到一个真正存在的 CPU 侧 runtime host，让 viewport 维持 page table、resident page、pending request 与 evictable page 的长期状态宿主，同时继续保持 capability-gated，不伪装真实 GPU uploader 已经完成。

**Non-Goal:** 本轮不实现 GPU page upload backend、feedback DMA/readback、cluster hierarchy refine、Nanite raster、meshlet decode、page streaming I/O 或任何真实 Hybrid GI 行为。

## Delivered Slice

- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 现在成为每个 viewport 的 Virtual Geometry runtime host。
- `ViewportRecord` 新增 `virtual_geometry_runtime`，render server submit 路径会在 `VirtualGeometry` feature 真正进入有效 compiled pipeline 时维护它。
- `RenderStats` 新增三项 runtime-host 可观测计数：
  - `last_virtual_geometry_page_table_entry_count`
  - `last_virtual_geometry_resident_page_count`
  - `last_virtual_geometry_pending_request_count`

## Runtime Contract

- `register_extract(Some(&RenderVirtualGeometryExtract))`
  - 记录 page `size_bytes`
  - 为 extract 中标记 `resident = true` 的页分配稳定 slot
  - 已经 resident 的页保持原 slot，不做重排
- `ingest_plan(generation, &VisibilityVirtualGeometryPageUploadPlan)`
  - 把 `resident_pages` 提升或保持为 resident
  - 只对 `dirty_requested_pages` 里的 missing page 生成一次 pending request
  - 保留当前帧的 `evictable_pages`
- `apply_evictions(...)`
  - 回收 resident page 的 slot
  - 释放出的 slot 会被后续 fulfill/reload 按最小空闲 slot 优先复用
- `fulfill_requests(...)`
  - 把 pending page 升格为 resident
  - 清除对应 request，并复用可用 slot

## Capability And Server Behavior

- runtime host 不会越过 capability gate。当前 headless `wgpu` 基线依然会让 `VirtualGeometry` feature 保持关闭，因此 façade 统计仍然全部回落到 `0`。
- 只有当 compiled pipeline 的 `enabled_features` 中真正包含 `BuiltinRenderFeature::VirtualGeometry` 时，`submit_frame_extract(...)` 才会：
  - 建立或复用 viewport 级 runtime host
  - 注册 extract page metadata
  - 吞入本帧 visibility 产生的 page upload 计划
  - 把 runtime host 计数写回 `RenderStats`

## Validation Summary

- `virtual_geometry_runtime_state_tracks_page_table_and_request_sink`
  - 证明 resident baseline page 会拿到稳定 slot，dirty missing page 会转成 pending request
- `virtual_geometry_runtime_state_deduplicates_requests_and_reuses_evicted_slots`
  - 证明 repeated request 不会重复入队，eviction 后 slot 会被 fulfill 的页复用
- `headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features`
  - 证明 capability gate 关闭时，新老 Virtual Geometry 统计都稳定保持 `0`
- `cargo test -p zircon_render_server --locked`
  - 证明 `RenderStats` 扩展字段在 façade crate 默认构造与 builder 测试中保持兼容

## Remaining Route

- GPU uploader / request sink backend 与 feedback consumer 的真实实现
- cluster hierarchy / parent-child refinement / SSE-driven split-merge
- `virtual-geometry-prepare` pass 真正消费 runtime host 的 resident/request/evictable 状态
- 与 indirect draw、instance upload、occlusion、BVH/AS update 的更深层统一
- Hybrid GI 的 scene representation / radiance cache / RT hybrid lighting 路线
