---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_args.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，不要中途确认，继续把 authority 压进更真实的 GPU-generated args source
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-visibility-owned-indirect-args-compaction.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-prepare-owned-indirect-order-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_fall_back_to_gpu_generated_indirect_args_tokens_when_debug_channels_are_missing -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Indirect-Args Token Fallback

## Goal

把 `Virtual Geometry` renderer last-state 对 draw-level submission token 的观测继续下沉到真实 GPU-generated indirect args buffer，不再在 dedicated `submission_buffer` 或 renderer-local cached token record 缺失时直接失去 submission truth。

## Delivered Slice

`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens(...)` 现在按以下优先级恢复 draw-level token：

1. renderer-local cached token record
2. GPU `submission_buffer`
3. 真实 indirect args buffer 的 `first_instance`

其中第 3 条直接消费已经提交给 GPU 的 indirect args record，而不是旁路 debug buffer。

## Why This Slice Matters

此前虽然 unified indirect / draw-ref / segment authority 已经下沉到了 shared indirect args source，但 renderer observability 仍然优先依赖平行的 token side-channel。

这会留下两个问题：

- debug buffer 或 cached token record 缺失时，draw-level submission order 会直接退化成“无 token truth”
- `first_instance` 已经编码了 visibility-owned submission token，但 last-state 并没有把它当成 authoritative recovery source

这条 slice 把最后一层 observability residue 继续压回 actual GPU-generated args source，使 unified indirect authority 更接近单一真值。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_fall_back_to_gpu_generated_indirect_args_tokens_when_debug_channels_are_missing -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_fall_back_to_gpu_generated_indirect_args_tokens_when_debug_channels_are_missing -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Virtual Geometry`: 继续把 unified indirect authority 从 draw-level observability 推进到更深的 cluster-raster / indirect execution ownership。
- `Virtual Geometry`: 继续补 split-merge frontier 与 page-table/completion 真值更深的 residency-manager cascade。
- `Hybrid GI`: 继续推进 scene-driven hierarchy / runtime-source / RT hybrid-lighting continuation。
