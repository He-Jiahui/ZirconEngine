---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/assign_execution_owned_indirect_args.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_execution_summary.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/assign_execution_owned_indirect_args.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_execution_summary.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
plan_sources:
  - user: 2026-04-20 continue M5 on the absorbed runtime graphics layout and push Virtual Geometry execution-owned indirect submission deeper before returning to Hybrid GI
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_draw_ref_indices_default_to_execution_args_without_dedicated_execution_buffer --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry Execution-Owned Indirect Submission

## Goal

继续把 `Virtual Geometry` 的 execution truth 从 “renderer host 额外拷一份 compact execution args 供 readback” 推进到 “真实 scene pass 自己就消费 compact execution-owned indirect source”。

## Problem

上一刀已经让 `execution args + authority/shared submission tokens` 成为默认 readback/fallback source，但真实 scene pass 仍然有一层 residual：

- `MeshDraw.indirect_args_buffer` 仍指向 shared visibility-owned superset buffer
- `MeshDraw.indirect_args_offset` 仍保留 shared slot order
- `execution_args_buffer` 只是 renderer 末端再 copy 出来的一份 compact mirror

这意味着 execution-owned source 还没有真正进入 draw submission ownership。本轮红测直接把这个裂口钉住了：

- `virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order`

在 deferred 下 shared visibility slot 顺序是 `[transparent, opaque]`，真实 scene-pass execution 顺序则是 `[opaque, transparent]`。改动前记录到的实际 draw offsets 还是 `[20, 0]`，说明 draw call 本身仍在吃 shared superset offsets。

## Delivered Slice

### 1. build 阶段显式保留 shared args truth

`build_mesh_draws(...)` 与 `build_compiled_scene_draws(...)` 现在会把 shared visibility-owned indirect args buffer 独立保留下来，而不是只让 `MeshDraw` 间接持有它。这样 renderer 后面就可以安全地重写 draw-time source，同时不丢掉 shared readback / fallback truth。

### 2. 新增 execution-owned indirect assignment

`assign_execution_owned_indirect_args.rs` 会在 `render_compiled_scene(...)` 里、真正进入 scene passes 之前执行：

- 先按真实 execution order 计算本帧 indirect draw 顺序
  - deferred: `opaque -> transparent`
  - non-deferred: 保持 mesh draw order
- 为 indirect draws 分配一份 compact `INDIRECT | COPY_SRC | COPY_DST` buffer
- 从 shared superset buffer 把每条真正会执行的 args 拷贝到 compact sequential slots
- 回写每条 `MeshDraw` 的 `indirect_args_buffer + indirect_args_offset`

因此真实 `draw_indexed_indirect(...)` 现在消费的是 compact execution-owned source，而不是 shared superset offsets。

### 3. shared last-state 与 actual execution source 解耦

`VirtualGeometryIndirectStats` 现在显式区分两类 source：

- `args_buffer`: 继续保留 shared visibility-owned indirect args buffer，供 readback / fallback 使用
- `execution_args_buffer`: 直接指向真实 scene pass 消费的 compact execution-owned buffer

这让 shared draw-ref / submission-token / authority 恢复链保持稳定，同时 execution-owned submission 又真正落到了实际 draw path 上。

### 4. execution offsets surface 补到 last-state

为了把这条 ownership 下沉验证固定下来，本轮又补了一条 execution debug surface：

- `last_virtual_geometry_execution_indirect_offsets`
- `read_last_virtual_geometry_execution_indirect_offsets()`

它记录真实 execution draws 在 render-time 使用的 indirect offsets。deferred 红测从原来的 `[20, 0]` 变成 `[0, 20]`，证明 actual draw path 已经切到 compact execution-owned buffer。

## Why This Matters

这条切片把当前 M5 VG 主链再往前推了一层：

- execution-owned args 不再只是 readback mirror
- shared visibility-owned args 继续作为统一 fallback truth 存在
- 实际 scene pass 的 indirect submission ownership 已经真正下沉到 compact execution source

换句话说，当前 renderer 已经不再停留在 “CPU 按 authoritative offset 排序提交，然后再事后拷一份 compact source” 的阶段，而是开始让 compact execution source 本身成为真实 draw-call source。

## Validation

- `cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_draw_ref_indices_default_to_execution_args_without_dedicated_execution_buffer --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

## Remaining Route

`Virtual Geometry` 这条主链已经不再薄在 “真实 pass 还吃 shared offset”。后面更值得继续推进的剩余任务是：

- 把 compact execution-owned source 从当前 renderer-side copy 继续压向更真实的 GPU-generated args compaction
- 继续把同一份 truth 压进 deeper cluster-raster submission ownership
- 再回到更深的 residency-manager cascade，把 page-table / completion / frontier truth 继续压进 recycle / hold / reconnect merge point

完成这几刀之后，再切回 `Hybrid GI` 的 scene-driven screen-probe hierarchy / RT hybrid-lighting continuation 会更稳。
