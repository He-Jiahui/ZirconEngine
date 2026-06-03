---
related_code:
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
plan_sources:
  - user: 2026-05-31 完善 ECS 到渲染链路，参考 Unity SRP / Unreal / Bevy / Fyrox / WGPU 架构
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/workflow.xml
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/main-plan.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m00-baseline-evidence/plan.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m01-ecs-schedule-foundation/plan.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m02-derived-transform-active/plan.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m03-canonical-render-extract/plan.md
  - docs/zircon_runtime/scene/ecs.md
  - docs/zircon_runtime/scene/render_extract.md
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/m5_flagship_slots.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - tests/acceptance/ecs-to-render-chain.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m00-baseline-evidence/baseline-commands.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m01-ecs-schedule-foundation/validation-evidence.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m02-derived-transform-active/validation-evidence.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m03-canonical-render-extract/validation-evidence.md
doc_type: workflow-detail
---

# ECS To Render Workflow Baseline

This document records the M00 evidence-only baseline for the scene ECS to render chain. It describes the current owner path and risks before later milestones change behavior. It does not introduce a new runtime contract by itself.

## Scope Boundaries

- `zircon_runtime::scene::World` remains the authoritative runtime scene model.
- Typed ECS scheduling lives under `zircon_runtime::scene::ecs`; fixed scene maps still hold product-facing serialized/editor/render-extract fields until later hard cutovers replace them.
- `RenderFrameExtract` is the renderer-facing production DTO. `SceneViewportRenderPacket` and `RenderFrameExtract::from_snapshot(...)` remain preview, roundtrip, and test adapter surfaces.
- Concrete GPU resource preparation, compiled render graph execution, WGPU submit, history, and runtime-provider sidebands are owned by `zircon_runtime::graphics`.
- Virtual Geometry and Hybrid GI are advanced opt-in capabilities. Empty or disabled sidebands in a baseline frame do not mean the default renderer depends on those providers.

## Current Frame Workflow

The current scene schedule order is:

```text
First -> PreUpdate -> FixedUpdate -> Update -> PostUpdate -> Last -> RenderExtract
```

`SystemStage` already contains `PostUpdate` and `RenderExtract`, and there is no `LateUpdate` stage in `zircon_runtime/src` at the time of this baseline. Built-in scene systems are registered as:

| Stage | Built-in systems | Purpose |
| --- | --- | --- |
| `PostUpdate` | `zircon.scene.hierarchy_validity`, `zircon.scene.active_hierarchy`, `zircon.scene.world_transform`, `zircon.scene.node_cache` | Validate hierarchy and flush derived active/transform/node cache state after mutations. |
| `RenderExtract` | `zircon.scene.render_extract_prepare` | Flush pending render-extract dirty state before scene rows are read. |

`WorldDriver::tick_level(...)` reads the world's configured stage order and registered descriptors, then runs each stage through `SceneScheduleRunner::run_stage(...)`. The runner merges internal systems, native ECS systems, explicit `ApplyDeferred` barriers, and plugin hooks into sorted scheduled steps. On successful stage completion it also calls `world.run_internal_scene_systems_for_stage(stage)`, which is useful for final dirty-state flushing but is a risk if future internal systems gain non-idempotent side effects.

## Prepared Render Frame Extract Path

The production scene extraction path is:

1. A host or caller asks a `RenderExtractProducer` for `RenderFrameExtract`.
2. `LevelSystem::build_render_frame_extract(...)` enters `World` mutably and delegates to `World::build_prepared_render_frame_extract(...)`, then appends cached animation pose sidebands.
3. `World::build_prepared_render_frame_extract_for_request(...)` runs `RenderExtract` built-ins before reading camera, active state, transforms, render layers, mesh rows, sprites, lights, and visibility inputs.
4. The world builds `RenderViewExtract`, `GeometryExtract`, `LightingExtract`, `PostProcessExtract`, `DebugOverlayExtract`, `SpriteExtract`, `ParticleExtract`, and `VisibilityInput` directly.
5. `WgpuRenderFramework::submit_frame_extract(...)` builds a `FrameSubmissionContext`, prepares advanced provider submissions, applies effective advanced extracts and postprocess graph state, converts to `ViewportRenderFrame`, and asks `SceneRenderer` to render through the compiled pipeline.

The read-only `World::to_render_frame_extract()` helper clones the world before prepared extraction. That preserves callers that only have `&World`, but the mutable `LevelSystem` route is the authoritative scheduled path because it can flush dirty state on the live world.

## Snapshot Adapter Boundary

`SceneViewportRenderPacket` still exists for preview and legacy roundtrip surfaces. The adapter `RenderFrameExtract::from_snapshot(...)` converts that packet into a frame DTO, but it does not populate the full advanced sideband shape: Hybrid GI is `None`, sprites and particles default empty, and Virtual Geometry only carries the debug override rather than the direct scene-produced sideband.

The production scene module currently has a structural test guard that rejects `RenderFrameExtract::from_snapshot(...)` inside `zircon_runtime/src/scene/render_extract/mod.rs`. Adapter use remains visible in framework, graphics, and plugin tests where synthetic frame construction is intentional.

## Owner Map

| Concern | Current owner path | Current M00 fact | Follow-on milestone pressure |
| --- | --- | --- | --- |
| Stage vocabulary | `zircon_runtime/src/scene/ecs/system_stage.rs` | `PostUpdate` and `RenderExtract` exist; `LateUpdate` absent. | M01 should audit convergence and tests rather than recreate these names. |
| Stage ordering and registry | `zircon_runtime/src/scene/ecs/schedule.rs`, `scene_system_registry.rs` | Default order is Bevy-style with Zircon `RenderExtract`; built-ins are already registered. | M01 should verify native system, hook, deferred, and diagnostic behavior. |
| Stage execution | `zircon_runtime/src/scene/module/world_driver.rs`, `scene/ecs/schedule_runner.rs` | Stage runner merges built-ins/native/hooks and flushes deferred commands. | M01 should decide whether the post-success built-in flush can double-run future side effects. |
| Derived scene state | `zircon_runtime/src/scene/world/dirty_state.rs`, `derived_state.rs` | Active hierarchy, world transform, node cache, and render-extract freshness are dirty-state driven. | M02 should keep derived systems stable and verify parent reorder/inactive/static-dynamic cases. |
| Frame DTO contract | `zircon_runtime/src/core/framework/render/frame_extract.rs`, `scene_extract.rs` | `RenderFrameExtract` carries view, geometry, lighting, postprocess, debug, sprites, particles, visibility. | M03-M06 should add real material/postprocess data through this DTO rather than snapshot defaults. |
| Scene producer | `zircon_runtime/src/scene/world/render.rs`, `scene/render_extract/mod.rs`, `scene/level_system_render_extract.rs` | Scene producer builds frame sections directly and runs `RenderExtract` built-ins before reads. | M03 should keep direct construction canonical and keep snapshot conversion preview/test-only. |
| Materials | `zircon_runtime/src/core/framework/render/material/**`, `zircon_runtime/src/graphics/scene/resources/**` | Mesh rows carry model/mesh/material handles, tint, alpha mode side inputs, and morph weights; renderer resource streamer owns prepared GPU material state and fallback policy. | M04-M05 must define/consume neutral material snapshots instead of raw asset/editor state. |
| Postprocess | `PostProcessExtract` in `frame_extract.rs`, renderer postprocess stack under `zircon_runtime/src/graphics/scene/scene_renderer/**/post_process/**` | Current scene extract uses default bloom/color-grading settings and the submit context can override effective graph state. | M06 must add scene-owned volume blending and prove defaults are not a false success path. |
| WGPU submit | `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/**` | Submit consumes `RenderFrameExtract`, prepares advanced sidebands, resolves history, builds runtime frame, renders with compiled pipeline, records stats. | M08 should become the only render submission path after materials/postprocess/SRP requirements are ready. |
| Visibility | `zircon_runtime/src/graphics/visibility/**` plus `VisibilityInput` in `frame_extract.rs` | Visibility inputs are derived from the same extracted mesh/sprite rows. | M03/M10/M11 must keep visibility, VG, and GI update plans aligned with canonical frame rows. |
| Virtual Geometry | `RenderVirtualGeometryExtract`, `zircon_runtime/src/graphics/virtual_geometry_runtime_provider/**`, `zircon_plugins/virtual_geometry/runtime/**` | Scene can emit an empty VG sideband/debug override; runtime activation is capability/provider/pipeline gated. | M10 deepens opt-in inputs without making default rendering depend on VG. |
| Hybrid GI | `RenderHybridGiExtract`, `zircon_runtime/src/graphics/hybrid_gi_runtime_provider/**`, `zircon_plugins/hybrid_gi/runtime/**` | Scene baseline uses disabled/default GI sideband; submit clears runtime state when capability/provider is not active. | M11 deepens opt-in inputs without coupling to VG or default lighting. |

## Reference Matrix

| Reference | Relevant pattern | Zircon landing implication | Intentional divergence |
| --- | --- | --- | --- |
| Bevy schedules and extract | `First/PreUpdate/Update/PostUpdate/Last`, transform propagation in `PostUpdate`, explicit render extract schedule. | Confirms `PostUpdate` as the derived-state boundary and supports an explicit `RenderExtract` stage. | Do not introduce `bevy_ecs`; keep Zircon `World`, `Schedule`, and DTOs self-owned. |
| Unity Graphics / SRP | Renderer features enqueue passes; RenderGraph owns pass/resource validation; VolumeManager blends per camera/trigger. | Supports renderer feature assets, pass requirements, and future postprocess volume blending. | Do not copy Unity C# APIs or serialization. |
| Unreal RDG | Pass declarations drive resource lifetimes, barriers, culling, execution, and debug traces. | Supports explicit compiled-pipeline/resource diagnostics in WGPU submit. | Do not clone RDG API or Unreal resource layouts. |
| Unreal Nanite | Virtual geometry is view-packed, budgeted, visibility/page/feedback driven. | Supports opt-in VG extract/provider inputs: camera, material, visibility, cluster/page budgets, upload plan, debug stats. | No Nanite algorithm or asset-format parity is promised. |
| Unreal Lumen | GI is a persistent scene representation with cards, probes, surface cache, trace regions, budgets, history, and debug state. | Supports opt-in HGI extract/provider inputs and debug/readback contracts. | No Lumen algorithm parity; default rendering must not require HGI. |
| Fyrox | Rust engine layering keeps scene authority, renderer caches/resources, visibility, and editor/runtime separation distinct. | Confirms Zircon should keep runtime scene authority separate from renderer-owned GPU caches and editor-only authoring state. | Do not switch to Fyrox scene graph or import Fyrox renderer code. |

## M01 Pre-Start Risks

M00 evidence partially supersedes the original M01 wording that says to add `PostUpdate` and remove `LateUpdate`: current code already has `PostUpdate`, has `RenderExtract`, and no `LateUpdate` was found. M01 should therefore narrow to convergence and verification: schedule runner behavior, native-system execution, hook ordering, `ApplyDeferred`, diagnostics, conflict graph coverage, and focused test repair.

Other risks that later milestones must carry forward:

- Direct scene extraction and snapshot adapters still coexist. Renderer production paths must use `RenderFrameExtract`, not a lossy snapshot bridge.
- `RenderExtract` built-ins can be run from schedule execution and from direct extraction helpers. Future systems must stay idempotent or the runner/extract boundary must be tightened.
- Scene-produced VG/HGI defaults are placeholders unless the pipeline/profile/provider explicitly enables those advanced capabilities.
- Postprocess currently reaches the renderer through defaults and submit-context overrides; scene-owned volume blending is still later work.
- M00 recorded a non-green focused Cargo baseline because cold-target compilation timed out and an active shared target hit unrelated morph-weight schema drift. M01 isolated that baseline with `D:\cargo-targets\zircon-m01-schedule-convergence` and restored the focused scene schedule/scene suites: `cargo test -p zircon_runtime --lib scene::tests::ecs_schedule --locked --jobs 1` passed with `36 passed`, `cargo test -p zircon_runtime --lib scene::tests::ecs_scheduled_native_systems --locked --jobs 1` passed with `6 passed`, and the final `cargo test -p zircon_runtime --lib scene::tests --locked --jobs 1` rerun recorded in `m01-ecs-schedule-foundation/validation-evidence.md` passed with `189 passed`.

## M01 Schedule Convergence Evidence

M01 remained a verification/convergence milestone. It added structural coverage guards only, with no production runtime behavior change:

- `scene_ecs_does_not_reintroduce_late_update_stage_or_compatibility_path` scans the scene scheduling owner paths (`src/scene/ecs`, `src/scene/module`, `src/scene/world`) for `LateUpdate`, guarding against local aliases, shims, compatibility stages, and re-export bridges.
Together with the existing schedule tests, M01 covers stage order, built-in registration, native system ordering, hook ordering, deferred command application, duplicate/blank/missing-resource diagnostics, conflict graph stage boundaries, event/message/resource/component access conflicts, `ApplyDeferred` barrier batches, and render-extract hook positioning. The remaining architectural warning is future-facing: `SceneScheduleRunner` still runs internal descriptors as sorted steps and then performs a success-only final `run_internal_scene_systems_for_stage(stage)`. That is safe for the current dirty-flag-driven built-ins, but future non-idempotent internal scene systems must either avoid that stage path or tighten the runner contract first.

## M02 Derived-State Convergence Evidence

M02 keeps the existing scheduler and built-in stage order intact. It strengthens the existing dirty-flag/idempotent derived-state path instead of adding a new internal system: hierarchy, active hierarchy, world transform, and node-cache still run in `PostUpdate`, while `RenderExtractPrepare` still flushes those systems before render rows are read. No `LateUpdate` compatibility path, `bevy_ecs` dependency, reference-engine source copy, app loop change, or `SceneScheduleRunner` contract/order change is part of M02.

The M02 behavior repair is intentionally narrow:

- `World::world_matrix(...)` now uses the same projected read path as `world_transform(...)`, so callers that read a matrix before `PostUpdate` observe the current pending hierarchy/transform inputs without clearing dirty flags.
- `World::set_active_camera(...)` marks node-cache/render-extract freshness when it actually changes the active camera. Re-selecting the same camera remains a no-op and does not add dirty work.
- `MeshRenderer.morph_weights.N` property-path writes now treat vector extension as a real mutation even when the new value is the default `0.0`, ensuring node-cache/render-extract freshness is marked after the stored component shape changes.

`zircon_runtime/src/scene/tests/derived_state.rs` covers M02's bottom-up surface: retained node-cache staleness versus projected fresh reads, no-op setter cleanliness, invalid-import cleanup and out-of-order import preservation, cycle rejection without state corruption, inactive/reactivated ancestor propagation, large hierarchy transform/active propagation, mobility validation and visibility bucket freshness, legacy viewport plus direct `RenderFrameExtract` freshness, property-path node-cache dirty marking, and active-camera render freshness. Final M02 validation used `D:\cargo-targets\zircon-m02-derived-state`: `cargo fmt --all --check` passed, the focused `scene::tests::derived_state` filter passed with `10 passed`, exact schedule/performance guard commands each passed with `1 passed`, `scene::tests::world_basics` passed with `14 passed`, and the aggregate `scene::tests` filter passed with `199 passed`. These counts are focused M02 evidence only, not workspace-wide/root/plugin acceptance.

## M03 Canonical RenderFrameExtract Evidence

M03 keeps the existing scheduler, stage order, app/editor loops, profile selection, and renderer submit ordering intact. It records `RenderFrameExtract` as the canonical production scene DTO and keeps `SceneViewportRenderPacket`, `RenderFrameExtract::from_snapshot(...)`, `RenderFrameExtract::to_scene_snapshot(...)`, and `ViewportRenderFrame::from_snapshot(...)` as preview, roundtrip, test, or synthetic validation adapters.

The focused `zircon_runtime/src/scene/tests/render_extract.rs` module locks the current producer contract without moving feature depth from later milestones: direct `World` extraction fills view, geometry, mesh phase inputs/queues, sprites and sprite phase queues, active lights, postprocess defaults, empty VG sideband/debug override, disabled GI sideband, particles default, and visibility buckets from the same extracted mesh/sprite rows. It also verifies inactive-camera empty payload shape, camera-layer filtering for meshes/sprites/visibility, explicit request camera layer overrides, and `LevelSystem` animation pose sideband merging for mesh+skeleton entities only. A structural guard rejects snapshot adapters in scene production owners and in non-test `submit_frame_extract` code.

The M03 evidence is recorded in `m03-canonical-render-extract/validation-evidence.md` using `D:\cargo-targets\zircon-m03-canonical-render-extract`. It is focused `zircon_runtime` evidence only and must not be represented as root-workspace, plugin-workspace, export, full workspace, or final acceptance green.

The 2026-06-02 named validation stage records exact command results in the milestone evidence. Focused runtime checks passed for the new render-extract module (`6 passed`), structural no-adapter guard (`1 passed`), exact legacy `ecs_schedule` canonical/layer/override/inactive-camera guards (`1 passed` each), `world_basics` (`14 passed` after a transient concurrent renderer-output compile mismatch cleared on rerun), the M5 sideband smoke (`1 passed`), graphics visibility (`23 passed`), and aggregate `scene::tests` (`205 passed; 2272 filtered out`). `cargo check -p zircon_runtime --lib --locked --jobs 1` passed with existing warnings after rerunning through concurrent submit-context and plugin feature-shape visibility mismatches. Root `cargo fmt --all --check` passed only after `cargo fmt --all` formatted unrelated active-lane editor/render/plugin/framework files, so M03 does not claim those lanes are functionally validated.
