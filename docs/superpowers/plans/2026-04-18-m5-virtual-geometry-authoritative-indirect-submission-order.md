---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
plan_sources:
  - user: 2026-04-18 把 confirmed submission_slot / page-table / completion 真值继续压进更深的 residency-manager cascade
  - user: 2026-04-18 把同一套 confirmed slot/page truth 继续下沉到更真实的 GPU-driven cluster raster / indirect execution
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-fallback-slot-submission-authority-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-table-confirmed-completion-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Authoritative Indirect Submission Order

## Goal

把已经确认下来的 `submission_slot / page-table / completion` 真值继续推进到更真实的 `GPU-driven cluster raster / indirect execution`，让 renderer 最终提交的 indirect draw 顺序也开始服从同一份 authority，而不再只是在 segment readback 或 uploader fallback 选槽里保持正确。

## Non-Goal

- 本轮不实现真正的 visibility-owned GPU compaction buffer。
- 本轮不实现 Nanite-like cluster rasterizer、meshlet execution 或 full GPU multi-draw compaction。
- 本轮不再新增一套独立 residency policy；residency-manager 继续沿 `page_table_entries` 驱动的现有主链收口。

## Delivered Slice

### 1. Draw-ref buffer 改为按 authoritative submission key 排序

`build_shared_indirect_args_buffer(...)` 现在不再把 `draw_ref_buffer` 固定写成 CPU `pending_draws` 的插入顺序。

它会先把每条 pending draw 的 indirect draw ref 收束成 `OrderedDrawRef`，再按以下统一 key 排序：

- `submission_slot`
- `frontier_rank`
- `entity`
- `cluster_start_ordinal`
- `page_id`
- `cluster_span_count / cluster_total_count`
- `lod_level`
- `lineage_depth`
- `state`
- `mesh_index_count`
- 原始 `pending_draw_index` 只作为最终稳定打平项

这样 `draw_ref_buffer` 本身已经变成 authoritative submission order 的真实镜像，而不是只在固定 CPU draw 顺序上保存一个 segment remap。

### 2. Shared indirect args offset 改为跟随排序后的真实提交顺序

`SharedIndirectArgsBuffer` 现在额外缓存 `indirect_args_offsets`。

这些 offset 不是再用 `index * stride` 从 CPU `pending_draws` 顺序直接推出来，而是按排序后的 `OrderedDrawRef` 真实位置回填到对应 pending draw。

随后 `build.rs` 在创建 `MeshDraw` 时会消费这份 authority-owned offset，因此真正的 `draw_indexed_indirect(...)` 提交顺序已经跟随同一条 slot/page/frontier truth，而不会继续停留在旧的 CPU pending-draw 顺序。

### 3. Segment / draw-ref / indirect args 三层现在共享同一条 submission truth

在这轮之后，以下三层已经收敛到同一套排序 authority：

- `segment_buffer`
- `draw_ref_buffer`
- `MeshDraw.indirect_args_offset -> indirect args buffer`

这意味着 fallback recycle-slot authority 不再只改变：

- uploader 选槽
- segment readback 顺序
- shader 内部对 segment 的消费

它现在还会真实改变最终 indirect draw 在 GPU 上的编码顺序与执行顺序。

### 4. Residency cascade 这一侧没有再发现新的 raw completion 泄漏点

本轮继续复核了 `VirtualGeometryRuntimeState` 与 `update_virtual_geometry_runtime(...)` 当前主链：

- final `page_table_entries`
- confirmed completion/replacement reconstruction
- pending clear
- next-frame prepare request / available-slot / resident-page projection

当前路径里没有新的 raw `completed_page_assignments` / raw replacement side-channel 继续绕开 final page-table truth。

也就是说，`residency-manager cascade` 在当前 host path 上已经由上一轮 `page-table-confirmed completion` 收口；本轮的新增价值主要落在更深的 indirect execution ownership。

## Why This Slice Exists

上一轮已经让：

- confirmed completion 只信 final `page_table_entries`
- replacement 只信 confirmed slot 的 previous owner

但 renderer 最终仍然可能在更底层保留另一条旧 truth：

- `segment_buffer` 顺序是对的
- `draw_ref_buffer` 和 `indirect args offset` 却仍然跟着 CPU pending-draw 插入顺序走

这样会导致 authoritative `submission_slot` 虽然已经进入 readback / stats / segment truth，却还没有真正落到最后的 indirect execution order。

本轮补上的就是这条最后的 submission-order 断层。

## Validation Summary

- `virtual_geometry_submission_authority`
  - 证明 fallback slot authority 现在会同时改变 segment order、draw-ref order 与 indirect args order
- `virtual_geometry_unified_indirect`
  - 证明新的 offset/order 规则没有破坏 prepare-owned unified indirect 与 GPU submission readback 主链
- `virtual_geometry_prepare_render`
  - 证明更深的 indirect ordering 没有把已有 cluster-raster consumption 回归打回旧语义
- `virtual_geometry_runtime`
  - 证明 residency-manager 侧继续保持 final page-table / confirmed completion 主链，没有被新的 indirect execution ordering 破坏
- `virtual_geometry_gpu`
  - 证明 uploader/page-table/replacement readback 仍与 runtime/prepare/indirect 主链兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 shared indirect args buffer 新 contract 没有破坏 crate compile closure

## Remaining Route

- 把当前 renderer-side authoritative ordering 继续下沉到真正的 visibility-owned unified indirect args buffer，而不再由 mesh build step 做最后排序
- 继续推进更深的 GPU-driven indirect compaction / cluster raster consumption
- 在 residency-manager 侧继续沿 split-merge frontier policy 与 page-table truth 做更完整的 cascade，而不是只停在当前 host/runtime 闭环
