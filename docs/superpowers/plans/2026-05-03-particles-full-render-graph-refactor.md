# Particles Full Render Graph Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the metadata-only render pass executor path with a renderer-hosted graph execution/resource model that can carry particles M6 GPU work.

**Architecture:** Add neutral stage metadata, GPU execution context, named execution resources, and stage-aware execution records in `zircon_runtime`; then move graph execution into `SceneRendererCore::render_compiled_scene()` while keeping plugins as normal executor consumers.

**Tech Stack:** Rust, Cargo workspace, `wgpu`, `zircon_runtime` render graph/scene renderer, `zircon_plugins/particles` runtime.

---

## Milestone 1: Neutral Graph Execution Foundation

**Files:**
- Modify: `zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs`
- Modify: `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registration.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs`
- Create: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs`

- [x] Add `CompiledRenderPipelinePassStage` and populate it during pipeline compile.
- [x] Add `RenderGraphExecutionResources` for imported texture views and owned buffers keyed by graph resource name.
- [x] Extend `RenderPassExecutionContext` with optional GPU payload while preserving pass metadata access.
- [x] Convert the executor registry to invoke executor objects, with function-pointer registrations adapted to the new context.
- [x] Add tests for pass-stage preservation, GPU payload availability, missing named resources, and executor object invocation.

## Milestone 2: Renderer Stage Execution Cutover

**Files:**
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs`

- [x] Remove the pre-render metadata-only graph execution loop from `render_frame_with_pipeline.rs`.
- [x] Execute graph stages inside `render_compiled_scene()` while the frame command encoder is live.
- [x] Import `scene-color` and `scene-depth` target views into the execution resource registry.
- [x] Execute `Transparent` graph passes at the particle transparent stage boundary before CPU fallback sprite rendering.
- [x] Return and store the real graph execution record from the compiled-scene output.

## Milestone 3: Particles Integration Surface

**Files:**
- Modify: `zircon_plugins/particles/runtime/src/render/executors.rs`
- Modify: `zircon_plugins/particles/runtime/src/tests.rs`
- Modify: `zircon_runtime/src/core/framework/render/frame_extract.rs`
- Modify: `zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs`

- [x] Update particle executor contract tests to use the new GPU-capable context type.
- [x] Ensure particle readback outputs remain neutral and are collectable through the graph execution output sink.
- [x] Preserve CPU fallback behavior until GPU submissions provide named particle buffers.
- [x] Add regression coverage that particle executor ids still register as graph executors and validate their resource contracts.

## Milestone 4: Docs and Testing Stage

**Files:**
- Modify: `docs/assets-and-rendering/render-framework-architecture.md`
- Modify: `docs/zircon_plugins/particles/runtime.md`
- Modify: `.codex/sessions/20260503-1529-particles-gpu-renderer-closeout.md`

- [x] Update docs with the new graph execution/resource ownership model and implementation files.
- [x] Run scoped formatting checks over changed Rust files.
- [x] Run particles runtime/editor tests with the known external target dir.
- [x] Run focused `zircon_runtime --lib` graph/renderer tests if unrelated physics/animation churn no longer blocks runtime compilation.
- [x] Record exact validation evidence and any unrelated blockers in docs/session notes.

## Validation Evidence

- 2026-05-04: `Get-PSDrive -Name E` reported 133.51 GB free before validation.
- 2026-05-04: `cargo check -p zircon_runtime --tests --locked --offline --jobs 1 --target-dir "target\codex-shared-a" --message-format short --color never` passed with existing warnings.
- 2026-05-04: `cargo test -p zircon_runtime --lib graph_execution --locked --offline --jobs 1 --target-dir "target\codex-shared-a" --message-format short --color never` passed 16/16 filtered tests with existing warnings.
- 2026-05-04: `cargo test -p zircon_runtime --lib compile_preserves_renderer_stage_for_each_graph_pass --locked --offline --jobs 1 --target-dir "target\codex-shared-a" --message-format short --color never` passed 1/1 filtered test with existing warnings.
- 2026-05-04: `cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_particles_runtime --locked --offline --jobs 1 --target-dir "target\codex-shared-a" --message-format short --color never` passed 20/20 tests with existing runtime warnings.
- 2026-05-04: `cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_particles_editor --locked --offline --jobs 1 --target-dir "target\codex-shared-a" --message-format short --color never` passed 1/1 test with existing runtime/editor warnings.
- 2026-05-04: `rustfmt --edition 2021 --check <scoped render graph/runtime prepare/particles files>` passed.
- 2026-05-04: `git diff --check -- <scoped render graph/runtime prepare/particles/docs/session/plan files>` reported no whitespace errors; only LF-to-CRLF warnings.

## Residual Risk

- Workspace-wide validation was not run from this dirty checkout.
- A separate active VG/HGI runtime-prepare collector session overlaps the neutral collector plumbing; this particles lane only completed the shared propagation needed for scoped validation and did not wire VG/HGI plugin-owned collectors.
