---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 下一条最值当继续的是把 frontier/order 真值继续下沉到更真实的 GPU submission / deeper cluster raster consumption
  - user: 2026-04-18 切到 Virtual Geometry，当前活动任务是 unified indirect / residency-manager cascade / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-frontier-priority-and-active-request-lineage.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Frontier Rank GPU Submission And Cluster Raster Consumption

**Goal:** 把上一轮已经落到 runtime host 的 `requested_pages` frontier 顺序继续下沉到真实 GPU submission / cluster-raster consumption，而不只停在 CPU-side pending uploader queue 与 eviction ordering。

**Non-Goal:** 本轮不改写 uploader compute pass、本轮不把 unified indirect authority 直接改成 GPU-generated compaction、本轮不展开更深的 residency-manager cascade 或 split-merge hysteresis 策略。

## Delivered Slice

### 1. Prepare 现在会把 pending request order 投影成 segment frontier rank

- `VirtualGeometryPrepareFrame::unified_indirect_draws()` 新增一层 `request_order_by_page` 投影：
  - 从 `pending_page_requests` 顺序提取当前 runtime frontier rank
  - 把这条 rank 真值写进 `VirtualGeometryPrepareIndirectDraw.frontier_rank`
- 这样 `frontier/order` 不再只是留在 runtime host 的 queue policy；它已经进入 frame-local prepare submission contract。

### 2. Frontier rank 继续下沉到真实 GPU submission segment buffer

- `VirtualGeometryClusterRasterDraw`
- `VirtualGeometryIndirectSegmentKey`
- `VirtualGeometryIndirectSegmentInput`

现在都会显式保留 `frontier_rank`。

- `build_shared_indirect_args_buffer(...)` 提交给 compute pass 的 segment buffer 已经携带这条字段。
- `read_last_virtual_geometry_indirect_segments()` 也从 8-word 解码扩成 9-word 解码，因此回归可以直接断言真实 GPU-submitted segment buffer 内的 frontier rank。

这意味着 pending request frontier 顺序已经不是“只有 runtime 自己知道”的临时排序，而是真实 submission authority 的一部分。

### 3. Frontier rank 现在会改变真实 cluster-raster consumption

- `virtual_geometry_indirect_args.wgsl` 新增 `frontier_rank_cluster_trim(...)`
- 这条 trim 会在已有的：
  - `resident_slot`
  - `page_id`
  - `lineage_depth`
  - `lod_level`

之后，继续裁剪 segment 的有效 triangle span。

- 结果是：
  - 较早 request rank 继续保留更完整的 cluster span
  - 较晚 request rank 会被额外 trim
  - 同一 `page_id / slot / state / lineage_depth / lod_level` 下，只要 frontier rank 改变，GPU indirect args 与最终 raster coverage 就会跟着变化

### 4. 回归从 readback 扩到真实 frame output

- `virtual_geometry_unified_indirect_keeps_pending_request_frontier_rank_in_gpu_submission_and_indirect_args`
  - 直接断言 GPU indirect args 与 GPU-submitted segment readback 会随着 pending request rank 改变
- `virtual_geometry_prepare_cluster_raster_output_changes_when_pending_request_frontier_rank_changes`
  - 直接断言最终离屏帧输出与 raster coverage 会随着 pending request rank 改变

所以本轮 closure 不再只是“队列顺序变了”，而是 “同一条 rank 真值真的改变了 GPU submission 和最终 cluster raster 消费”。

## Why This Slice Exists

- 上一轮 `Frontier Priority And Active Request Lineage` 已经把 current frontier 顺序压进：
  - pending uploader queue
  - eviction ordering
- 但这仍然留有一个明显缺口：
  - runtime 知道哪个 pending page 更靠前
  - 真实 renderer submission / indirect args / raster output 却不知道

这会让 split-merge frontier 的优先级只影响“什么时候上传”，而不会继续影响“当前帧怎么消费这条 frontier truth”。

本轮补上的，就是这条 runtime-to-submission 的断层。

## Validation Summary

- `virtual_geometry_unified_indirect`
  - 证明 existing visibility-owned unified indirect contract 没有因为新增 frontier rank 而回归
  - 新增回归证明 pending request frontier rank 已进入 GPU indirect args 与 GPU-submitted segment buffer
- `virtual_geometry_prepare_render`
  - 证明 frontier rank 不只改变 readback，也会改变真实离屏 cluster-raster output
- `virtual_geometry_runtime`
  - 证明 runtime host 的 frontier-order / active-request-lineage / lineage-distance eviction 基线没有被新的 submission field 破坏
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明新的 prepare/submission/readback 合同仍然保持 crate 编译闭环

## Remaining Route

- 继续把 unified indirect authority 往更真实的 GPU-generated / visibility-owned submission 下沉，而不只是在 CPU 侧把 frontier rank 编进 segment buffer。
- 继续推进更深层的 residency-manager cascade，把 `frontier_rank`、slot recycling、page table truth、split-merge hysteresis 放到同一套 page hierarchy policy 下。
- 继续把 renderer-local cluster-raster contract 向更完整的 Nanite-like cluster streaming / raster consumption 演进。
