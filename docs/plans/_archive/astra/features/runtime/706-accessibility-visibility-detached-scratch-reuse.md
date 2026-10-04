---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/03-accessibility-indexed-focus.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
tests:
  - zircon_runtime/src/ui/accessibility/extract/visibility.rs
---

# Runtime Accessibility Detached-Visibility Scratch Reuse

`EffectiveHiddenIndex` previously allocated a path `Vec` and an ordered cycle
set for every detached component it resolved. A fragmented or malformed tree
could therefore repeat two temporary collection allocations even though the
resolver processes one component at a time. The resolver now owns one path
scratch and one `HashSet` scratch for the whole build, clears both before each
component, and retains the hash buckets across components. The published
`BTreeMap<UiNodeId, bool>` remains the ordering and visibility authority; cycle
handling, missing-parent fallback, deadline checks, and hidden-state semantics
are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime78 accessibility extraction | Reuse detached-node path and cycle-detection scratch across disconnected components. | Accessibility extraction contracts pass; a production test proves a visible cyclic component is not changed by a following collapsed component; scoped Rustfmt/diff and source guard pass. | implemented_pending_validation |

## Batched local evidence

- Accessibility extraction, indexed-focus, and accessibility payload/node-lookup
  source contracts pass (`6/6` in the focused batch; the extraction contract
  itself passes `4/4`).
- `detached_resolution_clears_reused_path_before_next_component` covers scratch
  clearing, cycle termination, and preservation of the earlier component's
  effective visibility.
- A source-bound guard confirms one production scratch declaration for each
  collection, `clear()` before every detached resolution, and no per-component
  `BTreeSet`/path allocation.
- Current source snapshot SHA-256:
  `9DD4229F4EF6A90560BC5E998C583494DB281829E007161F501F8C71BBADEBAE`.
- Scoped Rustfmt and `git diff --check` pass; the repository emits only its
  existing LF/CRLF line-ending notice.

This is source/contract evidence, not managed Cargo execution or product
accessibility CPU/allocation/RSS/latency p50/p95/p99 evidence.

## Managed acceptance gate

The owner-attributed Windows Release gate remains deferred by the external
`E:\\Git\\zr_vm` dirty-worktree admission recorded in `696`. No coordinator
state was polled in this slice. Keep this row at
`implemented_pending_validation` until the batched managed Runtime tests and
Release measurements validate the current source. Tooling changes remain
deferred by request.
