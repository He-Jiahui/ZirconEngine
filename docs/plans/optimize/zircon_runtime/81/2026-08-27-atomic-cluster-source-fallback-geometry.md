# Runtime Text atomic-cluster source fallback geometry

## Status

`atomic_cluster_ltr_source_fallback_geometry_implemented /
source_aware_editable_pointer_route_implemented /
canonical_artifact_fast_path_preserved / static_checks_complete /
rich_bidi_vertical_gdef_and_managed_validation_open`

This is a non-validation infrastructure slice for `RTS-P1-034`. It does not close the item or any
product, corpus, performance, power, WGPU, or screenshot gate.

## Current-source finding

The process-local `ResolvedTextGlyphArtifact` already snaps caret, hit-test, and selection geometry
to backend glyph-cluster edges. The remaining split was the stale-or-missing artifact route:

- caret and IME geometry reshaped a source prefix but discarded `MeasuredGlyphCluster`;
- selection could reshape two prefixes for every visual span;
- editable pointer hit-testing fell back to per-grapheme DTO midpoints;
- the existing `AtomicCluster` measurement receipt was therefore consumed by wrapping but not by
  this source-qualified UI fallback.

Local Unreal `FShapedGlyphSequence` retains source-to-glyph mapping and treats a glyph covering
multiple grapheme clusters atomically when a font caret is unavailable. The checked local
`ttf-parser`/RustyBuzz stack still exposes no GDEF LigCaretList caret provider, so Zircon must publish
an explicit atomic fallback rather than invent equally-spaced interior carets.

## Implemented boundary

`GraphemeAdvanceIndex` is now the single crate-private owner for three LTR atomic-cluster queries:

1. an interior source caret resolves to the leading or trailing cluster edge by affinity;
2. a physical hit resolves to the nearest legal cluster edge and matching visual grapheme boundary;
3. any non-empty selection intersecting an atomic cluster expands to the full cluster range.

The UI source fallback builds this index by shaping the complete final physical line once. It is
admitted only when source and visual text are exactly congruent, the line has one LTR run, and the
layout is horizontal, non-justified, non-ellipsized, and tab-free. Rich, secure, virtual, BiDi, and
vertical routes continue to require their canonical artifact or fall back to the existing DTO; they
do not reinterpret source order.

Editable pointer dispatch already owns the render command's exact text and complete resolved style,
so it now calls the source-aware hit route. Commands without text continue to use the DTO-only entry.
No public ABI, serde field, resolved-layout DTO, cache key, or renderer contract changed.

## Complexity and performance gate

- canonical artifact hit: unchanged; no source fallback shape;
- eligible caret/hit query: one complete physical-line shape plus index construction
  `O(graphemes + backend clusters)`;
- atomic caret/hit lookup after construction: `O(log graphemes + log clusters)`;
- eligible selection: one shape per intersecting physical line and one monotonic cluster expansion,
  replacing repeated prefix reshapes per visual span;
- ordinary non-profiling code adds no timing or dynamic profiler label.

The extra source shape is intentionally limited to a missing/stale canonical artifact. Managed
profiling must still compare artifact-hit, stale-artifact, and DTO-only lanes over 1/100/1k/10k
graphemes, 31 samples, with shape calls, CPU p50/p95/p99, allocation/RSS, and power. A retained
geometry artifact or cache is not authorized until that evidence shows this recovery route is a
material bottleneck.

## Static evidence and open work

Behavior regressions are authored for atomic caret edges, physical hit midpoint selection, visual
boundary index, and partial-selection expansion. Rust 2024 formatting, scoped diff-check, call-site
inventory, and file-budget checks pass; the largest touched owner is 684 lines. Managed Cargo was not
run in this slice.

Still open: cross-rich-run grapheme continuation, rich/BiDi/vertical missing-artifact geometry,
arbitrary public source-range measurement, font-derived GDEF carets, official corpus, managed fault
and performance runs, WGPU framebuffer, PNG evidence, commit, and WeCom notification.
