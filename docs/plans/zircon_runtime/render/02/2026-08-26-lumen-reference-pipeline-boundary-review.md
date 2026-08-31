---
date: 2026-08-26
related_plan: docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
doc_type: reference-architecture-review
status: design-review-complete
coordination_owner: docs/plans/zircon_runtime/render/02
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs
reference_code:
  - dev/LumenInUE5.5.4WithComputeShader/App.cpp
  - dev/LumenInUE5.5.4WithComputeShader/RenderPass.cpp
  - dev/LumenInUE5.5.4WithComputeShader/Pipeline.cpp
---

# Lumen Reference Pipeline Boundary Review

## Reference Scope

`LumenInUE5.5.4WithComputeShader` is a Direct3D 12 learning prototype derived
from UE 5.5.4 techniques, rather than Unreal Engine's renderer implementation.
It is useful for phase dependencies and temporal-resource semantics. Its global
resource ownership, hand-written descriptors, explicit D3D12 barriers, and
fixed resource dimensions are not a WGPU architecture to port.

## Observed Frame Order

`App::RenderOneFrame` establishes this high-level dependency sequence:

1. Advance frame index and choose current/previous ping-pong resources.
2. Clear frame-local work, run pre-depth, then construct HZB.
3. Update surface/card cache before the base pass consumes its material data.
4. Run base, shadow-mask, and direct-lighting passes.
5. Run radiance-cache and screen-probe compute stages, including indirect work
   generation and temporal reprojection against explicit previous resources.
6. Compose indirect lighting into scene color, tone-map, and present.

The useful rule is that compute GI consumes published scene/depth/HZB artifacts;
it does not own or rebuild the mesh-command system. That agrees with the
Render-02 boundary where `MeshPassCommandBuffers` is a completed input artifact
to graph execution rather than a per-GI-pass mutable list.

## WGPU Architecture Decision

Adopt the dependency and ownership model, not the prototype mechanics:

- Mesh extraction, pipeline-variant resolution, cached-command transactions,
  and command publication remain in the Render-02 owner path.
- The render graph owns ordering and WGPU read/write hazard declaration between
  depth, HZB, base lighting, and later compute GI nodes. GI code must not issue
  a second mesh draw build or take mutable command-cache ownership.
- Temporal GI inputs must be named current/previous graph resources with a
  persistent history owner. They cannot be acquired as transient pool backing,
  and hidden view selection must not retire a live static mesh command.
- Indirect arguments and visibility remaps are graph-execution products that
  consume the published mesh/GPUScene artifact. Their resource generation must
  be validated at the handoff; a recycled GPUScene span is not a stable cache
  identity.

## Explicit Non-Transfers

- Do not port `App.cpp` global pass pointers, mutable process-wide resource
  variables, or the manual current/last-frame pointer selection.
- Do not translate D3D12 `ResourceBarrier`, descriptor heap mutation, or one
  command-list submission per compute pass into WGPU code. The render graph and
  WGPU pass encoders are the required synchronization boundary.
- Do not reproduce fixed 4096/1024/512 resource sizes or hard-coded dispatch
  values. Resolution, atlas budgets, and workgroup dimensions require typed
  configuration owners and measured workload justification.

## MVP Order And Evidence

The MVP first closes deterministic mesh commands, depth/HZB, base lighting,
and persistent history ownership. Lumen-style surface cache, radiance cache,
and screen-probe stages follow only after those artifacts have current-source
validation. Any later GI optimization must add typed CPU/GPU observations first
and compare matched WPR, RenderDoc, and PNG artifacts under
`docs/tests/runtime/render`; this review makes no performance or power claim.
