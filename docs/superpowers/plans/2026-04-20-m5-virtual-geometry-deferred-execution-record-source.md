---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
plan_sources:
  - user: 2026-04-20 继续把同一份 confirmed truth 压进更真实的 visibility-owned / GPU-generated args source 和 deeper cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Deferred Execution Record Source

## Goal

继续把 `Virtual Geometry` 的 submission truth 从 “shared indirect buffers + CPU records” 往更深的 actual execution source 下沉，尤其是把 `Deferred` 路径下真正的 scene-pass 执行顺序，以及 shared args/draw-ref/segment 缺失后的 execution truth，统一压到 renderer 自己可回读的 GPU source。

## Problem

此前这条链还有两处残余裂口：

- `indirect_execution_buffer` 虽然已经只保存 actual submitted subset，但它仍然在 `build_mesh_draws(...)` 阶段按 CPU build order 写出；到了 `Deferred` 路径，真正的 execution 顺序其实已经变成 `opaque -> transparent`，因此 last-state 仍可能把 unified-indirect build order 误认成最终 scene-pass submission order。
- 即使有 execution subset source，`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 仍然需要 shared `indirect args / draw_ref / segment` buffers 才能恢复 `(entity, page, token)`；这些 buffer 一旦缺失，deepest fallback 会直接掉回空集。进一步地，`read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 也仍然只依赖 dedicated index buffer，本身没有更深 fallback。

## Delivered Slice

### 1. actual execution order 已经改成 scene-pass authority

`virtual_geometry_indirect_stats(...)` 现在不再在 `build_mesh_draws(...)` 里提前固化 execution subset，而是会在 `render_compiled_scene(...)` 里基于真实执行序列计算：

- `Forward` 路径：沿 `mesh_draws` 当前顺序
- `Deferred` 路径：显式按 `opaque_mesh_draws -> transparent_mesh_draws`

因此 renderer last-state 现在记录的是 “真正执行的 scene-pass 顺序”，而不再只是 unified-indirect build order。

### 2. renderer 现在额外保存 execution-record GPU source

除了 `indirect_execution_buffer(draw_ref_index list)` 之外，当前又新增了一份更深的 `indirect_execution_records_buffer`，每条记录都会保存：

- `draw_ref_index`
- `entity`
- `page_id`
- `submission_index`
- `draw_ref_rank`

这份 buffer 会沿：

- `VirtualGeometryIndirectStats`
- `render_compiled_scene(...)`
- `render_frame_with_pipeline(...)`
- `store_last_runtime_outputs(...)`
- `SceneRenderer`

一路保留到 renderer last-state。

### 3. deepest fallback 已经开始直接认 execution-record truth

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 现在会在 CPU token records 缺失后，优先读取 `indirect_execution_records_buffer`。因此即使：

- CPU submission records 没了
- submission token buffer 没了
- shared indirect args 没了
- draw-ref buffer 没了
- segment buffer 没了

renderer 仍能直接回读出真实执行过的 `(entity, page_id, submission_index, draw_ref_rank)`。

### 4. execution-index fallback 也已经并到同一份 execution-record source

`read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 在 dedicated `indirect_execution_buffer` 缺失时，当前也会回退去解码 `indirect_execution_records_buffer` 中保存的 `draw_ref_index`。这使得 actual execution observability 不再依赖两条彼此割裂的 buffer truth。

## Why This Matters

这条 slice 让当前 M5 主链又往下收了一层：

- unified indirect / visibility order 不再只影响 build order，而开始真实影响 `Deferred` 场景下的最终 pass submission order
- actual execution subset 不再只是一份 “index-only debug side channel”，而是升级为包含 `draw_ref + entity/page + token` 的独立 GPU execution contract
- deepest last-state fallback 不再要求 shared indirect buffers 一直活着，execution truth 终于开始脱离 visibility superset buffers 独立存在

这让后续继续往更深的 GPU-driven cluster raster / residency cascade 推进时，renderer 已经有了一份更接近 “真实执行结果” 的统一 authoritative source。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers -- --nocapture`
- `cargo test -p zircon_graphics --offline virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`

## Validation Notes

- 本轮新增的 `virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone` 已通过。
- 后续尝试继续执行更宽的 `cargo test/check -p zircon_graphics --offline --locked` 时，工作区外部出现了新的 manifest/lockfile 漂移：根 `Cargo.toml`、`Cargo.lock` 与 `zircon_graphics/Cargo.toml` 正在被别的改动重写到 `zircon_runtime/crates/*` 路径，并临时造成 `zircon_runtime` crate 解析失败。这个错误不来自本次 Virtual Geometry execution-record slice，本 note 只记录它作为 wider validation blocker。

## Remaining Route

- 继续把同一份 execution truth 往更真实的 GPU-driven cluster-raster / indirect execution ownership 下沉，而不只停在 renderer last-state。
- 继续把已对齐的 execution/page-table/completion/frontier truth 往更深的 residency-manager cascade / split-merge frontier policy merge 点压缩，减少仍存在的 runtime-side多源解释。
