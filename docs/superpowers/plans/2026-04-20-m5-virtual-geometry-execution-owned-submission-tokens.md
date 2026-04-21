---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_submission_tokens.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_submission_tokens.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
plan_sources:
  - user: 2026-04-20 continue the absorbed-runtime M5 Virtual Geometry chain and remove the remaining shared submission-token dependency before returning to Hybrid GI
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_submission_tokens_and_shared_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_args_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry Execution-Owned Submission Tokens

## Goal

继续把 `Virtual Geometry` 的 actual execution truth 从 “execution args + shared submission token remap” 往下压一层，让 execution-order token source 自己进入 renderer last-state，而不必在 readback 时重新扫描 shared submission buffer 或 shared indirect args。

## Problem

上一刀之后，actual scene pass 已经真正消费 compact execution-owned indirect args，execution authority 也有了 compact execution-owned authority buffer；但 token truth 仍然是薄点：

- `read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 仍然需要从 `execution_args_buffer.first_instance` 读 token
- 如果 execution authority 缺失，它还会继续回退到 shared `submission_buffer` 或 shared indirect args
- `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 因而仍可能在 execution args 缺失时掉回 shared authoritative order

这在 deferred 路径上尤其明显。新红测 `virtual_geometry_submission_records_survive_with_execution_submission_tokens_and_shared_authority_only` 证明了这一点：当 execution args 与 shared token/args 全部丢失时，submission records 会从真实 `[opaque, transparent]` execution order 退回 shared visibility-owned authoritative order。

## Delivered Slice

### 1. 新增 compact execution-owned submission token buffer

`virtual_geometry_indirect_stats(...)` 现在会基于真实 `indirect_execution_draws` 和 shared GPU submission buffer，按 actual execution order 拷出一份 compact `execution_submission_buffer`。

这份 buffer 与 compact execution args / compact execution authority 保持同一 execution-order indexing，而不再继承 shared visibility-owned draw-ref indexing。

### 2. execution token source 挂入 renderer last-state

新 buffer 现在沿以下链路持久化：

- `SceneRendererCore::render_compiled_scene(...)`
- `render_frame_with_pipeline(...)`
- `store_last_runtime_outputs(...)`
- `SceneRenderer.last_virtual_geometry_indirect_execution_submission_buffer`

因此 renderer last-state 现在同时持有三份 execution-owned truth：

- compact execution args
- compact execution submission tokens
- compact execution authority records

### 3. execution draw-ref readback 优先消费 compact execution tokens

`read_execution_submission_tokens(...)` 现在会优先读取 `last_virtual_geometry_indirect_execution_submission_buffer`；只有这份 compact token source 缺失时，才会回退到 `execution_args_buffer.first_instance`。

于是 `read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 在 execution authority 缺失但 shared authority 仍然存在时，可以直接走：

- execution-owned compact submission tokens
- shared authority records

而不再依赖 shared submission buffer 或 shared indirect args。

### 4. submission records 在 execution-args 缺失时保持 actual deferred order

因为 `read_last_virtual_geometry_indirect_execution_records()` 建立在 execution draw-ref indices 之上，`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 也自动继承了这条更深的 compact token source。

结果是：

- CPU submission token records 缺失
- CPU submission records 缺失
- compact execution authority 缺失
- compact execution args 缺失
- shared submission buffer 缺失
- shared indirect args 缺失

只要 shared authority 还在，actual deferred submission order 仍能继续恢复，不会再掉回 shared visibility-owned authoritative order。

## Why This Matters

这条切片继续减少了 Virtual Geometry 这条 M5 主链上的 shared-side remap 残留：

- compact execution args 负责真实 draw-call source
- compact execution authority 负责 metadata readback
- compact execution submission tokens 现在负责 execution-order token truth

换句话说，execution-owned authority 已经不再只停在 “args + authority”，而是开始把 submission token 这份 truth 也一起从 shared visibility-owned source 里拆出来。

## Validation

- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_submission_tokens_and_shared_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_args_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

## Remaining Route

`Virtual Geometry` 这条 execution-ownership 主链现在的剩余薄点进一步收窄为：

- compact execution subset 仍是 renderer-side copy，不是更真实的 GPU-generated args compaction authority
- cluster-raster submission ownership 还没有完全被同一份 execution truth 统一主导
- residency-manager cascade / page-table / completion / frontier merge-point 还需要继续沿这份 confirmed truth 下沉

因此下一条最自然的 M5 主链仍然是：

- 把 compact execution-owned source 继续压向更真实的 GPU-generated args compaction / deeper cluster-raster submission ownership
- 然后再切回更深的 residency-manager cascade / page-table / completion / frontier merge-point 收敛
- Hybrid GI 继续保持在这条 Virtual Geometry execution-owned source 再向前推进一刀之后再回切
