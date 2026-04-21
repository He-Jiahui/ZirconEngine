---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
plan_sources:
  - user: 2026-04-20 continue M5 on the absorbed runtime graphics layout and keep pushing Virtual Geometry execution ownership deeper before Hybrid GI
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry Execution-Owned Authority Buffer

## Goal

继续把 `Virtual Geometry` 的 actual execution truth 从 “execution-owned compact args + shared authority remap” 推到 “execution-owned compact authority records 也一并下沉”，让 execution record / segment / submission 恢复进一步摆脱 shared visibility-owned authority 侧链。

## Problem

上一刀已经让真实 scene pass 直接消费 compact execution-owned indirect args，但 readback/fallback 仍有一层 residual：

- `read_last_virtual_geometry_indirect_execution_records()`
- `read_last_virtual_geometry_indirect_execution_segments_with_entities()`
- `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()`

这些恢复路径虽然可以先从 compact execution args 得到 actual execution order，但还需要再回头去 shared `authority_buffer` 或 shared `submission/draw_ref/segment` buffers 做 remap。

这意味着 execution-owned source 还不够自洽。一旦 shared visibility-owned authority 和 remap buffers 同时消失，actual deferred submission truth 仍会掉回空集。

## Delivered Slice

### 1. renderer last-state 新增 compact execution authority buffer

`virtual_geometry_indirect_stats(...)` 现在会基于真实 `indirect_execution_draws` 和 shared GPU authority buffer，按 actual execution order 拷出一份 compact `execution_authority_buffer`。

这份 buffer 的每条记录都与真实 execution draw 一一对应，顺序和 scene pass 提交顺序一致，而不再保持 shared visibility-owned layout。

### 2. compact execution authority 会沿 runtime-output 保存

新 buffer 现在沿以下链路持久化：

- `SceneRendererCore::render_compiled_scene(...)`
- `render_frame_with_pipeline(...)`
- `store_last_runtime_outputs(...)`
- `SceneRenderer.last_virtual_geometry_indirect_execution_authority_buffer`

因此 actual execution subset 现在除了 compact args 之外，还多了一份 compact authority truth。

### 3. execution draw-ref / record / segment readback 先吃 compact execution authority

以下 readback helper 现在会优先消费 `last_virtual_geometry_indirect_execution_authority_buffer`：

- `read_last_virtual_geometry_indirect_execution_draw_ref_indices()`
- `read_last_virtual_geometry_indirect_execution_records()`
- `read_last_virtual_geometry_indirect_execution_segments_with_entities()`

这意味着 actual execution subset 的 `draw_ref_index / entity / page / segment lineage / submission token fields` 已经可以直接从 execution-owned compact source 恢复，而不是必须回头查 shared visibility-owned authority buffer。

### 4. deeper submission fallback 自动继承这条 compact authority source

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 本来就先依赖 execution records。由于 execution records 现在可直接从 compact execution authority 读取，因此 submission fallback 也自动获得了更深层的 self-contained source：

- CPU submission records/token records 缺失
- shared authority/submission/args/draw-ref/segment buffers 缺失

在这种情况下，只要 execution-owned compact authority 还在，actual deferred submission truth 仍然有机会继续恢复。

## Why This Matters

这条切片继续把 M5 Virtual Geometry 主链往下压了一层：

- execution-owned args 已经进入真实 draw-call ownership
- execution-owned authority 现在也开始进入 actual readback ownership
- shared visibility-owned authority 不再是 execution record/segment/submission 恢复的唯一真值来源

换句话说，当前 renderer 已经不只是在提交层面拥有 compact execution source，连 execution metadata 也开始脱离 shared superset remap。

## Validation

- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

## Remaining Route

这条 execution-owned authority slice 之后，更自然的下一条主链继续收束为：

- 把 compact execution-owned source 继续压向更真实的 GPU-generated args compaction，而不是 renderer-side copy 终点
- 把同一份 truth 继续压进 deeper cluster-raster submission ownership
- 再切回更深的 residency-manager cascade / page-table / completion / frontier merge-point 收敛

在这些切片稳定之后，再回到 `Hybrid GI` 的 scene-driven screen-probe hierarchy / RT hybrid-lighting continuation 会更稳。
