---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-29
summary_slug: gpu-pass-timer-backend-facade
origin_plan: docs/plans/zircon_plugins/09-export-publishing.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_plugins/09
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/gpu_pass_timer/mod.rs
  - zircon_runtime/src/rhi_wgpu/gpu_pass_timer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
tests:
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - cargo +1.94.1 test -p zircon_runtime --test plugins09_export_validate_report --bin zircon_export_validate --locked --jobs 1 --color never -- --nocapture --test-threads=1
---

# Render17: GPU pass timer backend facade

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 来源执行切片：Plugins09 compact validate-report current-source successor
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：Render17 added the GPU pass timer beneath `graphics::backend::render_backend`, but its scene-renderer consumers compile against the established crate-private `graphics::backend` owner. Plugins09 must not reach into or rewrite this renderer boundary.

## 失败现象与复现证据

Managed job `85dba9347e894ec39fde1ebd41e3afc4` / run
`48847c0d689b4f759cd1ae175441f4d3` naturally terminated with exit 101 while
running the Plugins09 exact command. Rust 1.94.1 reported five E0432 errors and
one E0425 because `GpuPassTimer`, `GpuPassTimestampScope`,
`GpuTimerFrameResult`, and `DEFAULT_GPU_TIMER_MAX_PASSES` existed in
`graphics::backend::render_backend` but were inaccessible through
`graphics::backend`. Six later E0282 diagnostics were type-inference cascades.

The run was diagnostic-only: its 9,812-path Runtime compile-input attestation changed
on four active Render17 consumer files between start and terminal. No Plugins09 test
executed.

## 最低共享层根因

`graphics::backend` is the crate-private boundary consumed by scene renderer code.
The new timer owner exported its types only one level up to the private
`render_backend` module, leaving the real boundary incomplete. The missing projection
must be completed at `graphics::backend`; consumers must not import a private child or
grow parallel aliases.

## 架构修复验收

- Project only the four timer types (`GpuPassTimer`, `GpuPassTimestampScope`, `GpuPassTiming`,
  `GpuTimerFrameResult`) and one constant (`DEFAULT_GPU_TIMER_MAX_PASSES`) required by current
  scene-renderer consumers through `graphics::backend`.
- Keep the implementation module private and do not widen any timer item beyond `pub(crate)`.
- Re-run a managed Rust 1.94.1 Runtime lib compile against a stable full-input attestation.
- Re-run the pending Plugins09 exact upward gate after the independent Text04 visibility failure is fixed by its owner.

## 禁止临时方案

- Do not make `render_backend` public or teach scene renderer code to import its private child path.
- Do not add duplicate type aliases, compatibility modules, fallback timers, or call-site type annotations that mask the missing owner projection.
- Do not classify the source-raced diagnostic run as acceptance evidence.

## 修复结果与回传

Open state: the timer implementation now has one lower-level owner at
`rhi_wgpu::gpu_pass_timer`; `graphics::backend` preserves the crate-private scene-renderer facade
through a direct re-export, and retained-UI presentation consumes the same timer without a second
query/readback state machine. Scene and retained-UI timer resources now default off and require an
explicit profiling option; UI device negotiation also omits timestamp features while that option is
off. Completed three-slot readbacks are inserted by frame generation before drain, and scene
rendering now keeps the frame-profiler generation distinct from its mesh-command cache generation.
Managed current-source compile evidence remains pending, so this handoff is not yet promoted to
fixed.

2026-08-10 forward repair: `FrameProfiler` now retains every GPU profile resolved during one
submission instead of allowing a pipeline-statistics result to overwrite an independently resolved
timer result. Matching timer/statistics results are merged before their profile is cloned; distinct
generations are each backfilled into their matching capture mailbox, while the flat latest-profile
diagnostic still selects the newest generation. Focused Rust formatting, diff checks, and a
simultaneous timer/statistics regression contract pass statically. This remains source progress only;
the managed current-source Cargo/WGPU timestamp and RenderDoc evidence above is still required.
