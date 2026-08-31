---
title: Runtime81 Invalid Resolved Advance Fail-Closed Geometry
category: zircon_runtime
report_id: Runtime81-invalid-resolved-advance-fail-closed-2026-08-27
date: 2026-08-27
session_id: root-runtime81-invalid-resolved-advance-fail-closed-20260827
implementation_status: implementation_complete
validation_status: static_validation_complete_managed_pending
runtime_follow_up_status: implementation_complete_static_validation_complete
---

# Runtime81 Invalid Resolved Advance Fail-Closed Geometry

## Finding

`UiResolvedTextLine` defines `glyph_advances` as exactly one physical advance per visual grapheme.
`UiTextLineSourceMap::advance_to_visual_offset` nevertheless treated a cardinality mismatch as a
valid geometry source and divided `measured_width` uniformly across all visual graphemes. That
creates plausible but unowned interior caret, selection, and IME positions from an invalid or
legacy DTO.

The runtime hit-test owner can recover missing advances by shaping with the command text and style.
The neutral interface owner cannot: it has no font database, shaping session, resolved face, or
backend cluster artifact. A proportional reconstruction in that layer therefore violates the same
single-authority and atomic-cluster rules used by canonical text geometry.

## Reference Review

Unreal keeps line-breaking and shaped measurement behind owned break-iterator and shaped-sequence
services. Its `IBreakIterator` accepts a borrowed source and exposes candidate navigation, while the
Slate shaping owner retains the sequence/range evidence used for physical geometry. This supports a
strict adapter boundary: a DTO without complete advances is unavailable geometry, not permission
for the interface layer to invent per-cluster metrics.

## Implemented Hard Cut

1. Admit the exact prefix-advance path only when cluster and advance cardinality match and every
   advance is finite and non-negative.
2. On invalid advance data, retain only the two physically known line boundaries: the leading edge
   is `0`, and the trailing edge is sanitized `measured_width`.
3. Collapse every invalid interior query to the leading edge. This is intentionally conservative;
   runtime consumers with source text/style may still recover exact geometry through shaping.
4. Replace the regression that required proportional interpolation with endpoint-only behavior and
   cover non-finite/negative data under the same contract.

## Complexity And Performance Gate

Valid layout remains the existing bounded linear-prefix path with lazy prefix caching. Invalid DTOs
become O(1) instead of scanning clusters and dividing width. This slice is a correctness hard cut,
not a measured performance optimization; no timing, power, or engine-comparison claim is made.

## Acceptance

- no proportional width division remains in invalid resolved-line geometry;
- valid per-grapheme advances retain exact prefix behavior;
- mismatched, negative, and non-finite advances expose only line endpoints;
- Rust formatting, scoped diff checks, source-contract scans, and file-size checks pass;
- Cargo and rendering validation remain pending the managed Windows lane.

The implementation and both invalid-state regressions are present. Rust 2024 formatting, scoped
diff checks, source-contract scans, and the 255-line owner budget pass. Cargo, corpus, profile,
power, WGPU, and PNG validation were not run in this non-validation slice.

## Runtime Hit-Test Follow-Up

Current-source review found a second invalid-state bypass in runtime hit testing. After artifact
miss, a cardinality-invalid DTO was reshaped with a style reconstructed from layout-level defaults.
That loses rich run style, font identity, justification/tab decisions, and virtual/BiDi context.

The follow-up keeps the route order explicit: canonical artifact; exact source-congruent
`SourceLineGeometry`; valid DTO advances; endpoint-only invalid DTO fallback. The source index will
return ordinary grapheme midpoints as well as atomic-cluster endpoints, so the source-aware path does
not shape and then reopen a second DTO geometry policy. The invalid no-source path selects the
nearest known line endpoint by aggregate midpoint and owns no shaping session.

The follow-up is implemented. `GraphemeAdvanceIndex::ltr_caret_hit` now returns either an ordinary
grapheme midpoint result or an atomic backend-cluster endpoint. Runtime fallback accepts DTO advances
only when cardinality and finite/non-negative invariants hold; otherwise it selects one of the two
line endpoints by aggregate midpoint. The old temporary `SharedTextLayoutSession`, reconstructed
default style, and fallback advance allocation are removed. The valid tab/BiDi/vertical DTO route and
canonical artifact route are unchanged.

The follow-up regressions and source guards are present. Rust 2024 formatting and scoped diff checks
pass; removed runtime fallback symbols have zero matches, and the largest touched production owner is
719 lines. Managed Cargo and visual/performance qualification remain pending.
