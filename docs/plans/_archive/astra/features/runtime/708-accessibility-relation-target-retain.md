---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/706-accessibility-visibility-detached-scratch-reuse.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
tests:
  - zircon_runtime/src/ui/accessibility/extract/resolution.rs
---

# Runtime Accessibility Relation-Target Retain

Accessibility extraction used to collect every unusable hidden relation target
into a temporary `Vec`, then perform a second pass to remove those targets from
the published node map. The relation-target set now validates and removes each
unusable target in one `BTreeSet::retain` pass. The referenced-text predicate,
target ordering, node removal, and hidden-target publication semantics remain
unchanged; the transient target-list allocation is eliminated.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime78 accessibility extraction | Prune unusable hidden relation targets in-place. | Accessibility extraction/indexed-focus/payload contracts pass (`6/6`); source guard confirms retain-based pruning and no temporary target list; scoped Rustfmt/diff pass. | implemented_pending_validation |

## Batched local evidence

- The focused accessibility contract batch passed `6/6`; the final Runtime,
  Editor, and Runtime Text performance batch passed `1870/1870`, and pressure
  suites passed `283/283`.
- `hidden_relation_targets.retain` keeps the one-pass removal boundary while
  preserving the existing `referenced_text` predicate and node map authority.
- Current source snapshot SHA-256:
  `53A1E628C2091364631CC32DE095E74BF430AA3126B92DC4FA2837E68B9281BA`.
- Scoped Rustfmt and `git diff --check` pass; only the repository's existing
  LF/CRLF notice is emitted.

This is source/contract evidence, not managed Cargo execution or product
accessibility CPU/allocation/RSS/latency p50/p95/p99 evidence.

## Managed acceptance gate

The owner-attributed Windows Release gate remains deferred by the external
`E:\\Git\\zr_vm` dirty-worktree admission recorded in `696`. No coordinator
state was polled in this slice. Keep this row at
`implemented_pending_validation` until batched managed Runtime tests and
Release measurements validate the current source. Tooling changes remain
deferred by request.
