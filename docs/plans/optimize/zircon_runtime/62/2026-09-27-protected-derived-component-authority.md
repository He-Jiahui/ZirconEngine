---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction/staging.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/query/query_access_error.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/active_in_hierarchy.rs
  - zircon_runtime/src/scene/reflect/derived/component_adapter.rs
  - zircon_runtime_interface/src/reflect/error.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - .codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-runtime62-protected-derived-authority-candidate.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/protected_authority.rs
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/src/scene/tests/render_dirty_journal/render_component_projection.rs
  - zircon_runtime/src/graphics/scene/render_scene/component_projector/tests.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs
---

# Runtime62 protected derived-component authority

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| Runtime1015 | Close the generic ECS and reflection write routes for World-owned derived components | `candidate_static_review_complete_managed_validation_pending` | 2026-09-27 | [Astra checklist](../../../astra/features/runtime/1015-runtime62-protected-derived-component-authority-completion-list.md); `2026-09-27-astra-optimize-batch-s-runtime62-protected-derived-authority-*` receipts |

## Contract and ingress

`WorldMatrix` and `ActiveInHierarchy` are outputs of World derived-state rebuilds. The public generic `World::insert` and `World::remove` return `ProtectedDerivedComponentMutation` before advancing change ticks or changing registration, storage, lifecycle, or generation. `World::get_mut` returns `None` without a mutation receipt; its existing `Option` signature cannot return a typed error. Read-only `get`, query, presence, and tick inspection remain available. The private `replace_derived_component` writer continues to publish rebuilt rows.

Both mutable query data forms (`&mut T`, `Mut<T>`) return `ProtectedDerivedComponentWrite` from `QueryState::try_new`; the `new` and `World::query` convenience routes panic through that error. The World mutable query accessors and `component_mut_with_ticks` also fail closed before ticks or mutation recorders for direct trait-fetch and previously bound location paths. Direct bundle spawn/insertion and deferred bundle insert/remove reject at staging; Commands still queue `()` and expose the typed `SceneError` through `apply_deferred` diagnostics. A failed structural segment publishes no unrelated staged component. Built-in and generic reflection adapters return `NonRemovableComponent` for a present derived row instead of reporting it missing.

The real tests cover generic insert/get-mut/remove, both query markers, convenience query construction, direct trait fetches with real bound locations, direct bundle spawn/insertion, deferred insert/remove/spawn and atomic segment behavior, reflection removal, and internal derived rebuild after valid authoring edits. No source-string assertion substitutes for these behavior checks. The test-first receipt proves the initial behavior tests were written while all production owners still matched their saved preimages; direct bound-location and generic reflection regressions were added before the final static freeze. No dynamic red run was authorized.

Five preexisting test owners were reconciled with the protected writer boundary: a legacy source guard now rejects real fixed-owner maps and validation dispatch while permitting the type classifier, and scene/render fixtures spawn authored `LocalTransform` and `ActiveSelf` then use the normal RenderExtract rebuild for `WorldMatrix` and `ActiveInHierarchy`. Existing projection masks, candidate tick-probe counts, geometry replay, and transform-only expectations remain asserted. The static red receipt records the old incompatibility; runtime execution is pending managed validation.

The current public contracts are maintained in `docs/crates/zircon_runtime/scene/ecs.md`, `docs/crates/zircon_runtime/scene/ecs/query_state.md`, `docs/crates/zircon_runtime/scene/reflect.md`, and `docs/crates/zircon_runtime_interface/reflect.md`; the interface page documents the new serializable removal error.

Independent static review passed for the 25-path v4 source candidate (`2026-09-27-astra-optimize-batch-s-runtime62-protected-derived-authority-root-review.json`). The v5 successor updates only the plan review status; reviewed Rust and test bytes remain unchanged. Managed compilation, behavior tests, and performance evidence are still pending.

## Open Runtime62 gates

- RSH-P0-002 is closed by the source candidate across the mapped generic, query, bundle, Commands, and reflection ingress. Managed compilation and behavior tests are pending, so acceptance remains open.
- RSH-P0-001 and G01 remain open: generic `Hierarchy`, `LocalTransform`, and `Mobility` authoring is still publicly mutable. A complete hard cut requires Scene-authority callers and Q/R raw-cycle fixtures to move together to a test-only corruption hook; this slice deliberately preserves their predecessor bytes and diagnostics.
- G05 and the performance gate remain open. The type check is constant-time, but no managed Release measurements, render-product integration result, or absolute threshold is claimed. The wider G02-G04 scene graph and derived-product gates remain separate Runtime62 work.

No Cargo, managed validation submission, compiler monitoring, tooling change, commit, push, or external notification was performed for this source candidate. The earlier Batch R admission rejected before Cargo because of unrelated `zr_vm` drift; it provides no dynamic pass for Runtime1015.
