---
doc_type: milestone-detail
status: candidate_static_review_complete_managed_validation_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-unchanged-derived-component-publication.md
implementation_files:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
tests:
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
---

# Runtime1023: Runtime62 unchanged derived-component publication

The original four-path candidate manifest remains unchanged at `docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-g18-source-manifest.json`. Successor hashes, including the newly scoped `dirty_state.rs`, are sealed in `docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-g18-render-publication-successor-source-manifest.json`.

Status: `candidate_static_review_complete_managed_validation_pending`.

- [x] Skip `replace_derived_component` when computed `ActiveInHierarchy` or `WorldMatrix` compares equal to the stored component.
- [x] Mark the entity render-dirty only when either derived value changes.
- [x] Remove the checked-reparent-only render mark; run `RenderExtractPrepare` when any derived frontier is pending so preparation propagates checked reparent changes before component publication.
- [x] Preserve unchecked targeted hierarchy marks and global/repair `mark_all` behavior.
- [x] Keep `equivalent_parent_reparent_preserves_derived_ticks_and_render_fields`, covering equal world transform/active state, unchanged component ticks, stable render-artifact `Arc` and journal generation, and retained child fields.
- [x] Add `checked_reparent_render_stage_publishes_changed_world_matrix` in `dirty_state.rs`; a checked reparent changes the real child world value and must update its published render projection during a direct RenderExtract stage.
- [x] Keep the render assertion scoped to component fields; do not require hierarchy/topology publication or unrelated render output to be empty.
- [x] Format the changed `dirty_state.rs` with pinned Rustfmt 1.94.1 / edition 2021; check all three candidate Rust paths and directly check final newlines and trailing whitespace.
- [x] Preserve the `projected_reads.rs` preimage after its writer returned `OSError [Errno 22] Invalid argument`; place the positive changed-value regression in the writable `dirty_state.rs` unit-test module instead. No file permission or lock workaround was attempted.
- [ ] Obtain grouped managed compilation and execute `scene::tests::derived_state::projected_reads` and `scene::world::dirty_state::tests`; no Cargo command was run for this candidate.
- [ ] Accept G18 only after the managed behavior test passes. This source candidate alone does not close RSH-G18.
- [ ] Establish performance baselines and budgets separately; this slice has no latency/allocation measurement or threshold and does not close RSH-G19.

Comparison uses the current `PartialEq` semantics. Regular authored transform writes validate finite components; behavior for externally corrupted non-finite transforms is outside this candidate's guarantee. The frozen Runtime wave manifest and its source hashes were left unchanged; this is a successor to its exact `derived_state.rs` and `projected_reads.rs` preimages. The prior G18 manifest SHA and its four prior postimage hashes, plus the `dirty_state.rs` preimage hash, are recorded in the optimize plan; the prior manifest itself remains unchanged. Coordinator registration request `aa81a44ed9cc48c0b9aa33c4b1bd4ec9` remained `accepted` with no terminal result on the single reconciliation, so lease/attribution success is pending and is not represented as a pass.
