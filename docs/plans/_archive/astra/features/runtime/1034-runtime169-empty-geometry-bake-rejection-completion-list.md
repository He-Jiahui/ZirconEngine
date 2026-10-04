---
doc_type: feature-completion-list
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime169
related_plan:
  - docs/plans/optimize/zircon_runtime/169/2026-09-28-runtime169-empty-geometry-bake-rejection.md
  - docs/plans/optimize/zircon_runtime/169-runtime-navigation-current-working-tree-bake-artifact-query-crowd-editor-boundary-review.md
---

# Runtime1034: empty-source navigation bake correction

| Item | Implemented | Evidence | State |
|---|---|---|---|
| No-source bake result | Replace the synthetic surface-volume quad with an explicit empty `NavMeshAsset` and a warning that states no walkable polygons were generated. This is the explainable empty-artifact outcome permitted by P0-02; a typed failure or quarantine is not required for it. | `zircon_plugins/navigation/runtime/src/manager/bake/asset.rs`; source preimage `82d6012f…`, candidate `f8b4fdea…`. Geometry-backed Recast calls are unchanged. | candidate_static_review_complete_managed_validation_pending |
| Public behavior regression | Use `World::empty()` and exercise the public `NavigationManager::bake_surface` entry point. Assert zero source triangles, baked vertices/polygons/report tiles, empty report/snapshot geometry vectors, and a missing-source warning. | `bake_surface_without_source_geometry_publishes_no_walkable_mesh` in `zircon_plugins/navigation/runtime/src/tests/bake.rs`; preimage `f869e860…`, candidate `e09a20c5…`. | candidate_static_review_complete_managed_validation_pending |
| Test API compatibility | Handle all 21 `spawn_node` results in `tests/bake.rs` for the current fallible API: unwrap 19 assigned IDs and both intentionally unused results. Import `SceneNavigationRuntime` for the generated snapshot assertion. | Pinned rustfmt 1.94.1 edition 2021 check passes; managed compile remains pending. | candidate_static_review_complete_managed_validation_pending |
| Runtime169 P0-02 acceptance | The no-source fake-geometry defect now takes the plan-permitted explainable empty-artifact path. Keep broader P0-02 product acceptance open until off-mesh-link behavior and publication/query semantics for an empty asset are decided and tested. | The manager returns `Ok(report)` and publishes an empty result; no managed compile or runtime test has run. | open |
| Deferred cleanup | Retain the unused half-extent calculation in `manager/bake.rs` because an active foreign Session owns that path. | Exact-path audit found the file dirty and attributed at the same SHA to that Session; removing the call/helper is a follow-up after ownership handoff. | deferred |

- [x] Add the behavior regression before the production change, using an empty World that cannot contribute the default Cube.
- [x] Remove fake walkable polygons from the no-source bake branch.
- [x] Update the bake test module to handle every `spawn_node` result and import the snapshot trait.
- [x] Add the Runtime169 optimize record with exact source preimages/candidate hashes and acceptance limits.
- [ ] Run the focused plugin runtime test through managed grouped validation.
- [ ] Decide and test off-mesh-link publication/query behavior for the empty artifact before closing broader P0-02 product qualification.
