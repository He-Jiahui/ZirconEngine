# Runtime07 F459 M2 Generation Topology Candidate Manifest

Plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
Milestone: M2
Status: `candidate_pending_managed_validation`
Files: ["zircon_runtime/src/scene/world/bootstrap.rs", "zircon_runtime/src/scene/world/derived_state.rs", "zircon_runtime/src/scene/world/dirty_state.rs", "zircon_runtime/src/scene/world/hierarchy_topology.rs", "zircon_runtime/src/scene/world/mod.rs", "zircon_runtime/src/scene/world/typed_api.rs", "zircon_runtime/src/scene/world/typed_api/component_mutation_effects.rs", "zircon_runtime/src/scene/world/world.rs", "zircon_runtime/src/scene/tests/derived_state.rs", "zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs", "docs/plans/zircon_runtime/runtime/07/2026-08-22-m2-world-derived-state-generation-topology-manifest.md"]

## Candidate scope

- `zircon_runtime/src/scene/world/bootstrap.rs`
- `zircon_runtime/src/scene/world/derived_state.rs`
- `zircon_runtime/src/scene/world/dirty_state.rs`
- `zircon_runtime/src/scene/world/hierarchy_topology.rs`
- `zircon_runtime/src/scene/world/mod.rs`
- `zircon_runtime/src/scene/world/typed_api.rs`
- `zircon_runtime/src/scene/world/typed_api/component_mutation_effects.rs`
- `zircon_runtime/src/scene/world/world.rs`
- `zircon_runtime/src/scene/tests/derived_state.rs`
- `zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs`

## Architecture and baseline

- Current-source review found that component mutation invalidation discarded the entity ID and
  reduced every transform or active change to a world-wide dirty bit. The retained node cache
  also lacked an entity-to-row projection, so one changed component rebuilt the full cache.
- Unreal `USceneComponent::PropagateTransformUpdate` and `UpdateChildTransforms` establish the
  primary reference: change propagation begins at the affected component and visits only its
  attached descendants. Bevy corroborates the ECS form with changed roots and `Children` walks.
- The candidate gives one `HierarchyTopology` ownership of stable roots, dense child ranges,
  topological entity order, and a monotonic structural generation. Typed mutations retain entity
  identity in coalesced transform, active, and node-cache frontiers. Raw hierarchy mutation and
  preflight rows retain the conservative full invalidation path.
- Before this candidate, the deterministic 1,000-node leaf-transform baseline recorded 1,000
  world-matrix propagations and 1,000 node-cache row rebuilds. The candidate target is one for
  each; a changed parent remains required to visit its 1,000-node subtree. Existing 1, 1,000, and
  100,000-node full-baseline fixtures remain the scale comparison points.

## Required evidence

- Managed Windows Cargo evidence for the derived-state fixture family using a non-C target root.
- Deterministic counters: leaf transform matrix/cache = `1/1`, parent transform matrix = `1000`,
  leaf active propagation/matrix/cache = `1/0/1`.
- Static structure checks must confirm one topology owner and no temporary full traversal for an
  already-current inspection subtree.
- This document records a candidate only. It must not be treated as an accepted milestone,
  performance p95 result, power result, or completion record before coordinator validation and
  independent review.
