---
related_code:
  - zircon_runtime/src/graphics/scene/render_scene/change_journal.rs
  - zircon_runtime/src/graphics/scene/render_scene/resource_dependencies.rs
  - zircon_runtime/src/graphics/scene/render_scene/scene.rs
  - zircon_runtime/src/graphics/scene/render_scene/tests/resource_dependencies.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
plan_sources:
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrimitiveUpdates.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/DynamicTextureInstanceManager.cpp
doc_type: architecture-review-and-implementation-record
review_status: review_complete
implementation_status: before_after_arc_net_delta_and_residency_contract_foundation_implemented_product_scheduling_pending
---

# RenderScene resource dependency delta review

## Decision

`RenderSceneUpdatedPrimitive` must retain both the previous and current immutable primitive. The
previous value is borrowed from the scene's existing `Arc` immediately before replacement; it is not
deep-cloned. This is the minimum prerequisite for a future single residency authority to calculate
exact geometry and material reference removals/additions from the same scene journal.

The journal must also seal a deterministic net resource-reference delta once, from its additions,
removals, and before/after updates. It covers base and all LOD model/mesh/material handles, embedded
primitive mesh/material bindings, common material overrides, and the animation skeleton. Duplicate
references inside one primitive count once. Opposing acquire/release changes for the same typed
resource cancel within the journal so downstream residency does not churn a still-referenced asset.
Texture, shader, and other transitive material dependencies are deliberately not parsed by
RenderScene; the 09D residency authority must expand them from generation-bound asset metadata.

This slice does not introduce a residency manager, ticket, GPU resource, WGPU call, resource cache,
or product scheduling path. Those remain owned by 09D and 09A. It only preserves information that is
otherwise destroyed at the RenderScene mutation boundary.

## Current-source finding

The current `ResourceStreamer` cannot be used as the RenderScene geometry resolver:

- `ensure_scene_resources` walks frame-visible meshes and synchronously calls `ensure_mesh`,
  `ensure_model`, and `ensure_material`;
- mesh/model ensure reads revisions, loads and clones complete CPU assets, builds derived geometry,
  and creates WGPU resources on the caller path;
- it owns independent permanent maps for models, meshes, materials, textures, mip state, output
  targets, LUTs, and shaders;
- model dependency freshness is polled and external mesh payloads are loaded while resolving model
  geometry.

Connecting this owner to `RenderSceneComponentProjector` would preserve the old frame-driven load
architecture and violate 09D P0-1, P0-4, P0-5, and P0-6. Product wiring therefore remains deferred.

The new persistent `RenderScene` journal already publishes additions and removals with immutable
primitive payloads. Before this change, updates published only the new payload. When geometry or
material dependencies changed, the old dependency set was lost as soon as `install_update` replaced
the scene slot.

## Reference finding

Unreal does not require scene extensions to rediscover old state after mutation. In
`UpdateAllPrimitiveSceneInfos`, the pre-update change set exposes primitives that exist before the
update and processes removal observers before mutating the scene. The dynamic render-asset instance
manager similarly removes prior component references before inserting refreshed references. Zircon
does not copy these containers, but it preserves the same ordering and old-state observability in an
immutable journal suitable for multiple consumers.

## Algorithm review

| Alternative | Steady update cost | Retained state | Decision |
|---|---:|---:|---|
| Rescan every live primitive after a dependency change | O(N log D) or O(ND) | none | Reject: stable work scales with scene size |
| Residency consumer keeps a per-primitive dependency shadow | O(CD) | O(ND) plus another identity map | Reject: creates the forbidden third cache |
| Journal update retains previous and current `Arc` | O(C) pointer clones | O(C) old payload lifetime per journal | Select |
| Journal seals a sorted net typed-resource delta | O(CD log D + K log K) | O(K) for the immutable journal | Select |

`N` is live primitive count, `C` is changed primitive count, and `D` is dependencies per primitive.
The selected change adds one strong-pointer clone per published update. It does not clone model, mesh,
material, LOD, morph, bounds, or deformation payloads. Consumers can compare before/after only when
the dirty flags include geometry or material, and ignore the previous payload for transform-only
GPU staging.

`K` is the number of dependency observations across changed primitives. A private ordered key uses a
stable kind tag plus `ResourceId`; the published entry remains an `UntypedResourceHandle`. This avoids
changing the interface-owned `ResourceKind` ordering contract. The delta is not a residency state
table and owns no request lifecycle.

The implementation uses two reusable per-journal dependency `Vec` scratch buffers and one contiguous
observation buffer. Each primitive list is sorted and deduplicated in place; before/after updates use
a linear two-pointer difference; all observations are sorted once and folded into the immutable net
delta. It does not allocate one tree or hash table per primitive.

## Required contract and guards

1. `RenderSceneUpdatedPrimitive::previous_primitive()` exposes the exact `Arc` that occupied the
   scene slot before the update.
2. `primitive()` continues to expose the exact current `Arc` installed in the scene.
3. The two pointers differ for a real update; their stable instance key and handle ownership remain
   identical.
4. A material-only regression proves the old and new dependency handles are both observable.
5. A transform-only regression proves the old immutable payload is retained without changing dirty
   classification or GPUScene work selection.
6. Initial add/remove publish inverse typed reference deltas; material replacement publishes only the
   old material release and new material acquire after model/mesh cancellation.
7. A complete-source regression covers all LODs, primitive bindings, material overrides, skeleton,
   and duplicate suppression with deterministic ordering.
8. A remove/add replacement that keeps the same resources cancels to an empty journal delta.

## Implemented result

- `RenderSceneUpdatedPrimitive` now carries the exact previous and current `Arc` values.
- `RenderSceneChangeJournal` seals one immutable sorted `Arc<[RenderSceneResourceReferenceDelta]>`.
- dependency scanning is skipped for no-op and transform-only updates and exposes four O(1) build
  counters so this is distinguishable from scan-then-cancel behavior;
- add/remove, material replacement, cross-primitive cancellation, and complete camera-neutral source
  coverage have five folder-backed tests; the existing transform/no-op tests also assert zero delta
  and zero projected dependency payloads;
- the RenderScene subtree now has 60 authored tests, and the combined RenderScene, GPU journal
  consumer, and scene producer scope has 81 authored tests.

Scoped `rustfmt --check`, line-budget, forbidden-pattern, reverse-dependency, and WGPU leakage scans
pass for this owner. `resource_dependencies.rs` is 301 lines, its test owner is 310 lines, and the
largest RenderScene production owner remains 633 lines. These are static results only.

## Acceptance boundary

Managed Cargo is currently unresolved at `cargo.acquire`; this record does not claim compile or test
execution. No RenderDoc, framebuffer, timing, power, or allocation comparison applies to this CPU
journal contract. A 09D-owned CPU residency authority now consumes the typed net delta and issues
generation-bound all-LOD/bootstrap tickets while preserving last-good active residency. Runtime11
task admission, semantic-block I/O, 09A upload/fence retirement consumption, and product frame
scheduling remain pending.
