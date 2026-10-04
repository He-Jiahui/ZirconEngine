---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-25-ui-input-paint-style-static-candidates.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-ui-architecture-audit.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
related_code:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/v2/style.rs
tests:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime/runtime_pseudo_state.rs
---

# Runtime Single-Node Pseudo-State Fast Path

## Scope

The pointer-component state invalidation entry point previously wrapped every
single node in a one-element `BTreeSet` and sent it through the batch path.
That path then rebuilt descendant membership and minimized roots before applying
the style update. The single-node entry point now checks the existing borrowed
`node_state_can_affect_descendants` index directly: nodes whose state can match
descendant selectors retain the subtree style-application path, while ordinary
nodes apply runtime state style only to themselves. Both branches preserve the
explicit render-dirty mark and the existing error propagation.

This removes the per-event singleton ordered-set, descendant-set, root-vector,
and root-membership work from the common node-only case. It does not change the
batch entry point, descendant-selector semantics, style matching, or dirty-domain
ownership. The fast path is intentionally local to the pointer-component
invalidation boundary; tooling remains out of scope for this slice.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor01 / RUI-672 | Route single-node pseudo-state invalidation through a borrowed descendant-selector gate and direct node/subtree style application | implemented_pending_validation | Source regression requires the gate, both style branches, and the render-dirty mark while rejecting the singleton `BTreeSet` wrapper. Scoped Rustfmt, scoped diff check, direct source probe, and the combined 95-test Runtime/Editor static-contract batch passed. Managed Runtime Cargo and Windows Release allocation/time evidence remain pending. |

## Static Evidence

- `single_component_state_dirty_path_avoids_the_batch_set_wrapper` binds the
  single-node function to the borrowed descendant gate, both semantically
  required style-application branches, and `mark_node_dirty`; it rejects
  `BTreeSet::from([node_id])` in that function.
- Existing Runtime pseudo-state behavior tests continue to cover deep
  descendant restyling and ordinary node state transitions. They remain staged
  for the managed Rust test batch rather than being claimed as locally run.
- `rustfmt --edition 2021 --check --config skip_children=true` passed for the
  changed source file. Scoped `git diff --check` passed; Git reported only the
  repository's existing line-ending notice.
- The direct single-node source probe passed all five invariants.
- The combined Runtime/Editor static-contract batch passed `95/95` in `0.166s`.
  It covers navigation, incremental rebuild, render-cache authority, and
  Editor asset-browser, console, and watcher contracts.

## Complexity Boundary

For a node-only state event, the old path performed ordered-set construction
and batch root reduction before touching style state. The common path now does
one borrowed descendant-selector lookup, one node style application, and one
render-dirty update. A node with descendant pseudo-state selectors still pays
the existing subtree traversal and therefore retains the prior semantic cost.

## Source Snapshot

- `state_invalidation.rs`: `26783340D6D0A00BBD4B421999BCC90F7CAAA5A1C867FD309F2FBD3413813E54`

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task Runtime/Editor validation input must compile the
focused pseudo-state test module together with the existing Runtime/Editor
batch and collect a release allocation/time comparison. Until that batch
succeeds, this record remains `implemented_pending_validation` and makes no
product performance or p50/p95/p99 claim.
