---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-20 M5 还没收口，继续把同一份 confirmed truth 压进更真实的 visibility-owned / GPU-generated args source 和 deeper cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu_submission_fallback_ignores_non_submitted_visibility_draw_refs_when_cpu_records_are_gone -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU Submission Subset Source

## Goal

继续把 `Virtual Geometry` 的 confirmed submission truth 从 renderer-local CPU bookkeeping 往更真实的 `visibility-owned / GPU-generated args source` 下沉，尤其是让 deepest last-state fallback 也能认出“哪些 draw ref 真正被提交过”，而不是把 shared indirect buffers 里的 visibility superset 当成 execution subset。

## Problem

此前 `Virtual Geometry` 已经有两层 submission truth：

- shared `segment_buffer / draw_ref_buffer / indirect args / submission token`
- renderer-local `MeshDraw` submission records / token records

但 deepest fallback 仍然留着一个 authority 裂口：

- 当 `last_virtual_geometry_mesh_draw_submission_records`
- 和 `last_virtual_geometry_mesh_draw_submission_token_records`
- 以及 dedicated `submission_buffer`

都缺失时，`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 会直接枚举 shared `draw_ref_buffer`。

这会把“visibility-owned 的全部 draw refs”误当成“真实提交过的 mesh draws”。在 `prepare` 保留 visibility truth、但只有 drawable subset 真正生成 `MeshDraw` 的场景里，fallback 会重新复活未提交的 entity/page。

## Delivered Slice

### 1. renderer 现在额外下沉一份 GPU-side actual-submission subset source

`build_mesh_draws(...)` 现在会基于最终 `ordered_pending_draws` 和 shared layout 的 `pending_draw_draw_ref_indices`，生成一份新的 `indirect_execution_buffer`。

这份 buffer 的语义是：

- 一项对应一条真实提交过的 indirect mesh draw
- 内容是该 draw 最终消费的 shared `draw_ref_index`
- 顺序与真实 `MeshDraw -> draw_indexed_indirect(...)` 执行顺序一致

因此 renderer 不再只能从 CPU-side submission records 记住“实际提交子集”，而是开始把这条 truth 压成一份 GPU-side execution-index source。

### 2. runtime-output / last-state 会一起保留这份 execution source

新 buffer 现在沿以下链路持久化：

- `VirtualGeometryIndirectStats`
- `SceneRendererCore::render_compiled_scene(...)`
- `render_frame_with_pipeline(...)`
- `scene_renderer_runtime_outputs::store_last_runtime_outputs(...)`
- `SceneRenderer.last_virtual_geometry_indirect_execution_buffer`

并新增 readback helper：

- `read_last_virtual_geometry_indirect_execution_draw_ref_indices()`

这意味着 last-state 现在不只保存 “shared buffers 的 superset truth”，也开始保存 “真实提交子集”的 GPU-side source。

### 3. deepest submission fallback 现在先认 execution subset，再映射 token/segment/entity/page

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 在 CPU submission records 缺失时，当前逻辑改成：

1. 先读 `indirect_execution_buffer` 得到实际提交过的 `draw_ref_index` 子集
2. 再用 shared `draw_ref_buffer + segment_buffer` 还原 `(entity, page_id)`
3. 最后优先读 `submission_buffer`，缺失时再退回 indirect args `first_instance`

只有 execution buffer 本身也不存在时，才会退回旧的 “枚举全部 draw refs” 最深 fallback。

因此 deepest fallback 终于开始认 “actual submission subset”，而不是继续把 visibility-owned superset 伪装成 execution truth。

## Why This Matters

这条修补把 M5 当前剩余主链又往下推了一层：

- shared indirect buffers 继续保留 visibility-owned superset
- actual submission subset 现在也有了显式 GPU-side source
- last-state fallback 不再必须依赖 CPU-side records 才能分辨“真正执行过的 draw”

这让 `Virtual Geometry` 的 unified indirect authority 不只停在 shared args/readback 级别，而是开始把 “真实执行过的子集” 也收束到 renderer 自己可回读的同一条 submission contract 里。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu_submission_fallback_ignores_non_submitted_visibility_draw_refs_when_cpu_records_are_gone -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 如果继续把这条 M5 链再往下压，下一步就不该再只是 “CPU mesh draw loop + per-draw indirect offset”，而会进入更真实的 GPU-driven indirect compaction / execution ownership。
- `Virtual Geometry` 的更深层收口点将转向真正的 GPU-generated args source、cluster-raster submission ownership，以及更彻底的 residency-manager / page-table / completion / frontier 共真值级联。
