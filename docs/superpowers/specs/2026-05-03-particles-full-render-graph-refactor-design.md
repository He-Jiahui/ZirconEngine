# Particles Full Render Graph Refactor Design

## Goal

Implement particles M6 through a renderer-owned render graph execution/resource model instead of the current metadata-only executor path. The result must let plugin-owned GPU particle work run inside the renderer encoder lifetime, consume named graph resources, emit neutral readback outputs, and keep `zircon_runtime` independent from `zircon_plugins/particles`.

## Architecture

`zircon_runtime` owns the neutral graph contracts: compiled pass stage metadata, a GPU-capable execution context, a named execution resource registry, execution records, and neutral readback sinks. Plugin crates own concrete behavior behind render pass executor registrations. The scene renderer validates the compiled pipeline before rendering, then executes graph passes at renderer stage boundaries while the frame command encoder, target views, frame extract, resource registry, and neutral output sink are live.

## Reference Evidence

- `dev/bevy/crates/bevy_render/src/renderer/render_context.rs` shows render-graph execution using a render context that owns access to a command encoder and render device, with deferred command buffer submission preserving order.
- `dev/bevy/examples/shader/gpu_readback.rs` shows compute work plus GPU readback as render-graph work, not pre-render metadata validation.
- `dev/Graphics/Tests/SRPTests/Projects/VisualEffectGraph_URP/Assets/GraphicsTests/Scripts/OutputTextureFeature.cs` shows Unity Graphics RenderGraph passes binding target resources and executing command-buffer work through pass functions.
- `dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIGPUReadback.h` shows readback as an explicit enqueue/copy/lock lifecycle separated from immediate CPU ownership.

## Chosen Behavior

- `CompiledRenderPipeline` records the stage for every graph pass by pass name.
- `RenderPassExecutionContext` carries pass metadata plus an optional renderer GPU payload with `wgpu::Device`, `wgpu::Queue`, `wgpu::CommandEncoder`, `ViewportRenderFrame`, named execution resources, scene bind group, and plugin renderer output sink.
- `RenderPassExecutorRegistry` stores executor objects and invokes them with the GPU-capable context. Function executors remain valid through an adapter, but they receive the new context and no longer define the only execution model.
- `RenderGraphExecutionResources` maps graph resource names to imported texture views and owned buffers. Initial imported resources include `scene-color` and `scene-depth`; plugin executors may add and consume named GPU buffers such as `particles.gpu.alive-indices` and `particles.gpu.indirect-draw-args`.
- Scene rendering executes graph passes inside `render_compiled_scene()` at stage boundaries. The transparent stage runs where particle drawing belongs, before CPU billboard fallback emits transparent sprites.
- Graph execution records include stage metadata so tests can prove the renderer executed real graph stages, not only pre-render metadata.

## Boundaries

- `zircon_runtime` must not import `zircon_plugins/particles`.
- The old pre-render `execute_compiled_graph_passes()` metadata loop is removed from `render_frame_with_pipeline.rs`.
- No particle-specific branch is allowed in shared graph execution; particles use normal executor ids and resource names.
- CPU particle billboard rendering remains a fallback when GPU resources are absent.
- Current unrelated physics/animation module churn may block `zircon_runtime` package validation; that blocker must be reported separately.

## Validation Plan

- Add/adjust runtime graph tests for compiled pass stage preservation, executor context GPU payload detection, named resource registry access, and execution record stage counts.
- Add/adjust renderer tests so the registry executes graph passes from inside the encoder lifetime and preserves transparent-stage ordering.
- Keep particles plugin tests covering executor contracts, GPU frame planning, indirect readback DTO projection, and CPU fallback.
- Update `docs/assets-and-rendering/render-framework-architecture.md`, `docs/zircon_plugins/particles/runtime.md`, and the particles session note with implementation files, tests, and blockers.
