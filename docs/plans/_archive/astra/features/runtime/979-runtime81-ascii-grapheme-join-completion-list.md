---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/81/2026-09-26-ascii-grapheme-join-fast-path.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/text/grapheme.rs
tests:
  - zircon_runtime/src/ui/text/grapheme/tests.rs
---

# Runtime81 ASCII grapheme boundary completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| ASCII grapheme join | Adjacent ASCII joins return the Unicode-equivalent continuation length without copying or segmenting the accumulated line. CRLF and non-ASCII joins preserve their existing behavior. | Full 16,384-pair ASCII parity test, Unicode edge tests, managed Runtime compile/test batch, and `RUNTIME81_ASCII_GRAPHEME_JOIN_BENCH_V1` Release raw samples/P50/P95/P99 with P95 at least 50% below legacy. | implemented_pending_validation |
| ASCII caret boundary | Adjacent ASCII scalars return the exact Unicode-equivalent grapheme boundary without scanning the text prefix. An interior CRLF offset floors to CR; non-ASCII boundaries preserve their existing behavior. | The same pairwise parity and mixed-Unicode tests, plus `RUNTIME81_ASCII_GRAPHEME_BOUNDARY_BENCH_V1` Release raw samples/P50/P95/P99 with P95 at least 50% below legacy. | implemented_pending_validation |

The source and focused tests are ready for a grouped Runtime validation
manifest. No pending coordinator receipt or static source inspection is counted
as a passing Cargo or release-performance result.
