# Runtime Text retained grapheme authority wiring

Date: 2026-08-29

Status: `architecture_review_complete /
retained_grapheme_authority_product_wired_unvalidated /
snapshot_admission_accounted /
fixed_profile_receipts_implemented /
algorithm_unchanged / managed_profile_pending`

## Structural finding

The retained `TextDocumentSourceIndex` already owned revision-qualified Unicode grapheme boundaries,
but no product input path consumed it. Every max-length keyboard, text, IME, or accessibility edit
rescanned the source prefix and suffix in `text_constraints.rs`. That made the sanitizer a second
source-analysis authority and kept the production document index outside the workload that must
justify its eventual incremental repair design.

The current index still has an important known cost: after a changed revision it obtains a contiguous
snapshot and rebuilds the complete boundary vector. Replacing that algorithm before product wiring
would optimize a synthetic path and obscure snapshot, allocation, and Surface projection costs. This
slice therefore changes ownership and observability only.

## Implemented boundary

- `TextDocument::retained_grapheme_count` validates UTF-8 range bounds, requires exact grapheme
  boundaries, and computes `total - replaced` with two binary searches over the revision index.
- `TextDocumentStore` checks document revision and current-snapshot budget before an index-triggered
  flatten, then accounts the materialized snapshot in the existing residency receipt.
- `UiTextDocumentSession` additionally fences the query by tree, node, and source epoch.
- keyboard text/newline, text events, IME preedit/commit, and accessibility selected replacement use
  that authority. Whole-value accessibility replacement uses the exact retained count zero. A
  selected accessibility edit with active transient composition uses the visible-source fallback
  because the accessibility range is not a committed-document coordinate.
- the sanitizer's explicit source fallback remains for no-session/tests and query rejection. It does
  not retain another index or cache and records its actual scanned prefix-plus-suffix byte count.

Warm indexed constraint counting is `O(log G)` for `G` stored grapheme boundaries. A cold invalidated
query remains `O(N + G)` time and `O(N + G)` retained materialization for source bytes `N`; this slice
does not claim that cost is acceptable. Replacement filtering/truncation remains `O(R)` for admitted
replacement bytes/graphemes `R`.

## Measurement contract

The fixed, content-free counters are:

- `text_document_grapheme_query_count`, `text_document_grapheme_query_nanos`;
- `text_document_grapheme_binary_search_count`, `text_document_grapheme_index_hit_count`;
- `text_document_grapheme_index_rebuild_count`, `_input_bytes`, `_boundary_count`, `_nanos`;
- `text_input_grapheme_document_index_count`, `_source_scan_count`, `_source_scan_bytes`.

Timing begins only during an active profiling capture. Names contain no document/node/range/source
identity and no raw text. The managed matrix must cover 1/100/1k/10k edits for ASCII, CJK,
combining-mark and ZWJ emoji sources; tail, middle, selection, IME preedit update/commit; cold/warm
index state; and constrained/unconstrained controls. Record 31 samples of p50/p95/p99, allocation
count/bytes, RSS, fixed counters, fallback rate, and valid power. Compare the same hardware, font,
DPI, source, event sequence, warm-up, and build profile with the selected Unreal editable-text path.

## Current evidence and open work

Rust formatting, parser-level syntax, scoped whitespace checks, exact-boundary tests, stale-revision
and stale-epoch tests, pre-flatten admission tests, and fixed-name tests are present. No managed Cargo,
product workload, allocator/RSS sample, power capture, Unreal comparison, WGPU framebuffer, or PNG was
produced in this non-rendering slice. Consequently no latency, allocation, power, or bottleneck-removal
claim is made.

Incremental boundary repair, compact offset representation, index memory admission, and Surface
full-value projection remain separate evidence-gated decisions. Do not add a sanitizer-local cache or
change the rebuild algorithm until the managed product matrix attributes the dominant cost.
