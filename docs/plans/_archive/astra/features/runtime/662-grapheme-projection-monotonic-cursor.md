---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81/2026-08-27-paragraph-analysis-construction-profile.md
related_code:
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/measure/grapheme_projection.rs
tests:
  - zircon_runtime/src/text/layout/measure/grapheme_projection.rs
  - tools/tests/test_runtime_text_grapheme_projection_cursor_contract.py
---

# Runtime Text Grapheme Projection Monotonic Cursor

## Scope

The common LTR grapheme projection path previously performed two fresh binary searches over the
paragraph grapheme ranges for every shaped cluster. This slice keeps the existing source-range
validation, overlap math, cluster receipt, and visual-order behavior while reusing monotonic
boundary cursors. Reordered/RTL clusters automatically use the original binary-search behavior.

## Implementation

`GraphemeProjectionCursor` advances the first-overlap and after-last boundaries when cluster source
ranges are monotonic. A non-monotonic range resets both cursors through the exact `partition_point`
queries, so visual reordering and mixed backend payloads remain fail-compatible. Source validation
was moved beside the projection owner to keep the measurement root within its file-budget contract.

When a shaped cluster's endpoints match the caller-owned grapheme boundaries, its scalar span is now
read directly from the already-known index interval instead of rescanning the source substring with
Unicode segmentation. Partial-boundary and malformed ranges retain the existing counting fallback,
so rich subranges and cross-style clusters do not receive an unproven shortcut. The geometry-only
measurement entry instantiates tab tracking off, while width-capable measurements reuse this same
pass to publish tab presence. This keeps the tab probe out of callers that only need geometry.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime662 | Reuse monotonic grapheme overlap boundaries, direct aligned-span counts, and an exact reordered fallback | implemented_pending_validation | Focused cursor/source contract `5/5` (the tab-presence call-shape guard now tolerates rustfmt generic-argument wrapping), merged Runtime/Editor source batch `125/125`, scoped Rustfmt, and diff checks pass. Managed Runtime Cargo plus release CPU/allocation/RSS p50/p95/p99 remain pending. |

## Acceptance boundary

This is a source-level hot-path optimization. It does not claim that all paragraph projections are
monotonic, does not change cluster caret policy, and does not promote Runtime11b/81 or the product
performance gates. The managed batch must verify LTR, RTL, rich, and cross-run projection parity
before any performance-qualified status is recorded.
