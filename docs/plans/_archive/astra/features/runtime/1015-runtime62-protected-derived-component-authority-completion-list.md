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
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-protected-derived-component-authority.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/protected_authority.rs
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/src/scene/tests/render_dirty_journal/render_component_projection.rs
  - zircon_runtime/src/graphics/scene/render_scene/component_projector/tests.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs
---

# Runtime1015: protected derived-component authority

Status: `candidate_static_review_complete_managed_validation_pending`.

- [x] Save exact preimages and foreign diffs, obtain same-session path leases and plan maintenance authorizations, and record test-first production hashes.
- [x] Reject public generic World insert/get-mut/remove of `WorldMatrix` and `ActiveInHierarchy` while retaining the private derived-state writer and read-only access.
- [x] Reject both mutable QueryDataAccess forms and fail closed in lower World mutable fetches, direct/deferred BundleStaging, Commands apply diagnostics, and built-in/generic reflection removal.
- [x] Add real behavior regressions for all mapped ingress and valid internal rebuild; keep Q/R raw-cycle source and tests unchanged.
- [x] Update ECS/query/reflection and public runtime-interface reflection contracts, including the typed removal error.
- [x] Repair five preexisting scene/render test owners that directly spawned protected derived rows or prohibited the new type classifier; preserve real projection and replay assertions.
- [x] Complete independent source review, exact inverse, scoped static checks, source manifest, and attribution (`2026-09-27-astra-optimize-batch-s-runtime62-protected-derived-authority-root-review.json`; 25-path v4 reviewed, v5 plan status only).
- [ ] Obtain managed compilation and full relevant behavior-test acceptance after the unrelated admission blocker is reconciled; no pending receipt is a pass.
- [ ] Close RSH-P0-001 / G01 with protected authored-component authority and Q/R fixture migration in a separate complete slice.
- [ ] Close Runtime62 G05, wider G02-G04 graph/product gates, and managed Release performance evidence without inventing an absolute threshold.

The source candidate closes RSH-P0-002 only. Its public query and reflection error variants are additive. Direct Cargo, validation submission, compiler monitoring, tooling modification, commit, push, and external notification were outside this owner's authorization.
