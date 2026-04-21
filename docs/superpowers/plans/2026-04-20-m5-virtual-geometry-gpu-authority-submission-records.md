---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
plan_sources:
  - user: 2026-04-20 continue M5 Virtual Geometry and keep pushing authority below CPU-side submission mapping
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-20-m5-virtual-geometry-explicit-draw-ref-index-authority.md
tests:
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_execution_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_unified_indirect --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_args_source_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_stats --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU Authority Submission Records

## Goal

继续把 `Virtual Geometry` 的 submission truth 从 renderer host 侧显式记录往更真实的 GPU-generated source 下沉。上一刀 `draw_ref_index` 已经进入 shared layout / mesh draw / execution stats / last-state records，但 deepest submission fallback 仍然要求至少保留 CPU records、execution records、indirect args / submission token、或者 shared draw-ref / segment buffer 才能恢复 submission records。

这一刀的目标是让 execution indices 可以直接配合一份 GPU compute pass 写出的 authority sidecar，在 CPU submission records、execution records、indirect args、submission token、draw-ref buffer、segment buffer 全部缺失时，仍然恢复真正的 submission records。

## What Changed

### GPU compute pass 新增 authority sidecar

`virtual_geometry_indirect_args.wgsl` 现在除了输出：

- `IndexedIndirectArgs`
- submission debug token

之外，还会额外输出每条 draw-ref 的 `SubmissionAuthorityRecord`：

- `draw_ref_index`
- `page_id`
- `submission_token`
- `entity_lo`
- `entity_hi`

这份 buffer 和 indirect args 一样由同一条 compute dispatch 生成，因此它和 shared layout 保持逐 draw-ref 一致，而不是 renderer 末端再补一份 host-only map。

### Runtime last-state 现在保留 GPU authority buffer

当前吸收后的 `zircon_runtime/src/graphics/**` 路径里：

- `SharedIndirectArgsBuffer`
- `BuiltMeshDraws`
- `VirtualGeometryIndirectStats`
- `SceneRenderer` last-state
- `render_frame_with_pipeline(...) -> store_last_runtime_outputs(...)`

都已经新增 `indirect_authority_buffer` 这条链路，因此这份 GPU-generated authority 可以跨过 render 完成后继续留在 renderer last-state，被后续 readback fallback 消费。

### Deep fallback 改为优先消费 execution indices + GPU authority buffer

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 现在的优先级变成：

1. CPU submission token records
2. execution records
3. execution indices + GPU authority buffer
4. submission token buffer / indirect args buffer / shared draw-ref + segment buffer 的兼容路径

也就是说，只要 actual execution subset 的 `draw_ref_index` 还在，readback 就不再需要 CPU records、indirect args token side-channel、shared draw-ref buffer、或者 shared segment buffer 才能知道：

- 实际提交的是哪条 draw
- 对应 entity/page 是谁
- submission token 是多少

## Regression Coverage

新增红绿回归：

- `virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only`

它会故意清掉：

- CPU submission token records
- CPU submission records
- execution records buffer
- indirect submission buffer
- indirect args buffer
- draw-ref buffer
- segment buffer

只保留：

- actual execution indices
- GPU-generated authority buffer

修补前这条链会直接掉回空集；修补后能继续恢复正确的 `(entity, page_id, submission_index, draw_ref_rank)`。

## Why This Matters

这一刀让 submission ownership 再往下压了一层：

- `draw_ref_index` 不只在 host detail 里显式化了
- GPU args compute pass 现在也自己写出一份可读回的 submission authority sidecar
- deepest submission fallback 不再要求 shared draw-ref / segment buffer 还活着

因此当前剩余最薄的 Virtual Geometry 主链已经继续收敛为：

- 让 actual execution subset 本身也尽量更多地来自 GPU-generated authority，而不是 renderer host 保存的 execution index / execution records
- 然后继续推进更深的 cluster-raster / indirect execution ownership，使 GPU-generated authority 成为更完整的 consume source，而不只是 readback fallback source
