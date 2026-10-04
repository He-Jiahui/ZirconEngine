---
title: Runtime81 ASCII caret navigation fast path
category: zircon_runtime
report_id: Runtime81-ascii-caret-navigation-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime81 ASCII caret navigation fast path

## Scope

`previous_grapheme_boundary` and `next_grapheme_boundary` are used by text
navigation. Both previously enumerated graphemes from the beginning of the
whole string, so a caret movement near the end of an ordinary ASCII field
repeated work proportional to the text prefix.

The two functions now return the neighboring byte boundary directly when the
needed bytes are ASCII. The previous direction checks the two bytes before
the offset, with an explicit CRLF case. The next direction checks the current
and following bytes, with the same CRLF case. Other positions retain Unicode
segmentation. UTF-8 offset clamping remains shared with the prior path.

## Correctness and performance boundary

- The behavior test compares both directions at every offset of all 16,384
  ASCII byte pairs against `unicode-segmentation`. Mixed Unicode cases cover
  Prepend, combining marks, emoji ZWJ, CRLF, and offsets inside UTF-8 scalars.
- The admitted ASCII path uses constant-time byte reads with no allocation;
  the retired path scanned graphemes from the beginning of the string.
- `RUNTIME81_ASCII_GRAPHEME_NAVIGATION_BENCH_V1` compares both directions
  against the exact retired iterator expressions at the midpoint of a
  1,792-byte ASCII field. Each direction uses five warmups, 31 alternating
  paired samples, and 2,048 calls per sample; it retains raw samples and
  nearest-rank P50/P95/P99 with workload and environment metadata. The managed
  Release P95 target is at least 50%
  below the retired P95, with exact output parity.

This is a caret-navigation helper gate. It does not establish the Runtime81
document/edit/render product p99, allocations, RSS, WGPU, or visual acceptance.

## Validation manifest

Batch with the other Runtime and Editor changes in one managed Windows lane:

1. Grouped Runtime check and lib tests, including the non-ignored grapheme
   parity tests.
2. Grouped Release markers including
   `ascii_grapheme_navigation_release_p95` for both directions.
3. Grouped formatting, diff, and record structure checks over the exact
   source and documentation paths.

No Cargo or Release timing has been run for this slice. Keep its completion
status pending until terminal managed evidence is recorded.
