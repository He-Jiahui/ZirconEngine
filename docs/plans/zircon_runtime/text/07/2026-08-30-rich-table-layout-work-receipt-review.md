# Rich table layout work receipt review

Date: 2026-08-30

Status: `RRT-P1-038_table_layout_work_receipt_static_complete /
managed_profile_and_budget_decision_pending`

## Current-source correction

The original RRT-P1-038 finding says table/cell/token counts have no document-level bound. That is
no longer accurate for parsing. `RichParseBudget` currently owns request-local token, nesting,
table, cell, run, paragraph, projection-index, source-byte, and visible-output limits. BBCode table
state admits every cell against the shared default 65,536-cell request limit before materializing a
compiled artifact.

The remaining gap is layout work visibility. A table cell currently runs one no-wrap preferred
layout and one final layout at the resolved inline extent. The session reports shaping/cache work,
but it cannot state how many tables, cells, source bytes, tracks, lines, or boxes caused that work.
Without that receipt, a new execution limit, retained intrinsic cache, or partial-layout policy would
be an unmeasured behavior change.

## Reference boundary

Local Unreal Slate keeps `FLineModel` and `FLineView` as retained layout work units. Its wrapping
information has an explicit dirty/cache lifecycle, and `BeginLayout`/`BeginLineLayout` plus their end
hooks bracket regeneration. `FSlateTextBlockLayout::ComputeDesiredSize` updates that retained model
and then consumes `FTextLayout::GetSize`; source bytes are not treated as a substitute for generated
line/view work.

Zircon should preserve the same ownership direction:

1. Parser representation limits continue to protect construction and indexed artifact size.
2. `SharedTextLayoutSession` owns a frame-scoped `TextTableLayoutWorkReport`, alongside shaping and
   geometry reports.
3. Rich-table layout records actual preferred/final cell attempts, input bytes, table topology,
   resolved track counts, and published line/box counts with saturating arithmetic.
4. The report is telemetry only. It cannot skip cells, split a table, defer work, or authorize a
   retained intrinsic cache.
5. Profiling publishes only fixed-name aggregate counters at the session frame boundary. It emits no
   per-table/cell label or source text.

## Receipt fields

- table layout attempts, source bytes, total/max cells;
- preferred-cell layout attempts and input bytes;
- final-cell layout attempts and input bytes;
- resolved column and row track counts;
- published line and box counts.

Counts describe attempted work even when a later phase fails. Output counts are recorded only after
finite geometry admission. This makes rejected geometry and completed output distinguishable without
adding a second error policy.

## Implementation and validation order

1. Add the frame-scoped report and reset/profile lifecycle to `SharedTextLayoutSession`.
2. Instrument `rich_table/layout.rs` at the four semantic boundaries without changing layout order.
3. Add focused unit/static coverage for exact two-pass counts, frame reset, and fixed profile names.
4. Run the existing 1/16/256/4,096/admitted-maximum 31-sample E-drive matrix before proposing a
   layout execution threshold or RRT-P1-032 retained intrinsic cache.

No Cargo, timing, allocation, RSS, power, WGPU, or PNG evidence is claimed by this review. Real
framebuffer evidence remains under `docs/tests/runtime/text`, never under `target`.

## Implemented static slice

- `text/layout_session/table_work.rs` owns the saturating, content-free report and twelve fixed-name
  profile counters. `SharedTextLayoutSession` resets it at `begin_frame` and publishes it only at
  `finish_frame`.
- Rich-table layout records a table after checked source-range admission, each preferred/final cell
  immediately before the real layout call, tracks only after both track metrics resolve, and output
  only after aggregate geometry admission. Failed work remains visible without being counted as
  published output.
- Owner-local/reset and saturation Rust tests are written. The failing-first static contract and the
  complete reproducible Runtime Text static suite pass 52/52; targeted Rustfmt passes.

Managed Rust execution, the E-drive 31-sample profile matrix, allocation/RSS/power, product threshold
selection, retained intrinsic-cache decisions, real WGPU rendering, and PNG evidence remain open.
The implementation changes telemetry only; layout order, admission behavior, and output are unchanged.
