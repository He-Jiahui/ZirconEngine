---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-25-ui-input-paint-style-static-candidates.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/706-accessibility-visibility-detached-scratch-reuse.md
  - docs/plans/astra/features/runtime/708-accessibility-relation-target-retain.md
  - docs/plans/astra/features/runtime/711-accessibility-filter-map-membership.md
implementation_files:
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/resolution.rs
---

# Accessibility Resolution Key-Scratch Reuse

The accessibility snapshot builder must mutate its indexed node map while resolving names and
descriptions, so each pass previously materialized an independent `Vec<UiNodeId>` key snapshot.
The resolution layer now receives one caller-owned key buffer, refreshes it at each mutating pass,
and reuses its capacity between name and description resolution. The child-filter pass retains its
existing published-node snapshot contract for compatibility, while its per-node output vector now
reserves the direct-child lower bound before recursive flattening.

This keeps key ordering and mutation semantics unchanged. It removes one large-vector allocation
from every normal accessibility extraction and avoids geometric growth for the common direct-child
case; the buffer is still rebuilt when the node set changes after hidden relation pruning.

## Evidence

- `resolution_node_id_scratch_reuses_capacity_between_passes` covers key count and retained
  capacity across successive resolution passes.
- The focused accessibility contract remains `4/4` and the batch source guards, Rustfmt, and
  scoped diff checks pass.
- The 709 implementation snapshot hashes were `extract.rs`
  `650FB042FB5C0CF03438DE33B942C816F3A7ACA67C875228627A92702DA9AD95` and
  `resolution.rs` `D3CD8F5EAFCBF22617E006EED28121F190876266923C24A772233C1C4EAD1E97`.
  Record 710 subsequently extends the resolver; the current source hashes are
  `extract.rs` `CF5EECB9F7F070FD2B1FD53069BCBEF8FAB4F77DCB43E36F3CCB96258E4B31ED` and
  `resolution.rs` `48B9667E86E153B9E27FA91A301A268F7F67D426F8FA845C081C7E411126FB0F`.

These are source/static and deterministic-contract results. Managed Cargo, Windows Release
allocation/time, and product accessibility latency evidence remain pending under the external
dirty-worktree admission gate; no product performance acceptance is claimed here.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A/78 | Reuse one node-ID scratch buffer across accessibility name/description resolution and reserve child-filter output lower bounds | implemented_pending_validation | Focused accessibility contract `4/4`, Rustfmt, diff, and source-hash guards pass; managed Rust and product measurements remain pending |
