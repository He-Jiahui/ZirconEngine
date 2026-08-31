# Runtime Text physical-line content/placement geometry review

Date: 2026-08-29

Status: `intrinsic_and_paint_width_call_graph_reviewed / current_overflow_alignment_matches_unreal /
content_and_placement_geometry_separated / four_layout_routes_and_interaction_consumers_migrated /
static_checks_complete / managed_profile_power_wgpu_png_pending`

## Reviewed call graph

`resolve_line_widths_with_provider` returns natural measured width, canonical visual advances and a
frame-clamped alignment extent. The former `UiResolvedTextLine.frame` mixed the aligned content
origin, clamped paragraph slot extent and line hit candidate. Caret, selection, IME and renderer
projection needed natural content coordinates, while clipped line admission and nearest-line
selection needed the paragraph or rich-cell slot. Rich links additionally must not activate from
empty aligned slot space.

The initial risk hypothesis was that center/right `nowrap + Clip` might align an overwide natural
line using the clamped extent and therefore place the ink at the wrong origin. The Unreal comparison
does not support changing the algorithm: `FTextLayout::GetLineViewHorizontalDisplayOffset` uses
`max(DrawWidth, ViewSize)` as the justification width. When the line itself is wider than the view,
the extra justification space is zero, so its display origin also remains at the viewport start.
Zircon's clamped placement width produces the same origin, while natural advances overflow into the
existing clip.

## Structural correction

Unreal retains natural `FLineView.Size`, content-space offsets and justification width separately,
then converts display input back to content space for block hit testing. Zircon now follows the same
ownership split without copying Unreal's DTO shape directly:

- `frame` is absolute natural content geometry. Alignment changes its origin, never its main-axis
  extent.
- required `placement_frame` is the absolute paragraph or rich-cell slot used for clipped admission
  and physical-line selection.
- `hit_frame()` exposes the content candidate, so a slot gap may choose the nearest caret but cannot
  activate a rich link.
- Plain/rich horizontal and `VerticalRl` producers publish both frames; rich-table projection uses
  `translate()` to move them atomically.

The hard serde cut intentionally rejects an old layout payload without `placement_frame`; a default
would silently restore the ambiguous contract. Focused contracts cover distinct-frame roundtrip and
atomic translation, natural overwide main-axis extent, horizontal right-aligned slot-gap caret
selection, and rich-link exclusion. All 35 current files containing line literals were scanned: 90
actual literals, zero missing fields. Rust 2024 formatting for the touched owners, scoped diff check,
conflict scan and 208-678-line owner budgets pass. Managed Cargo and product rendering remain open.

## Performance and product validation gate

This is a correctness migration, not a performance optimization. It adds one 16-byte `UiFrame` per
published physical line and adds no shaping, wrapping, allocation or search loop. The managed paired
baseline must therefore measure both regression risk and any later optimization opportunity before
another algorithm changes:

- use identical source, font collection generation, backend, viewport, clip, render mode and warmup
  for parent and candidate;
- cover 1/100/1k/10k physical lines across horizontal left/center/right nowrap short/overwide,
  wrapped text, `VerticalRl`, rich links and rich tables;
- record 31-sample CPU p50/p95/p99 for cold layout, warm retained layout, clipped scroll and pointer
  hit, plus allocation count/bytes, peak RSS and shaping/cache counters;
- record Native/SDF GPU timestamps, glyph upload bytes and a separate same-scene power trace, then
  inspect a real product framebuffer under `docs/tests/runtime/text` rather than a strategy image.

The existing line publication and hit-selection asymptotic bounds are unchanged. Any measured
line-selection or memory bottleneck must be reported against this matrix before an index, arena or
cache is admitted. No performance, power, optimal-scale or matched-Unreal claim is made yet.
