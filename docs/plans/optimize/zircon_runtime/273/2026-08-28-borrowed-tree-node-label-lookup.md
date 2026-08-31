# Runtime273 Borrowed Tree Node Label Lookup

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime273-editor219-performance-batch-20260828ia-v1`

## Problem

Beginning a Runtime tree rename materialized the ordered node-ID vector, cloned the selected ID, and
then recursively cloned every visited node identity solely to compare it with that ID. Deep tree
views therefore allocated once per scanned node before the target label was found.

## Optimization

- Move the selected ID out of the already-owned ordered-ID vector.
- Return borrowed identities and labels throughout recursive tree lookup.
- Allocate only the two strings required by the final editing state: selected ID and editing text.
- Preserve property precedence, recursive child traversal, focused-index behavior, and label fallback.

## Regression Contract

The `optimization_batch_20260828ia_` Runtime tests prove identity and label allocation identity and
prevent selected-ID and recursive comparison clones from returning. The ignored paired release
benchmark emits `RUNTIME273_BORROWED_TREE_NODE_LABEL_LOOKUP_BENCH_V1`. It scans 2,048 nodes with
4-KiB identity and label fields eight times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
