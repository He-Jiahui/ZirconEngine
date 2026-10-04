---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/81/2026-09-26-ascii-caret-navigation-fast-path.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/text/grapheme.rs
tests:
  - zircon_runtime/src/ui/text/grapheme/tests.rs
---

# Runtime81 ASCII caret navigation completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Previous grapheme | Directly locates the previous ASCII boundary, preserving CRLF and falling back to Unicode segmentation elsewhere. | All ASCII byte pairs and mixed Unicode parity, grouped Runtime compile/tests, Release `RUNTIME81_ASCII_GRAPHEME_NAVIGATION_BENCH_V1` previous-direction raw samples/P50/P95/P99 with P95 at least 50% below legacy. | implemented_pending_validation |
| Next grapheme | Directly locates the next ASCII boundary, preserving CRLF and falling back to Unicode segmentation elsewhere. | The same parity and grouped gates, with next-direction Release raw samples/P50/P95/P99 and P95 at least 50% below legacy. | implemented_pending_validation |

Product-scale text edit, render, allocation, RSS, and p99 acceptance remains
open. Static checks and an asynchronous request are not passing evidence.
