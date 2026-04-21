---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_execution_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_execution_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
plan_sources:
  - user: 2026-04-20 continue M5 Virtual Geometry on the current absorbed runtime graphics layout and make actual execution truth feed a real runtime-facing consumer
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
tests:
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_args_source_authority.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_unified_indirect.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
doc_type: milestone-detail
---

# M5 Virtual Geometry Execution Stats Surface

## Goal

把已经进入 `SceneRenderer` last-state 的 Virtual Geometry actual-execution truth 继续推进到真正的 runtime-facing surface，而不是只留在 test-only GPU readback/helper 层。上一刀已经让 execution subset 的 `(draw_ref_index, entity, page_id, submission_index, draw_ref_rank)` 能从真实 scene-pass 执行次序回读；这一刀的目标是让 render-framework façade 也能直接看到这份 truth 的压缩结果。

## What Changed

`virtual_geometry_indirect_stats(...)` 现在除了原来的：

- indirect draw count
- shared indirect buffer count
- prepare-owned segment count
- submission-order / token records

还会额外归纳 actual execution subset 的六个标量：

- `execution_segment_count`
- `execution_page_count`
- `execution_resident_segment_count`
- `execution_pending_segment_count`
- `execution_missing_segment_count`
- `execution_repeated_draw_count`

这些标量全部来自真正参与本帧 indirect execution 的 `MeshDraw` 子集，而不是来自 prepare superset 或 shared draw-ref/segment buffer 的静态长度。

## Data Path

这条 summary 现在沿以下路径贯通：

1. `virtual_geometry_indirect_stats(...)`
   - 对真实执行的 indirect draws 去重生成 execution-segment key。
   - 分别统计 unique execution segments、unique execution pages、按 cluster state 划分的 resident/pending/missing segment 数量。
   - 额外记录 `execution_repeated_draw_count = executed_draws - unique_execution_segments`，直接暴露 repeated primitive compaction 是否真实进入执行面。

2. `SceneRenderer` last-state
   - `render_compiled_scene(...) -> render_frame_with_pipeline(...) -> store_last_runtime_outputs(...)` 会把这些 execution summary 标量随当前帧一起存进 renderer last-state。
   - `read_execution_summary.rs` 提供内部读取接口，避免 runtime/stats 再去依赖 GPU readback。

3. `RenderFramework::query_stats()`
   - `update_virtual_geometry_stats(...)` 现在会把 execution summary 写入 `RenderStats`。
   - 上层拿到的是稳定 façade 字段，不需要知道 renderer 私有 buffer 或 GPU readback 细节。

4. `build_mesh_draws(...)` 不再重建排序键
   - shared indirect layout 现在会直接产出 `pending_draw_submission_plan`，其中已经带好 authoritative `pending_draw_index + indirect_args_offset + submission_detail`。
   - `build_mesh_draws(...)` 现在直接消费这份 plan 来生成 ordered `MeshDraw`，不再在 renderer 末端用 `submission_order + token + offset` 再做一次 CPU sort reconstruction。
   - 这让 unified-indirect / cluster-raster submission ownership 又往 shared-layout authority 下沉了一层。

## Public Surface

`RenderStats` 新增：

- `last_virtual_geometry_execution_segment_count`
- `last_virtual_geometry_execution_page_count`
- `last_virtual_geometry_execution_resident_segment_count`
- `last_virtual_geometry_execution_pending_segment_count`
- `last_virtual_geometry_execution_missing_segment_count`
- `last_virtual_geometry_execution_repeated_draw_count`

这组字段和原有的：

- `last_virtual_geometry_indirect_draw_count`
- `last_virtual_geometry_indirect_args_count`
- `last_virtual_geometry_indirect_segment_count`

共同组成一条更完整的 façade 观测面：

- `indirect_*` 仍然反映 prepare-owned / shared-indirect contract 的总体规模。
- `execution_*` 明确反映真实执行子集的 unique segment/page/state/compaction 结果。

这样 runtime/editor/script 现在可以区分：

- “准备阶段有多少 VG segment”
- “本帧真正执行了多少 unique segment/page”
- “这些执行 segment 里有多少仍是 resident/pending/missing fallback”
- “repeated primitive compaction 是否已经真实进入 GPU execution”

## Regression Coverage

新增 `zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs`：

- `renderer_execution_stats_follow_actual_virtual_geometry_cluster_states`
  - 直接验证 renderer last-state 的 execution summary 会跟随真实 resident/pending execution subset，而不是只跟随 prepare superset。

- `render_framework_stats_expose_actual_virtual_geometry_execution_compaction`
  - 用 multi-primitive model 验证 `RenderStats` 能同时看见：
    - 两条真实 indirect draws
    - 一条 unique execution segment
    - 一页 unique execution page
    - 一条 repeated draw compaction delta

shared-layout 侧还新增：

- `shared_indirect_args_layout_emits_authoritative_pending_draw_submission_plan`
  - 验证 shared indirect layout 已经直接给出 authoritative pending-draw submission plan，而不再要求 `build_mesh_draws(...)` 用 CPU-side sort keys 重新拼一份提交顺序。

## Validation

本轮围绕当前 slice 已运行：

- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_stats --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_execution_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_args_source_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_unified_indirect --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline render_framework_bridge --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline shared_indirect_args_layout_emits_authoritative_pending_draw_submission_plan --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

## Remaining M5 Follow-On

这条 slice 收口后，Virtual Geometry 下一条更自然的主链回到：

- 把同一份 execution truth 继续压进更真实的 visibility-owned / GPU-generated args source，减少 CPU-side submission reconstruction 残留。
- 再继续向更深的 cluster-raster / indirect execution ownership 推进，让统一 indirect authority 不只可观测，而且直接成为 execution-side consume source。
- 完成之后再回到更深的 residency-manager cascade / split-merge frontier policy 收敛。
