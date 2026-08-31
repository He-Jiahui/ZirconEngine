---
title: World-owned render-dirty entity journal producer
status: dirty_journal_core_neutral_component_projection_and_render_extract_handoff_implemented_static_passed_managed_validation_unresolved
owner: render-03-gpu-scene
date: 2026-08-26
related_code:
  - zircon_runtime/src/scene/ecs/change_detection/component_mutation.rs
  - zircon_runtime/src/scene/ecs/change_detection/wrappers.rs
  - zircon_runtime/src/scene/world/render_dirty_journal
  - zircon_runtime/src/scene/world/render_component_changes
  - zircon_runtime/src/core/framework/render/frame_extract/scene_changes
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/tests/render_dirty_journal
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/PrimitiveComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PrimitiveSceneInfo.cpp
doc_type: implementation-record
---

# World-owned render-dirty entity journal producer

## Completed source slice

The scene world now owns the candidate-discovery side of the persistent RenderScene pipeline. Direct
component changes, raw mutable queries, actual lazy `Mut<T>` writes, component removals, and derived
active/transform propagation converge on one render-dirty owner. `RenderExtractPrepare` publishes an
immutable `Arc<RenderDirtyEntityJournal>` carrying a runtime-only world identity, monotonic journal
generation, source world generation/change tick, initial/full-reprojection flag, and a sorted unique
entity list.

This is deliberately graphics independent. It imports no WGPU, GPUScene, viewport, asset-residency,
or RenderScene primitive types. The later Render03 converter can inspect `ComponentTicks` only for
the published candidates and use a persistent `RemovedComponentReader<MeshRenderer>` for removals.
It must not replace this owner with full `Changed<T>` queries or build the delta per viewport.

The world now also owns that component projection. One cursor consumes each dirty publication once,
probes the fixed five render inputs only for published candidates, and reads bounded removal cursors
for mesh lifetime plus transform, active, layer, and mobility presence. The immutable artifact uses
`Unchanged`/`Present`/`Removed` sparse values, so a transform-only update does not clone the
`MeshRenderer` LOD, primitive, or morph vectors. Remove-then-readd collapses to one added upsert;
lost removal history or an explicit relevant-channel clear produces a typed full reprojection. The
projected `WorldMatrix` is the exact derived matrix; the Render03 consumer retains it without TRS
decomposition so hierarchical shear is not discarded, and projects the canonical local bounds into
one retained conservative world envelope at primitive construction rather than once per camera.

The published payload no longer exposes scene component wrappers. Its immutable DTO owner is
`core/framework/render/frame_extract/scene_changes/`: mesh component data is projected once to core
resource handles and immutable base/all-LOD arrays; derived state becomes exact `Mat4`, `bool`,
`u32`, and core `Mobility`. `GeometryExtract` carries the artifact as one optional `Arc`, and both
active and inactive camera extraction attach the same world-published allocation. The graphics
projector's first application/replay test consumes this frame field and a separate regression rejects
a mismatched external frame world before resolver work, proving the intended
`World -> RenderFrameExtract -> Render03` boundary without a global side channel or reverse core
dependency.

Lazy `Mut<T>` preserves its existing change-detection contract. Constructing or reading a wrapper
does not record a world mutation. Its first `DerefMut`, `as_mut`, `into_inner`, or explicit
`set_changed` appends one component mutation record; repeated writes through the same wrapper do not
append duplicates. Before an internal scene system runs, World drains those records and applies the
same inspection, scene-binding, hierarchy, active, transform, node-cache, generation, and render
invalidation effects as direct component mutation. Pending real writes are included in
`World::world_generation`, including across `World::clone`.

## Algorithm and ownership

| Operation | Bound | Allocation/ownership |
|---|---:|---|
| stable-frame pending check | O(1) | none |
| stable publication borrow | O(1) | one `Arc` refcount increment |
| direct mutation mark | amortized O(1) | append only when not already in full-reprojection mode |
| actual lazy query write | amortized O(1) | one typed record, one uncontended world-owned lock |
| publish C candidates | O(C log C) | O(C), one canonical immutable entity list |
| derived subtree propagation | existing O(S) | one O(1) entity mark per visited derived row |
| incremental component projection | O(5C + R log R) | O(C + R), one immutable shared artifact |
| transform-only payload projection | O(1) per candidate | zero `MeshRenderer` payload clones |
| removal-history recovery | O(N log N) | O(N), isolated to typed full reprojection |

The sort is paid once at the world publication boundary. Viewports and Render03 consumers share the
same canonical order and must not sort private copies. Stable frames neither scan entities/components
nor allocate a replacement journal. The lock is a correctness-first bridge for mutation wrappers
that hold component borrows while recording; no performance claim is made until WPR/ETW profiles
uncontended and future parallel-system workloads. If contention is measurable, the replacement must
be a profiled worker-local append/merge design preserving the same publication ABI.

## Authored guards and current evidence

Thirteen folder-backed tests cover the five dirty-publication cases above plus eight projection
cases:

- distinct world identity and exact `Arc` reuse on a stable frame;
- sorted deduplication and preservation of a removed entity candidate;
- parent transform propagation publishing every affected descendant;
- read-only lazy queries producing no world generation or publication change, while an actual write
  records once even after repeated `set_changed`;
- lazy transform mutation refreshing child derived state, plus effective generation and new identity
  across World clone.
- initial full projection and exact stable artifact `Arc` replay;
- candidate-only tick classification that skips an unrelated `Name` mutation;
- sparse layer removal and transform-only zero-mesh-clone projection;
- single-consumption mesh removal and remove-then-readd conflict collapse;
- removal-retention loss forcing a full current-world reprojection.
- repeated full-reprojection requests coalescing into one next-generation
  `Full(JournalRequested)` source snapshot.
- repeated frame extraction retaining exact component-artifact `Arc` identity, including the
  inactive-camera path.

Fresh scoped `rustfmt --edition 2021 --check`, module/call-site scans, line-budget checks, forbidden
pattern scans, and `git diff --check` pass. New production owners remain below the 800-line review
warning and add no unsafe,
compatibility alias, full-scene scan, WGPU dependency, or suppression attribute.

The earlier managed `zircon_runtime` check did not start Cargo because unrelated pre-existing D/E/F
build directories triggered `unmanaged_artifacts_detected`. The focused projection request was later
accepted but returned `command_post_timeout` while acquiring Cargo and supplied no terminal result.
Neither request is a compile/test pass; the coordinator is not polled and no external directory is
deleted or modified.
No WGPU run, RenderDoc capture, framebuffer PNG, timing, power, or comparison data exists for this
CPU producer slice.

## Next source work

1. The Render03 resolved-input converter now turns sparse component patches plus exact base/all-LOD
   bounds into one atomic `RenderSceneDelta`, and the neutral source artifact now reaches that
   converter through `GeometryExtract`. Connect its narrow resolver to the unified residency
   ticket/generation owner and add typed pending/fail-open primitive state without introducing a
   third cache or fabricated bounds. Product scheduling must route a typed consumer discontinuity
   back through `request_full_render_component_projection`; the source-side request and recovery
   artifact are implemented, but that runtime feedback edge is not yet wired.
2. Only after current-source validation and product parity, connect the journal consumer to real
   GPUScene capacity/upload/retirement owners and remove pending-draw lifetime ownership.

This record is not an accepted milestone. Commit, coordinator closeout, WeCom metrics, RenderDoc,
and `docs/tests/runtime/render` PNG evidence remain gated on current-source product validation.
