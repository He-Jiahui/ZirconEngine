# Runtime81 Source-Line Shaping Work Budget

Plan: docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md","docs/plans/optimize/zircon_runtime/81/2026-08-21-source-line-shaping-work-budget.md","docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md","zircon_runtime/src/text/hard_line.rs","zircon_runtime/src/text/layout/line_break/mod.rs","zircon_runtime/src/text/layout/rich_advance_index/tests.rs","zircon_runtime/src/text/mod.rs","zircon_runtime/src/text/shaping/cosmic.rs","zircon_runtime/src/text/shaping/horizontal/direct.rs","zircon_runtime/src/text/shaping/mod.rs","zircon_runtime/src/text/shaping/tests.rs","zircon_runtime/src/text/shaping/vertical/direct.rs","zircon_runtime/src/text/shaping/work_budget.rs","zircon_runtime/src/ui/surface/render/text_prewarm/tests.rs","zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs","zircon_runtime/src/ui/text/layout_engine/tests/measure.rs","zircon_runtime/src/ui/text/layout_engine/tests/wrapping.rs","zircon_runtime/src/ui/text/layout_engine/wrapping/tests.rs"]

- Date: 2026-08-21
- Integration owner: `optimize-runtime73-runtime81-runtime89-batch-m3-r1-01a00797-20260822`
- Former owner: `optimize-runtime81-shaping-budget-m0-r1-01a00797-20260821`
  (`cancelled` after grouped transfer fingerprint
  `2b9dfa137351b0771e7c7f4fa03b571010756adbbf0c8a60bde3ca394807aa29`)
- Source plan: `docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md`, RTS-P0-001 / M0
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The old 64 KiB shaping cap was deleted from `hard_line.rs` because it promoted an
internal backend limit into source-line identity. Seven test files still referenced
that deleted constant 24 times, so the test target could not compile. Several test
expectations also still counted the synthetic line and therefore encoded the removed
behavior.

## Scope Delivered

- `TextShapingWorkBudget` preserves the 64 KiB value as a typed, non-zero execution-policy
  contract for scale fixtures and the future scheduler: requests at or below it are inline
  candidates and larger requests are classified as exceeding the inline threshold.
- The budget is not a line, script-run, glyph-cluster, or source-range boundary. The
  direct and fallback backends continue receiving complete semantic segments.
- M0 does not yet connect this contract to a production deferred worker. Production shaping
  remains synchronous and must retain complete semantic context until M3 adds a typed outcome.
- All seven affected test files now derive scale fixtures from the typed budget. The
  deleted `TEXT_SHAPING_RUN_MAX_BYTES` identifier has zero remaining Rust references.
- Hard-line, rich advance, wrapping, horizontal/vertical shaping, and prewarm tests
  require an over-budget unbroken request to remain one source line.
- UI prewarm expectations now count only source separators and real rich-style/inline
  boundaries.

## Deterministic Performance Evidence

| Workload | Old synthetic-boundary requests | M0 requests | Reduction |
|---|---:|---:|---:|
| rich advance: `first`, `second`, over-budget run | 4 | 3 | 25% |
| inline rich prewarm with one inline object | 5 | 4 | 20% |
| vertical prewarm with CRLF and U+2028 | 4 | 3 | 25% |

The joined rich-run layout also changes from two synthetic layout lines to one source
line while retaining its two style-specific prewarm requests. The three request-topology
tests share the `source_line_request_topology_evidence` filter and emit
`PERF-MVP-RTS-P0-001` rows with `budget_bytes`, legacy/optimized request counts, and
reduction basis points. The managed validator requires exact rows for `rich_advance`,
`rich_inline_prewarm`, and `vertical_prewarm`, with reductions of at least 25%, 20%, and
25% respectively.

The managed long semantic request timing probe records 31 P50/P95 samples per Latin,
CJK, RTL, ligature, and vertical-CJK workload. It is diagnostic evidence, not a release
threshold: comparing the former semantically incorrect split result as a faster baseline
would be invalid.

## Fresh Testing Evidence

- `TextShapingWorkBudget::new(0)` is rejected; the default boundary classifies exactly
  64 KiB as inline and 64 KiB + 1 as exceeding the inline threshold.
- One `source_line_request_topology_evidence` Cargo filter must run all three topology
  workloads and produce exact 4 -> 3, 5 -> 4, and 4 -> 3 request-count evidence.
- The managed child also locks exact summaries for the typed budget contract, all nine hard-line
  tests, 22 active semantic-shaping tests with two release probes ignored, and the three layout-engine
  measure/wrapping regressions through `unwrapped_` with the unrelated UI measure-cache test
  explicitly skipped. These groups cover 38 passing behavior tests while retaining the six-group
  Runtime81 batch contract.
- Old constant references: 0 across `zircon_runtime/src/**/*.rs`.
- Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Runtime81 behavior tests, the full Runtime regression batch, and the 31-sample timing
  probe: pending serialized coordinator validation. No Cargo result is claimed here.

## Review

Independent static review required exact-count coverage for the hard-line, ligature, measure, and
wrapping contracts. The validator now carries those gates, and follow-up review found no Critical,
Important, or Minor defect. Grouped Cargo validation remains pending; no behavior or timing pass is
claimed by this record.

## Remaining Scope

M0 repairs source-line semantics and exposes a temporary typed policy contract; it does not
claim that production scheduling changed. It does not yet implement the M3 typed
`ShapingOutcome`, cancellation, deadline, glyph/RSS budgets, deferred worker publication, or
partial recovery. Those features must preserve the same complete semantic context rather than
reintroduce fixed-byte shaping chunks.
