---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_args_source_authority.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
plan_sources:
  - user: 2026-04-20 continue M5 on the absorbed runtime graphics layout and keep pushing Virtual Geometry authority below CPU-side submission reconstruction
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-20-m5-virtual-geometry-execution-stats-surface.md
tests:
  - cargo test -p zircon_runtime --locked --offline execution_draw_ref_index_prefers_explicit_submission_detail_source --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline shared_indirect_args_layout_emits_authoritative_pending_draw_submission_plan --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_stats --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_execution_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_args_source_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_unified_indirect --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry Explicit Draw-Ref Index Authority

## Goal

把 `Virtual Geometry` 当前仍然残留在 renderer 侧的 `indirect_args_offset -> draw_ref_index` 推导，再往下收一层。上一刀 shared layout 已经成为 authoritative pending-draw submission plan，但 execution stats、last-state submission readback 仍然会把 `indirect_args_offset` 当成 draw-ref source 的代理值。这一刀的目标是把同一份 draw-ref truth 显式带进 shared layout、mesh draw submission detail 和 last-state。

## What Changed

### Shared Layout 现在显式产出 draw-ref index

`build_shared_indirect_args_layout(...)` 原本已经在内部得到 `pending_draw_draw_ref_indices`，但这份 truth 只停留在 layout 内部。现在：

- `SharedIndirectArgsLayout` / `SharedIndirectArgsBuffer` 都会显式保留 `pending_draw_draw_ref_indices`
- `VirtualGeometrySubmissionDetail` 新增 `draw_ref_index`
- `pending_draw_submission_details` 会直接把当前 authoritative draw-ref index 写进去，而不是只暴露 `(submission_index, draw_ref_rank)`

这样 renderer 后续阶段不需要再把同一个 draw-ref 身份从 `indirect_args_offset / stride` 反推出去。

### Mesh draw fallback 改为优先吃 explicit draw-ref truth

`build_mesh_draws(...)` 仍然保留兼容 fallback，但 fallback 来源发生了变化：

- 优先使用 shared layout 带下来的 `pending_draw_draw_ref_indices`
- 只有 explicit authority 缺失时，才退回 `indirect_args_offset / stride`

也就是说，当前 renderer 末端的 `VirtualGeometrySubmissionDetail` 已经不再默认依赖 byte-offset residue 才能知道自己对应哪条 draw-ref。

### Execution stats / last-state submission readback 不再把 offset 当成主真值

`virtual_geometry_indirect_stats(...)` 新增 `execution_draw_ref_index(...)` helper：

- 有显式 `submission_detail.draw_ref_index` 时直接使用它
- 只有缺失显式 detail 时才回退到 `indirect_args_offset / stride`

同时 `draw_submission_records` 的 last-state 存储也从：

- `(entity, page_id, indirect_args_offset, original_index)`

改成：

- `(entity, page_id, draw_ref_index, original_index)`

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 读取 last-state submission records 时，也就不再需要先从 offset 除 stride 才能找到 draw-ref token。

## Why This Matters

这一步不是为了改一层数据结构，而是为了继续消除 renderer 侧的 CPU residue：

- shared indirect layout 现在不只 author submission order，也 author 每条 draw 对应的 explicit draw-ref identity
- mesh draw、execution stats、last-state readback 都开始消费同一份 draw-ref authority
- `indirect_args_offset` 退回兼容路径，不再是 execution ownership 的默认真值

这让下一条更自然的 M5 Virtual Geometry 主链进一步收窄成：

- 把这份 explicit draw-ref / submission truth 再继续压进更真实的 GPU-generated args source / compaction ownership
- 或者直接继续推进更深的 cluster-raster / indirect execution consume side，不再让 renderer host 保存最后一份关键映射关系

## Regression Coverage

本轮新增红绿回归：

- `execution_draw_ref_index_prefers_explicit_submission_detail_source`
  - 证明 execution stats 现在优先消费显式 draw-ref authority，而不是继续从 `indirect_args_offset` 推导

同时复跑了受影响的主链回归：

- shared layout authoritative submission plan
- execution stats surface
- submission execution order
- args source authority
- unified indirect reconstruction / repeated primitive compaction

这些回归一起证明：

- shared layout 没有丢掉 draw-ref / submission truth
- execution stats 和 last-state readback 仍然能恢复真实 submission records
- repeated primitive compaction、fallback cluster slices、deferred execution source 这些已有 M5 行为没有被这次 authority 下沉打回去

## Remaining Follow-On

当前更自然的下一条主链仍然是两项：

- `Virtual Geometry`
  - 把这份 explicit draw-ref truth 再压进真正的 GPU-generated args source / compaction ownership，继续减少 renderer host 保留的 submission mapping
  - 然后继续往 deeper cluster-raster / indirect execution ownership 推进
- `Hybrid GI`
  - 继续把 scene-driven screen-probe hierarchy gather / runtime source / RT hybrid-lighting continuation 从 encode-side 向 runtime/GPU source 收口

如果继续优先 M5 Virtual Geometry，这一刀之后最薄的残余点已经不再是 “offset 推导 draw-ref”，而是 “真正的 GPU-generated args / compaction source 仍然还没有成为唯一 authoritative consume source”。
