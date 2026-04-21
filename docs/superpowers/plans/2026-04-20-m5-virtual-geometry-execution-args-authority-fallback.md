---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
plan_sources:
  - user: 2026-04-20 continue M5 Virtual Geometry by pushing actual execution truth from host-held execution indices into a deeper GPU-generated args source
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_draw_ref_indices_default_to_execution_args_without_dedicated_execution_buffer --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_default_to_execution_args_and_authority_without_dedicated_execution_records_buffer --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry Execution Args Authority Fallback

## Goal

继续把 `Virtual Geometry` 的 actual execution subset truth 从 renderer host-held mirrors 往更深的 GPU source 下沉，尤其是把 “真实执行过哪些 draw / 顺序是什么” 从 `execution_indices` 这条 sidecar 再压到实际 GPU-generated indirect args 自身。

## Problem

上一轮已经完成了两条关键 closure：

- `execution_records + execution_segments` 可以只靠 `execution_indices + GPU authority buffer` 恢复
- `submission_records` 也可以在 CPU records 消失后依赖 `execution_indices + authority` 恢复

但这里仍然留着一条明显的 host residue：

- `last_virtual_geometry_indirect_execution_buffer` 只是 renderer host 重新序列化出来的 `draw_ref_index` 列表
- 一旦这份 execution-index buffer 和 host-built execution-record buffer 都被移除，deepest fallback 又会掉回空集
- shared indirect args / draw-ref / segment buffers 仍然只是 visibility superset，不能代表 actual scene-pass execution subset

这意味着 actual execution order 还没有真正压进更深的 GPU-generated args source。

## Delivered Slice

### 1. renderer 现在显式保存 compact actual-execution args buffer

`virtual_geometry_indirect_stats(...)` 新增一条 `execution_args_buffer` 路径：

- 它只针对真实 `execution_draws` 中的 indirect draws 建 buffer
- 每一项都从 shared indirect args buffer 拷贝对应 draw 的 `IndexedIndirectArgs`
- 顺序与真实 scene-pass execution order 一致，而不是 visibility superset order

这条 buffer 不是再存一份 host-built draw-ref mirror，而是保存“真实执行子集的 GPU-generated args 结果”。

### 2. runtime last-state 会持续保留这份 execution args source

新 buffer 沿当前吸收后的 `zircon_runtime/src/graphics/**` 链路挂进 `SceneRenderer` last-state：

- `render_compiled_scene(...)`
- `render_frame_with_pipeline(...)`
- `store_last_runtime_outputs(...)`
- `SceneRenderer.last_virtual_geometry_indirect_execution_args_buffer`

因此 renderer last-state 现在除了 shared visibility-owned indirect buffers、execution indices、execution records 之外，还保留了更深一层的 actual-execution GPU args source。

### 3. execution draw-ref recovery 现在优先吃 execution args + authority

`read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 的 fallback 顺序现在变成：

1. dedicated `execution_buffer`
2. compact `execution_args_buffer.first_instance` + GPU `authority_buffer`
3. host-built `execution_records_buffer`
4. 空结果

其中第 2 步会读取 compact execution args 里的 `first_instance submission token`，再用 authority records 把 token 映射回 `draw_ref_index`。这让 actual execution subset membership/order 即使失去 dedicated execution-index buffer，也不再必须重新退回 host-built execution records。

### 4. deeper submission fallback 自动继承这条新 source

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 原本已经会通过 execution draw-ref indices 继续往下恢复 submission truth。由于 execution draw-ref indices 现在能从 `execution args + authority` 恢复，因此 submission records 也自动获得了这条更深 fallback：

- CPU submission records / token records 缺失
- dedicated execution indices 缺失
- host-built execution records 缺失
- shared indirect superset buffers 缺失

在这种情况下，只要 compact actual-execution args buffer 和 authority buffer 还在，submission truth 仍然能恢复。

### 5. authority sidecar 缺失时也能继续依赖 shared GPU submission tokens

继续往下压一层后，`read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 又新增了第二条 GPU-only token fallback：

1. `execution args` 提供 actual execution order 的 compact `first_instance submission token`
2. shared `submission_buffer` 或 shared indirect args 提供 `submission token -> draw_ref_index` 对照表
3. `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 再用 shared `draw_ref_buffer + segment_buffer` 恢复 `(entity, page_id)`

因此当 GPU authority sidecar 本身被移除时，actual deferred submission order 也不会再退回 shared draw-ref buffer 的 visibility order。只要 shared GPU submission-token truth 还在，submission record fallback 就继续按真实 execution order 恢复。

### 6. dedicated execution-index / execution-record sidecars 已默认退场

继续收口后，renderer 不再默认发布两条 host-built sidecar：

- dedicated `execution_buffer(draw_ref_index[])`
- dedicated `execution_records_buffer([u32; 14][])`

当前默认链路已经变成：

- `read_last_virtual_geometry_indirect_execution_draw_ref_indices()`
  - `execution args + authority`
  - `execution args + shared submission tokens`
- `read_last_virtual_geometry_indirect_execution_records()`
  - `execution draw_ref_indices + authority`
- `read_last_virtual_geometry_indirect_execution_segments_with_entities()`
  - `execution draw_ref_indices + authority`

因此 actual execution subset/order 的默认恢复不再依赖额外 host-built u32/record mirrors，而是直接依赖 compact execution args 与现有 GPU-side authority/token truth。

## Why This Matters

这条切片把当前 M5 主链再往前推了一层：

- shared indirect buffers 仍然是 visibility-owned superset truth
- execution indices 仍保留为更直接的 debug/readback source
- 但 deepest fallback 已经不再只能停在 host-built execution sidecar

换句话说，actual execution subset 的 authoritative recovery 已经开始依赖真正执行后的 GPU args token，而不是继续把 execution ownership 锁死在 CPU 序列化列表上。

## Validation

- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_draw_ref_indices_default_to_execution_args_without_dedicated_execution_buffer --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_default_to_execution_args_and_authority_without_dedicated_execution_records_buffer --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

## Remaining Route

- 下一刀仍然是 `Virtual Geometry` 主链：继续减少 `execution_indices` 这条 dedicated host sidecar 的必要性，把同一份 truth 压进更真实的 GPU compaction / cluster-raster submission ownership。
- 这条 dedicated host sidecar 已经默认退场；下一刀更自然地收束为继续减少 renderer 逐 draw copy / CPU submission ordering 的参与度，把 compact execution source 本身压向更真实的 GPU compaction / cluster-raster submission ownership。
- 更深的收口点仍是：
  - visibility-owned unified indirect / GPU-generated args compaction
  - deeper cluster-raster execution ownership
  - residency-manager cascade / page-table / completion / frontier truth 继续统一
