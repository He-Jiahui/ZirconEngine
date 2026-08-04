---
handoff_kind: failure
status: open
created_at: 2026-08-03
summary_slug: pbr-viewer-one-shot-base-pipeline-timeout
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/shader/03
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app_tests.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_runtime/src/graphics/pipeline/async_compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
tests:
  - cargo test -p zircon_app --bin zircon_shader_pbr_viewer one_shot_base_pipeline --locked
  - cargo test -p zircon_runtime runtime_environment_only_pbr_base_queue --locked
---

# Shader03: PBR viewer one-shot Base pipeline wait is unbounded

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：M6 environment-only PBR performance forward repair and independent second review.
- 修复责任计划：`docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md`
- 交接原因：Shader03 owns the active `zircon_shader_pbr_viewer` source and tests. Shader06 must not edit its active owner without a transfer.

## 失败现象与复现证据

- Independent review found that `render_and_present` treats a one-shot screenshot or RenderDoc capture with `environment_only_base_pipeline_ready() == Ok(false)` as a retry: it schedules `WaitUntil` and returns.
- The retry interval is backoff-capped at 250 ms, but the total one-shot wait is unbounded. A queued or already-pending worker job that never completes can therefore keep a screenshot/capture process alive indefinitely.
- A full async budget is bounded backpressure, not a `GraphicsError`: it must keep the Base variant pending without reconstructing WGSL, then be admitted by a later nonblocking host retry after capacity is reclaimed.

## 最低共享层根因

The viewer has retry-delay state only. It has no deadline owned by the shared one-shot screenshot/RenderDoc gate, and therefore has no terminal transition for a non-completing Base pipeline.

## 架构修复验收

- Track one bounded deadline for the shared one-shot Base-pipeline gate; it must cover both screenshot and RenderDoc capture and report a clear terminal timeout error.
- Reset the deadline and retry state on readiness, scene-load failure, and a fresh load, so successful interactive rendering never inherits a stale timeout.
- Preserve the nonblocking event loop and the existing `SkipDraw` policy; do not restore synchronous PSO creation or continuous redraw polling.
- Add focused unit/source-contract coverage for timeout, readiness reset, failure reset, and the shared screenshot/RenderDoc gate.
- Rerun the managed Windows one-shot screenshot and RenderDoc capture gates after the focused test passes.

## 禁止临时方案

- Do not restore synchronous PSO creation, spin/retry in the event loop, or treat a bounded worker queue as a terminal compilation failure.

## 修复结果与回传

Open state: `待修复`; no validation pass is claimed. The candidate forward repair is implemented but remains open until its focused managed Windows tests, current-source screenshot metadata, and RenderDoc replay complete.

### 前向修复收据 (2026-08-03)

- Applied Viewer patch 76: one shared 45-second screenshot/RenderDoc Base-PSO deadline, reset on fresh load, load failure, and readiness.
- Applied Viewer patch 78: when the readiness query returns `false`, the host retries Base-PSO admission before scheduling its existing bounded backoff. Worker and shader-compilation failures remain terminal.
- Runtime now checks pending/capacity before shader-source assembly, so a full queue does not repeatedly assemble the reduced WGSL source. Regression coverage holds a 64-slot queue full across repeated admissions, releases one worker completion, verifies target re-admission, then verifies readiness.
- This handoff remains `open` until managed Windows focused tests, current-source screenshot metadata under `docs/tests/runtime/shader`, and RenderDoc replay complete. M6 remains `in_progress`; no accepted milestone status is claimed here.
