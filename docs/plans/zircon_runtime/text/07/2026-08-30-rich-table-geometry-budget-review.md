# Rich table intrinsic geometry budget review

Date: 2026-08-30

Status: `RRT-P1-033_geometry_budget_and_table_cutover_static_complete /
managed_compile_render_and_profile_pending`

## Scope and current-source correction

This record covers the remaining rich-table geometry defect. It does not reopen the 2026-08-24
parent repair: general rich intrinsic measurement already uses Text03's unbounded-main-axis contract
instead of a UTF-8-byte-count square frame. The remaining owner is
`ui/text/layout_engine/rich_table/layout.rs`.

The pre-cutover table path performed two layouts per cell. Its preferred-column pass disabled wrapping but
uses a `f32::MAX / 4` square frame for horizontal text. The final cell pass uses the resolved inline
track and estimates the provisional block extent as visible source bytes multiplied by the larger of
line height and font size, again capped at `f32::MAX / 4`. Vertical preferred measurement uses the
same byte-derived estimate on both axes.

That design has four structural faults:

- a near-maximum float is not a work or geometry budget;
- source bytes are neither glyph advance nor wrapped line/column count;
- alignment, clipping, prefix sums, cell boxes, and renderer artifacts observe the fake frame before
  the measured result is translated into the final cell;
- non-finite values are sanitized to zero in parts of track sizing, which can turn overflow into valid-
  looking but incorrect geometry instead of a typed failure.

Parser representation limits and the current 64 KiB shaping scheduling threshold cannot own this
policy. They have byte/count units and do not constrain logical-pixel extents or font size.

## Unreal reference and target ownership

Local Unreal Slate keeps intrinsic text measurement separate from allotted widget geometry.
`FSlateTextBlockLayout::ComputeDesiredSize` sets a wrapping width and asks `FTextLayout` to update its
line model; a wrapping width of zero is the explicit no-wrap mode. `FTextLayout` accumulates actual
block widths into `TextLayoutSize.DrawWidth`/`WrappedWidth`, while view size and final justification
remain separate. Parent panels consume cached child desired sizes and later arrange them inside their
allotted geometry. The reviewed path does not require an `FLT_MAX` fake widget to discover preferred
text width.

Zircon should retain the same separation:

1. A runtime/session-owned `TextLayoutGeometryBudget` defines admitted logical inline/block extents
   and accumulated document/table geometry. It is not a parser constant or renderer clamp.
2. Intrinsic requests carry a typed bounded/unbounded constraint. While Text03 still represents
   unbounded main axes as positive infinity internally, infinity is request metadata and may never be
   published as measured geometry.
3. Preferred table measurement uses no-wrap shaping/layout and the measured glyph/inline-object
   result directly. It does not position content in a near-maximum frame.
4. Final cell layout uses resolved inline tracks and an unbounded block measurement constraint. A
   vertical intrinsic owner must support an unbounded cross axis without converting it to zero or
   inventing a byte-derived column capacity.
5. Every measured line, cell, prefix sum, span, row/column total, and translated frame is checked
   before publication. Non-finite or over-budget output returns a dedicated `GeometryTooLarge`
   outcome; it is never silently clamped or sanitized to zero.
6. The session records the rejecting owner, requested/admitted extent, cell/table identity, and work
   consumed. Renderer, hit-test, spatial index, and cache consume only admitted finite artifacts.

## Delivered infrastructure and production cutover

`TextLayoutError::GeometryTooLarge` now reserves diagnostic code `ZR-TEXT-LAYOUT-013` and catalog
key `text.layout.geometry_too_large`. The enum's exhaustive uniqueness test and the Runtime Text
static contract include the new variant.

The 2026-08-30 non-acceptance implementation now adds `TextLayoutGeometryBudget` as an immutable
`SharedTextLayoutSession` snapshot. Its default `16,777,216` logical-pixel ceiling is the `2^24`
exact-integer boundary of `f32`; it is a hard numeric safety ceiling, not the lower product policy
that still requires the documented viewport/DPI/font/table corpus. Checked axis, coordinate,
accumulated-add, and accumulated-scale operations return a typed violation without clamping.

`TextLayoutAxisConstraint` now distinguishes bounded requests from positive-infinity unbounded
metadata. Shared rich/VerticalRl intrinsic measurement and rich-table preferred/final cell passes
use that protocol. The old `f32::MAX / 4` and source-byte-times-line-advance frames are removed.
Unbounded table width retains natural column extents instead of being sanitized to zero or expanded
to infinity.

`layout_engine/geometry_admission.rs` is the neutral resolved-DTO publication owner. Rich/VerticalRl
layouts, plain backend sizes, fixed-line-height accumulation, source-range width, and table layouts
all consume the same session budget; table modules only add table-specific rejecting context.

Column/row solvers and `TrackMetrics` now return checked results. Preferred layouts, final cell
layouts, line/placement frames, glyph advances, boxes, translated output, row/column prefix totals,
and whole-table block accumulation are admitted before publication. Rejection records the owner,
source range, attempted/admitted extent, and work units in the session and returns
`GeometryTooLarge`; fallback reporting owns a separate geometry counter.

Failing-first static contracts cover the session owner, typed unbounded measurement, checked track
sizing, and publication gate. The infrastructure contract module passes 31/31; focused Rust tests
for budget arithmetic, report/reset behavior, unbounded sizing, and track metrics are written but
not run because the managed Cargo acquisition chain remains blocked. No WGPU/PNG or product
performance claim is made.

## Dependency-ordered implementation slices

1. **Static implementation complete:** typed geometry budget, session snapshot, checked arithmetic,
   rejection receipt, and hard numeric ceiling. A lower product policy remains evidence-gated.
2. **Static implementation complete:** shared horizontal and VerticalRl bounded/unbounded intrinsic
   request protocol with finite publication checks.
3. **Static implementation complete:** table preferred/provisional byte-derived frames removed;
   preferred and final metrics remain distinct.
4. **Static implementation complete:** table track prefix sums, layout frames, boxes, translations,
   and final aggregates fail closed. Managed deterministic behavior validation remains open.
5. **Pending after correctness acceptance:** profile whether retained intrinsic metrics remove the
   second full cell layout. That optimization belongs to RRT-P1-032, not this numeric-safety repair.

## Required performance and product evidence

Before an optimization implementation, capture the current algorithm on an E-drive release target.
Use 31 isolated samples for 1, 16, 256, 4,096, and admitted-maximum cells, with short/long Latin,
CJK, combining, RTL, nested table, inline object, horizontal, and VerticalRl lanes. Record p50/p95/
p99 wall time, shape/backend calls, cell layouts, glyphs, allocations/bytes, RSS, maximum logical
extent, overflow/rejection reason, WGPU timestamps, and package power. Compare the same corpus after
each slice and against Unreal's desired-size/arrange behavior.

Real framebuffer evidence belongs under `docs/tests/runtime/text`, never under `target`. The complete
Runtime Text static suite passes 50/50; no managed Cargo, WGPU, PNG, RSS, power, or matched Unreal
timing was run for this review, so RRT-P1-033 remains open.
