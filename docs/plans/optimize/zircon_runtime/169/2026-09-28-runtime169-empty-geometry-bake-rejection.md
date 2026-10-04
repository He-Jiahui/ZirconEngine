---
title: Runtime169 no-source navigation bake does not invent walkable geometry
category: zircon_runtime
report_id: Runtime169-empty-geometry-bake-rejection-2026-09-28
date: 2026-09-28
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: static_review_complete_managed_tests_pending
performance_status: not_applicable
related_code:
  - zircon_plugins/navigation/runtime/src/manager/bake/asset.rs
  - zircon_plugins/navigation/runtime/src/tests/bake.rs
tests:
  - bake_surface_without_source_geometry_publishes_no_walkable_mesh
plan_sources:
  - docs/plans/optimize/zircon_runtime/169-runtime-navigation-current-working-tree-bake-artifact-query-crowd-editor-boundary-review.md
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
---

# Runtime169 no-source navigation bake does not invent walkable geometry

## Scope

When a selected navigation surface has no collected source triangles and was
not emptied by an obstacle or modifier, the bake previously asked the Recast
backend to create a 4-vertex, 2-triangle quad from the surface volume. The bake
then returned and published that synthetic quad as ordinary generated data.

The no-source branch now retains a warning and returns `NavMeshAsset::empty`.
The public bake report and generated snapshot therefore contain no walkable
polygons. Geometry-backed bakes and the existing obstacle/modifier empty path
are unchanged.

The focused regression uses the production `NavigationManager::bake_surface`
entry point with a `NodeKind::Empty` surface in `World::empty()`, so the
default cube from `World::new()` cannot add unrelated source geometry. It
checks zero source triangles, baked vertices, baked polygons, and report tiles;
empty vertices/indices/polygons/tiles in both the report and generated snapshot
assets; and a warning that names the missing source and states that no
walkable polygons were generated. The test file also handles all 21
`World::spawn_node` results for the current fallible API: 19 assigned entity
IDs and two intentionally unused spawn results are unwrapped.

## Scope limits

This implements the no-source outcome allowed by Runtime169 P0-02: an
explainable empty artifact instead of invented walkable geometry. A typed
`NoSourceGeometry` failure or quarantine is not required for this outcome.
Broader P0-02 product acceptance remains open until off-mesh-link behavior on
an empty artifact, and its resulting publication/query semantics, are decided
and covered by a focused regression. `finish_bake` can still attach off-mesh
links to the empty asset when present.

`manager/bake.rs` still computes and passes the old surface half-extent, now
unused by the no-source branch. That file has an active foreign Session owner,
so this slice leaves the call and helper in place. Removing that redundant
calculation is a small follow-up after ownership handoff.

No performance claim is made. Managed compilation and the focused behavior
test remain pending; this implementation did not run Cargo.

## Source evidence

| Path | Preimage SHA-256 | Candidate SHA-256 |
|---|---|---|
| `zircon_plugins/navigation/runtime/src/manager/bake/asset.rs` | `82d6012ff3c30d870dcfc2d66af5901a80e3033cb5cd75a2a5d9ec17e8a32a1a` | `f8b4fdea5dc235f523915cfecb89a1c9047a31716877bb5d9332c208540ae5ab` |
| `zircon_plugins/navigation/runtime/src/tests/bake.rs` | `f869e860dc855e5f38c8acb907a2d566e78c08ae15f563dd27902560f999a132` | `e09a20c589ff82e6b4b9f56b77eb4477c387b21c129c3900cb4d49ab2a6e0265` |

## Attribution audit

The initial four-path claim (`9fb756fa815447c8a600fd0c779d42a8`) expired at
2026-09-29 03:57:13 UTC. The writes that produced the prior candidate hashes
occurred after that expiry: `asset.rs` at 03:58:45.112 UTC, `tests/bake.rs` at
03:57:21.487 UTC, this record at 04:02:20.602 UTC, and the Astra record at
04:02:41.791 UTC. This attribution gap was disclosed to the root agent before
further edits. The uncertain 04:10 claim (`5c56c1f61fb146f0957deaf2beb8d74d`)
was reconciled as `command_request_not_found`. A fresh exact four-path claim
for `astra-optimize-20260926-batch-a` was accepted without conflicts under
request ID `d241026b20794661876fc469bbc76e6f`. The subsequent
`baseline.attribute` attempt returned terminal `baseline_lease_missing`
(request ID `bd4ddb3245a5461c91164763ef383652`) and did not attribute these
hashes. A new exact four-path claim for the same Session was accepted without
conflicts under request ID `10b1e6aa94bd4ce9b52034e696c96000`. Final content
hash attribution is a separate coordinator operation; use its exact terminal
receipt to determine status. This record does not claim that attribution
succeeded.

The two Rust paths are outside the frozen v11 manifest. Pinned rustfmt 1.94.1
with edition 2021 and `git diff --check` passed. Managed compilation and the
focused behavior test remain pending; this implementation did not run Cargo.
