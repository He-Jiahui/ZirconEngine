---
title: Runtime81 ASCII grapheme boundary fast paths
category: zircon_runtime
report_id: Runtime81-ascii-grapheme-join-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime81 ASCII grapheme boundary fast paths

## Scope

The UI wrapping path calls `leading_grapheme_continuation_len` each time a
candidate line appends a text run. The existing helper concatenated the entire
accumulated line and new run, then segmented from the beginning to decide
whether the join crosses a grapheme boundary. This makes ordinary ASCII joins
allocate and rescan the retained line on every append.

The helper now checks the two bytes adjacent to the join first. If both are
ASCII, the only cross-boundary grapheme is CRLF, so it returns 1 for CRLF and 0
otherwise without copying or segmenting either string. Other joins retain the
existing Unicode segmentation path. Empty strings retain their existing result.

The same local boundary rule applies when a caret or IME offset is already at a
UTF-8 boundary. `clamp_grapheme_boundary` now returns that offset directly for
adjacent ASCII scalars, or the CR position for an interior CRLF offset. The
prior path scanned graphemes from the beginning of the text. Mixed and
non-ASCII boundaries still use the existing segmentation path. These changes
do not alter wrapping, caret, or source-range APIs.

## Correctness and performance boundary

- The behavior test compares all 16,384 ASCII byte pairs with
  `unicode-segmentation` for both join and caret-clamp results, then checks
  longer plain-text, mixed-Unicode, CRLF, combining-mark, and emoji-ZWJ joins.
- The ASCII path uses constant-time byte reads and makes zero allocations;
  the prior path allocated one combined string and scanned the accumulated
  line. The ASCII caret-clamp path also uses constant-time byte reads instead
  of scanning every preceding grapheme. Unicode cases retain their prior cost
  and behavior.
- `RUNTIME81_ASCII_GRAPHEME_JOIN_BENCH_V1` compares the legacy helper and the
  production helper on a 1,792-byte accumulated ASCII line. It uses five
  warmups, 31 alternating paired samples, 2,048 calls per sample, and exact
  output parity. Raw samples and nearest-rank P50/P95/P99 are retained with
  workload and environment metadata. The release P95 target is at least 50%
  below the legacy P95.
- `RUNTIME81_ASCII_GRAPHEME_BOUNDARY_BENCH_V1` compares the legacy full-prefix
  grapheme scan and the production caret clamp at the midpoint of a 1,792-byte
  ASCII line using the same sampling, raw evidence, and P95 target.

## Validation manifest

Batch with the other Runtime milestone changes in one managed Windows lane:

1. `cargo check -p zircon_runtime --lib` for the package batch.
2. `cargo test -p zircon_runtime --lib grapheme` for the focused behavior
   tests, grouped with other Runtime filters by the validator.
3. `cargo test --release -p zircon_runtime --lib ascii_grapheme_ -- --ignored --nocapture`
   for both release timing markers, grouped with the milestone's release evidence.
4. `rustfmt --check --edition 2021` on the two source files and
   `git diff --check` on the four changed paths.

No Cargo or release measurement was run during the implementation slice. The
constant-time and allocation claims follow from the code paths; both P95 targets
and product-scale performance remain pending terminal managed evidence.
