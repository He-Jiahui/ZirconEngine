---
related_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
related_code:
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/line_break/mod.rs
  - zircon_runtime/src/text/layout/line_break/boundary_correction.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/layout/rich/materialize.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/glyph_artifact.rs
  - zircon_runtime/src/text/layout/logical_virtual_line.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/candidate_line.rs
  - zircon_runtime/src/ui/text/layout_engine/virtual_fragment_sequence.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_layout.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_layout_vertical.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontMeasure.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/slint/internal/core/textlayout.rs
  - dev/slint/internal/core/textlayout/fragments.rs
status: implementation_complete_static_checked_validation_pending
---

# Text03 Physical Line Metrics And Shaping Reuse Design

## Scope And Status

This is a non-acceptance design record for Text03. It records a current-source
audit and the implementation gate for the physical-line metrics hard cut. It
does not claim a Cargo result, a performance measurement, a framebuffer image,
or milestone acceptance.

The near-term MVP remains the same: a packaged default font, real shaping,
correct wrapping, and a renderable UI text artifact. The work below corrects a
structural layout defect without creating a second shaper, a parallel glyph
model, or a compatibility facade.

## 2026-08-29 Physical-Line Geometry Re-Review Before Implementation

The current-source audit found a second structural defect at the public line
boundary. `UiResolvedTextLine.frame` simultaneously carries the aligned content
origin, a main-axis extent clamped to the paragraph slot, and the rectangle used
to choose a line during pointer hit testing. Natural glyph advances can extend
beyond that rectangle, while a short aligned line exposes no full placement slot.
The result is one value with three incompatible meanings across renderer,
caret/selection/IME, rich-table hit routing, and clipping consumers.

Unreal's `FLineView` does not collapse those meanings. It retains content-space
`OffsetY` and natural `Size`, computes justification as a display offset, and
converts display-space input back to content space before block hit testing.
The Zircon migration therefore uses these invariants:

1. `UiResolvedTextLine.frame` is the absolute natural **content frame**. Its main
   extent agrees with the final resolved advances; its cross extent is the
   physical line/column extent.
2. `placement_frame` is the absolute paragraph/cell slot used to choose and
   retain a physical line. Alignment never changes this slot.
3. Content hit candidacy is obtained through `hit_frame()` and is deliberately
   the content frame. A point in a line slot may choose the nearest caret without
   making empty aligned space part of a rich link.
4. Horizontal and `VerticalRl` paths apply the same main/cross-axis contract,
   and table translation moves both frames atomically.

This is a correctness migration, not a claimed performance optimization. The
audit found no new shaping, wrapping, or allocation loop to optimize. The DTO
adds one `UiFrame` (16 bytes) per published line rather than duplicating a third
hit rectangle. Performance acceptance remains gated on the existing cold/warm,
scroll/edit, profile, power, and product-frame matrices; no timing or power claim
is made by this implementation record.

Implementation status on 2026-08-29 is
`content_and_placement_geometry_separated / four_layout_routes_migrated /
interaction_consumers_migrated / static_checks_complete /
managed_validation_pending`. The required serde field intentionally rejects the
old ambiguous DTO shape. Focused contracts cover distinct frame roundtrip,
atomic translation, overwide natural main-axis extents, right-aligned slot-gap
caret placement without content activation, and rich-link exclusion. No Cargo,
profile, allocation, power, Native/SDF WGPU, framebuffer PNG, milestone, commit,
or WeCom claim is made by this non-acceptance record.

## Current-Source Findings

1. `text/shaping/horizontal/direct.rs` derives ascent, descent, leading, and
   baseline from the actual faces selected for a shaped hard line. The
   `ShapedTextLine` DTO already carries those metrics.
2. `text/layout/rich/materialize.rs` already has the required line-view
   pattern: it combines the involved run ascents/descents and inline-object
   extents, advances a cumulative y cursor, and publishes a line-local
   baseline. This is the closest current implementation to Slate's physical
   line model.
3. The plain horizontal UI path now materializes a fragment for every
   source-congruent non-virtual physical candidate and uses its metrics for
   capacity and frame publication. It reuses raw fragment advances only when
   they are independent of the physical pen position: a tab-containing line
   retains the exact fragment for metrics and artifact identity, while the
   existing tab-stop owner supplies final x advances. The `"Hg"` sample remains
   only for empty, synthetic, and viewport-uniform fallback paths. The Text03
   parent plan explicitly records that those paths require their own certified
   line model; this remains known architecture debt rather than a font-tuning
   issue.
4. The normal line breaker uses `TextShapeRunProvider` for UAX#14 opportunities
   and boundary correction. `SharedTextLayoutSession` owns the only shaped-run
   cache and keys it by exact text, absolute source range, style, direction,
   orientation, features, language, and font generation.
5. `text/glyph_artifact.rs` projects a source-congruent retained final-line
   fragment directly into renderer glyph and font-handle identity. The private
   handoff is indexed only by the final published plain lines and rechecks
   source text, absolute range, and captured font generation. A horizontal
   LTR ellipsis line with contiguous LTR visual runs uses the separate
   visual-projection owner: it shapes the full physical display line locally,
   restores each glyph's source ownership from the runs, and marks zero-width
   anchors as virtual. Other source-congruent horizontal Plain virtual lines
   retain `LogicalVirtualLineSequence` before UAX#9 materializes physical text.
   Layout shapes its canonical logical fragment once for metrics and advances;
   current-generation artifact construction projects that same fragment through
   the captured visual permutation. Stale, unordered, non-isomorphic,
   cross-anchor or cross-direction cluster output, rich, and vertical entries
   retain the conservative artifact-shaping/render fallback.

## Invariant

For every published physical text line, its frame height, baseline, glyph
advances, glyph IDs, face/instance IDs, and artifact source range must derive
from one canonical shaped line fragment. A line may contain explicit synthetic
content (ellipsis, soft-hyphen suffix, or Arabic tatweel), but that content
must be represented as an explicit shaped virtual fragment with a zero-width
source anchor. It must never be inferred from a font-size ratio.

The input to the canonical fragment is the logical source span plus its
boundary-shaping context. Visual reordering remains a Text03 presentation
operation after the logical line is chosen. A ligature, combining sequence, or
RTL cluster that crosses a soft-wrap boundary must keep using the existing
boundary-correction owner; slicing a whole-paragraph glyph vector at arbitrary
byte offsets is invalid.

## Current Implementation Assessment

The first non-virtualized plain-text slice is present in current source, but it
is not accepted as a performance implementation:

1. `text/font/line_metrics.rs` is the shared selected-face ascent/descent/gap
   owner used by both the horizontal shaper and Text03.
2. `text/layout/physical_line_fragment.rs` now owns one immutable
   `CanonicalPhysicalLineFragment`: the final shaped run, derived metrics, and
   grapheme advances share one `Arc`. The non-virtualized plain owner maps a
   candidate's local source range to the shaper's absolute range exactly once,
   then retains source-congruent entries through overflow materialization with
   hash indexing plus full text/range equality. The old complete-paragraph
   metric probe and its binary source-face projection are removed.
3. Plain non-viewport layout resolves candidate metrics before capacity, then
   refreshes fragments after ellipsis/tatweel and gives `line_box` the retained
   advances. Source-congruent unchanged lines therefore have one layout shape;
   virtual candidates fail closed to the existing separate path. Frame
   y/height/baseline and complete document height consume the fragment metrics.
   Viewport layout remains deliberately uniform-height until a prefix-metrics
   cache is available.
4. `tests/physical_line_metrics.rs` asserts that the UI line DTO agrees with
   the current direct-shaper baseline and frame-height contract. Its soft-wrap
   regression also compares every resolved physical line against an independent
   direct shape, then verifies adjacent frames advance by the previous actual
   height. It deliberately does not require a CJK fallback baseline to differ
   from the primary face: that is a layout-policy decision, not a font-metadata
   fact. The focused managed Cargo invocation returned no durable receipt, so
   these are pending validation tests, not passed tests.
5. Profiling capture records the added physical-line source-shape request and
   shaped-cache delta so the extra topology is visible. The existing profiling
   test checks the counter contract without adding machine-dependent timing
   thresholds. These counters are diagnostic evidence, not a performance win.
6. The layout root's failure-publication behavior has moved to
   `layout_engine/failure_layout.rs`; the root is 792 lines after the split and
   remains an orchestration facade.
7. `text/layout/measure.rs` now has `MeasuredTextLine` and
   `measure_line_with_provider`: grapheme advances and selected-face metrics
   are projected from one final-line shape. Its focused one-shape regression
   is present but has no managed Cargo receipt. Intrinsic `TextSize` now sums
   each non-empty hard line's real height from this DTO; empty lines retain the
   explicit sample-metric fallback. The DTO is still not consumed by `line_box`.
8. `ui/text/layout_engine/measurement.rs::measure_unwrapped_text_height_with_provider`
   is a separate fixed-width height fast path. It obtains a shared-font snapshot
   and first preserves the existing primary-only certificate: that path shapes
   the `"Hg"` sample once, multiplies by the hard-line count, and rejects a
   generation change. For fallback content it now asks the generation-local
   `FontChainLineMetricEnvelope`; only when the requested line height covers
   every eligible face's maximum content extent plus the resolver primary gap
   does it return the requested height. Otherwise it returns `None` for
   complete measurement. Fallback content never inherits the Latin sample's
   line height.
9. The horizontal UI dispatch formerly entered its general rich-layout route
   only when the parsed document had an inline object. A rich document
   containing only styled text consequently fell through the plain candidate-line wrapper, which
   measured with the base `UiResolvedStyle` instead of the run-specific style.
   The route now enters `rich_layout` for every non-Plain horizontal rich
   format, while Plain text retains its existing owner. The same repair closes
   an incomplete `TextLayoutOutcome<Vec<f32>>`
   conversion in `item_advances`. The focused BBCode size-override regression
   contains no inline object and requires rich width and height to exceed its
   plain counterpart. The same owner split now routes every non-Plain
   `VerticalRl` document through `rich_layout_vertical`, so a styled-only
   vertical document cannot fall through the base-style column wrapper. This
   is a dispatch repair, not a second rich shaper.
10. `FontFaceMetadata` obtains `ascender`, `descender`, and `line_gap` from
    `ttf-parser`'s normalized face metrics, which select OS/2 typo values when
    `USE_TYPO_METRICS` is set and otherwise hhea values. The content-envelope
    owner formerly re-promoted raw Windows ascender/descender whenever typo
    metrics were disabled, contrary to Text03's single-face rule. It now
    consumes the normalized values only; Windows metrics remain asset metadata
    for clipping consumers. A private regression gives Windows bounds larger
    than hhea and locks the hhea-derived scaled envelope.
11. `FontShapingFaceResolver` chooses one primary face before fallback
    itemization, and `FallbackTextSpan` now retains that same identity
    independently of every actual selected face. Direct horizontal, direct
    vertical, and cosmic fallback output copy it into
    `ShapedGlyphRun.primary_face_id`; the CJK-first fallback and non-empty
    serde roundtrip regressions reject loss or inference from the first
    selected span. The direct horizontal content envelope and the physical-line
    probe now take maximum selected-face ascent/descent with that resolver
    primary face's line gap. This is only raw content-extent input:
    `ShapedGlyphRun` retains no fragment metric policy, and the hard cut must
    materialize the immutable identity into the canonical fragment without a
    second resolver pass before it establishes a composite baseline and glyph
    origin adjustment.

This assessment establishes the correctness data flow but does not prove a
performance win. The plain non-virtual artifact path now receives the exact
final fragment instead of issuing its former independent normal-line request.
Its cached source/text/range guard falls back to the established artifact
request when the handoff cannot prove identity. Profiling and a managed test
receipt must still show the duplicate topology is absent before this becomes an
accepted optimization.

## Cross-Engine Fragment Corroboration

Slint's `internal/core/textlayout.rs` constructs one `ShapeBuffer` before
its line breaker operates, and `textlayout/fragments.rs::TextFragment` carries
both byte and glyph ranges with measured width and trailing-break context. Its
renderer then consumes positioned glyphs from that layout result rather than
asking a renderer-side shaper to rediscover the line. This corroborates the
same ownership direction as Slate: shaping identity belongs to the layout
fragment, and presentation is a projection of it.

Zircon intentionally does not adopt Slint's uniform `font.height()` line
advance. That is appropriate for its single active font abstraction but cannot
certify Zircon's resolver-selected primary face plus selected fallback faces.
Zircon's fragment needs maximum selected ascent/descent, resolver primary
line-gap identity, and later composite baseline/origin adjustment. The adopted
structure is therefore Unreal's run/block aggregation with Slint's explicit
byte/glyph fragment ownership, not a copy of either engine's line-height rule.

## Unreal Baseline Re-Review

The source reference changes the target formulation for the physical-line
hard cut. `FShapedGlyphSequence` retains glyph identity, source coverage, and
measured width as one shaped object. `FTextLayout::CreateBreakCandidate` and
`FlowLineLayout` then aggregate `Run.GetBaseLine` and `Run.GetMaxHeight` into
maximum above/below-baseline extents and place each run block with an explicit
vertical offset. Slate's composite-font character cache also normalizes a
fallback glyph's raster baseline to the default face. These are separate
operations; a selected fallback face's raw ascender is not itself the public
line baseline.

Zircon's current direct shaper exposes maximum selected-face ascent/descent
with the resolver primary face's line gap and derives a raw per-hard-line
content baseline from them. That is useful correctness data, but it is not yet
a proven Slate-equivalent line-policy contract. The canonical fragment must
therefore carry all of the following independently:

- selected-face ascent/descent/line-gap extents for clipping and glyph-origin
  adjustment;
- a run/composite baseline policy selected from the resolved style and font
  collection;
- the final line's maximum above/below-baseline extents after rich runs and
  inline objects participate; and
- source/cluster/glyph identity used by rendering and hit testing.

Do not use the current CJK-versus-Latin baseline values as an acceptance
oracle. The future line-box change must choose one policy for layout and pass
the corresponding glyph-origin adjustment to raster projection; changing only
the line DTO baseline can misplace or clip fallback glyphs. This re-review
does not revert the current probe and does not claim a visual result.

## Target Ownership

```text
TextRuntimeService / SharedTextLayoutSession
  owns ShapedParagraph and cache generation
  owns canonical ShapedLineFragment requests
    -> glyphs, advances, selected face extents, primary-face identity,
       baseline policy, source/context ranges

Text03 line breaker
  owns break opportunities and PhysicalLine source/virtual-fragment sequence
  requests or reuses one canonical fragment per physical line

Line metric policy
  owns run/composite baseline normalization and line-local max-above/max-below
  aggregation; selected-face raw extents remain available for glyph placement

UI layout presentation
  owns frames, alignment, clipping, viewport selection, and visual order
  consumes PhysicalLineMetrics; does not sample "Hg" or reshape for metrics

FontDatabase metric-envelope service
  owns a generation-keyed upper bound for every face eligible through a
  resolved family/query/composite fallback chain
  returns CertifiedUniformHeight only when the requested height covers that
  bound; otherwise forces complete per-line measurement

Glyph artifact projection
  owns font-handle registration and renderer DTO projection
  consumes the same canonical fragment; it does not issue a second normal-line
  shape request
```

This is the Zircon analogue of Unreal's shaped-glyph-sequence plus line-view
ownership: measurement exposes source-indexed glyph data, line layout creates
physical views over it, and rendering projects those views. It is deliberately
not an adaptation of `cosmic-text` line layout, because Text03 remains the sole
owner of wrapping, kinsoku, ellipsis, and final UAX#9 visual ordering.

## Migration Algorithm

1. Preserve the primary face selected by `FontShapingFaceResolver` as
   immutable shaping metadata through fallback itemization into the canonical
   fragment. Do not re-resolve per physical line and do not infer it from the
   first selected fallback span. The line policy then takes maximum selected
   ascent/descent and primary-face line gap.
2. Introduce a private, immutable `PhysicalLineMetrics` owner beside Text03
   line presentation. It contains `ascent`, `descent`, `line_height`,
   `baseline`, and the canonical fragment identity. It has no renderer handles
   and no UI public-contract change.
3. Change the line-break result from a bare candidate text/range into a
   physical-line request that preserves the corrected source interval and any
   virtual fragment anchors. Continue to use the existing boundary-correction
   contract for soft-wrap edges.
4. Materialize a canonical fragment after final line construction and before
   frame placement. Preserve selected-face extents, then apply the explicit
   run/composite baseline policy and aggregate maximum above/below-baseline
   extents with inline objects, as Slate does for line blocks. Cumulative frame
   y becomes the sum of each resolved physical line height, rather than
   `line_index * sample_height`.
5. Move vertical capacity, clip decisions, and `measured_height` to the same
   prefix-height sequence. Viewport virtualization must either consume cached
   prefix metrics or remain on the existing uniform-line path until that cache
   exists; it must not silently mix uniform and actual line frames for the same
   document.
6. Project source-congruent plain canonical fragments into the glyph artifact
   through private request-local state. Keep a distinct explicit virtual-
   fragment path for ellipsis/tatweel and the conservative artifact request
   for rich, vertical, viewport, and any non-matching line until their owners
   move to the same contract.
7. Delete the plain-path `"Hg"` sample from layout frame publication. A sample
   remains permissible solely for empty-document intrinsic fallback where no
   source glyph or inline object exists.
8. Before retaining any O(1) fixed-width document-height shortcut, add a
   private `FontChainLineMetricEnvelope` query to `FontDatabase`. Its cache key
   includes the shared font generation, resolved family/query, requested style
   axes, language, and project composite identity. Its bound covers all faces
   eligible to the resolver, not a sampled string. The shortcut may return an
   exact height only when `requested_line_height >= envelope.max_content_height`;
   otherwise it must select complete per-hard-line measurement. It must never
   infer the proof from `"Hg"`, the current packaged font pair, or an arbitrary
   codepoint sample.

No compatibility wrapper or duplicated old/new layout route is allowed. The
move is complete only when all normal UI text, rich text, artifact extraction,
hit testing, and viewport geometry reach the same physical-line owner.

## Post-Review Boundary Decision

The current source review makes the migration order non-negotiable. The
existing `SelectedFaceLineExtents` produces a raw content envelope from maximum
selected-face ascent/descent and the resolver primary face's line gap. It is
useful input to clipping and raster placement, but its
`resolve_content_envelope` result cannot be published as the final UI baseline
until the matching glyph-origin correction is present. The first
`CanonicalPhysicalLineFragment` slice reuses that raw data for source-congruent
plain lines only; it remains `implemented_not_acceptable` until the composite
line policy and glyph-origin projection are complete.

The hard cut must introduce a private `HorizontalLineFragment` owned under
the backend-neutral text shaping layer. It contains the corrected source range,
boundary context, logical glyph clusters, selected-face content extents, and a
composite baseline. `Text03` may materialize this fragment only after its line
breaker has selected the final physical interval. The line-policy owner then
computes a frame from the maximum above/below extents of all text runs and
inline objects and assigns each glyph a finite origin adjustment from its face
baseline to the composite baseline. Artifact projection forwards that adjusted
glyph origin; it must not reconstruct it from `font_size`, `"Hg"`, or a second
shape request.

This keeps the responsibility boundaries required by
`engine-code-structure-convention.md`: the layout root remains orchestration,
the named fragment owner shapes and retains source identity, the named line
policy owner computes vertical geometry, and the renderer consumes prepared
glyph origins. The former inline-only name was a naming/ownership regression
once all horizontal rich documents entered that route. The source now uses the
dedicated `rich_layout` and `rich_layout_vertical` owners and no
`rich_inline` compatibility module; the canonical-fragment cut must extend
those owners rather than restore an inline-only alias.

The in-progress shared edit to `rich_layout.rs` has independently converted
parts of the route to `TextLayoutOutcome`; during this review its
`item_advances` fallback returned a bare `Vec` despite the new outcome return
type. The source now returns `TextShapingOutcome::Ready(Vec::new())`. This is
a source-level closure repair, not acceptance evidence for the dispatch or
canonical-fragment work.

## Cache Topology Review

`ShapedRunCacheKey` correctly includes the text hash, absolute source range,
normalized line height, face request, direction, orientation, feature list,
language, and font generation. The key is intentionally exact: it must not be
weakened to make different fragments appear reusable, because that would break
cluster identity and generation invalidation.

This makes the present topology measurable rather than speculative:

1. Boundary correction and wrapping retain their independent shaping requests:
   a final wrapped source interval has an exact text/range/style identity and
   must not be aliased to another interval merely to improve a cache ratio.
2. The plain non-virtual final-line owner creates that exact fragment once.
   `line_box` consumes its advances and the artifact builder projects the same
   run when the request-local source/text/range guard matches the published
   line. That branch makes no artifact-stage shape-cache request.
3. A horizontal final-LTR ellipsis line with contiguous LTR visual runs performs
   one explicit visual-line shape and monotonic projection. It cannot reuse a
   source-congruent fragment because its display text contains zero-width source
   anchors. The profiling counter records this distinct topology. RTL/tatweel,
   mixed-direction, rich, vertical, viewport-selected, tab-stop-positioned,
   unordered, clipped, and otherwise non-matching lines retain the conservative
   fallback.
4. The profiling regressions record
   `physical_line_fragment_initial_shape_request_count`, its shaped-cache
   hit/miss deltas, `artifact_build_retained_fragment_projection_count`, and
   `artifact_build_fallback_shape_request_count`, plus
   `artifact_build_visual_projection_shape_request_count`. Counters demonstrate
   topology, not elapsed-time improvement.

Therefore no shaped-cache capacity, hash, alias, or prewarm tuning may be
attempted for this issue. The performance hard cut is to retain canonical final
fragments after boundary correction, then let artifact projection consume those
fragments directly. That removes a request topology rather than attempting to
hide it behind a less correct cache key.

## Font-Chain Envelope Implementation (2026-08-26)

`FontChainLineMetricEnvelope` is implemented as a private Text03 admission
service in `text/font/line_metrics.rs`. Its candidate set was traced through
the current `FallbackResolver` and `CompositeFontIndex`; it neither samples the
current text nor walks every registered font face. For one normalized query and
language, the safe upper-bound family set is the de-duplicated union of:

1. every project-composite sub-font whose culture is eligible for the language;
2. the composite default family;
3. the explicit query families; and
4. the database fallback families.

For every family, the envelope visits every query-matched face before
codepoint coverage filtering, and includes the resolver primary face even when
it is already part of that union. This deliberately over-approximates the
script/range branch selected by one cluster: `FallbackResolver` may choose a
complete candidate, a partially covered base candidate, or the primary
last-resort face. The result is a safe maximum content ascent/descent plus the
resolved primary line-gap, not a sampled-string metric and not a public line
baseline.

The bounded cache key contains the normalized query identity, normalized
language, project-composite identity, and scaled font size. The cache belongs
to the immutable `FontDatabase` snapshot and is detached whenever render inputs
advance the shared database generation, so it cannot cross a font-generation
boundary. It is capped at 64 entries and 1/64 of the existing 2 MiB fallback
cache budget; its bytes and evictions remain part of the existing total report.
It does not reuse the fallback resolver's codepoint-keyed candidate cache.

Its only fast-path result is a certificate that the requested line height covers
the envelope. That branch returns the requested height directly; it does not
shape `"Hg"`. Otherwise the caller returns `None` and complete per-hard-line
measurement remains the only exact path. This follows Unreal's separation of
composite typeface selection from `FShapedGlyphSequence` baseline and max-height
ownership: the envelope admits a uniform-height shortcut, while final line
policy still belongs to actual shaped runs.

Profiling remains a prerequisite to cache-capacity or allocation tuning. The
managed Cargo acquire timeout recorded below prevented the required profiler
run, so this section establishes candidate correctness and an implementation
plan only; it reports no timing, energy, or reference-engine comparison.

## Virtual Fragment Contract

`CandidateLine` expresses ellipsis and Arabic tatweel as non-empty visual runs
with a zero-width source range. This is the correct UI selection and IME
anchor. A final-LTR line whose every visual run is LTR continues to use the
physical visual-line path, which walks contiguous visual runs once to restore
source clusters. Source-congruent horizontal Plain virtual lines that cannot
safely shape physical text retain a logical display sequence before UAX#9;
initial construction and generation rebuild shape that logical input and
project it through the captured permutation. A glyph entirely backed by one
zero-width run receives `anchor..anchor` and `virtual_glyph = true`; a glyph
that crosses ordinary and virtual clusters, spans distinct virtual anchors,
crosses a bidi direction boundary, has unordered ranges, or lacks complete run
coverage is rejected and uses the existing renderer visual fallback. The
visual-run and logical-sequence traversals use monotonic cursors, so they are
respectively `O(G + R)` and `O(G + C)`, rather than a per-glyph run scan.

This is deliberately not a synthetic absolute-range shape request: supplying
`"…"` with an absolute `anchor..anchor` range would corrupt backend cluster
coordinates. It is also not yet the full typed physical-fragment sequence:
retained source-congruent fragments continue to own the normal hot path, while
the generated portion is a complete visual-line shape retained only in the
artifact.

The remaining hard cut introduces a backend-neutral private sequence:

```text
PhysicalLineFragmentSequence (logical source order)
  Source { CanonicalPhysicalLineFragment }
  Virtual {
    kind: Ellipsis | ArabicTatweel,
    display_text: Arc<str>,
    zero_width_source_anchor: usize,
    inherited_bidi_level: u8,
    logical_insertion_index: usize,
  }
```

Each `Virtual` entry shapes `display_text` with a local `0..display_text.len()`
range. Projection then replaces every output glyph source range with
`anchor..anchor`, marks the glyph virtual, and applies the final resolved
advance without inventing a selectable source span. The sequence is reordered
by the existing UAX#9 result: a virtual entry inherits the level of its
adjacent anchored source cluster, never runs BiDi analysis over a zero-length
interval. Artifact projection concatenates source and virtual projected glyphs
in that resolved visual order, preserving backend order within a cluster.

For `EllipsisStart`, `EllipsisEnd`, and `EllipsisMiddle`, the anchor remains
the existing line start/end boundary selected by overflow policy. Tatweel uses
the `insert_virtual_text` anchor at the justified Arabic cluster boundary.
The migration must add regressions for all three ellipsis positions, multiple
tatweel insertions, mixed RTL source, hit testing at each anchor, and glyph
source ranges/virtual flags. The current pure and artifact/rebuild regressions
cover one end-anchor artifact path plus 64 distinct LTR virtual anchors. The
plain retained-fragment branch does not partially reuse virtual content; the
full sequence remains required for rich, vertical, and mixed-fragment adoption.

The measurement procedure follows the existing Text09 WGPU matrix: 1, 100,
1,000, and 10,000 label rows; 60 warm-up frames; 300 measured frames; three
repetitions per named scenario. Record p50/p95 CPU stage time, resolved GPU
time, exact shape-cache request/hit/miss/insert deltas, artifact fragment
reuse, native/SDF raster work, upload bytes, allocation deltas, and frame
energy only through the separate system-owner capture. The accepted claim is
that the duplicate normal shape is absent and the measured bottleneck moves;
it is not a comparison against an unmeasured generic engine power figure.

## Status Ledger (2026-08-25)

| Work item | Status | Evidence or remaining gate |
|---|---|---|
| Packaged default font bootstrap and deterministic fallback | implemented, validation pending | Text01 source and its focused tests exist; no managed Cargo receipt in this session. |
| Direct selected-face metric capture | implemented, validation pending | `SelectedFaceLineExtents` preserves parser-normalized typo/hhea priority, combines maximum selected ascent/descent with the resolver primary face's line gap, and remains content-extent data rather than accepted UI line policy. |
| Final source-congruent physical-line fragment | implemented, validation pending, not acceptable | `CanonicalPhysicalLineFragment` retains one absolute-range shaped run, metrics, and advances. `line_box` and source-congruent plain artifact projection reuse it; the paragraph probe is deleted. Virtual and tab-stop-positioned candidates deliberately remain outside this slice. |
| Horizontal virtual artifact projection | implemented, static checks passed, Cargo pending | Source-congruent Plain ellipsis/tatweel/mixed lines retain `LogicalVirtualLineSequence` before UAX#9. Its canonical logical fragment supplies final metrics and advances, then current-generation artifact build projects the same run by captured visual indices; only an absent or stale generation falls back to re-shaping preserved logical input. Projection is `O(G + C)`. Rich, VerticalRl, non-isomorphic runs, cross-direction/anchor glyph clusters, and non-monotonic backend output fail closed to the existing fallback. |
| Styled-text-only rich dispatch and owner naming | implemented, validation pending | Non-Plain HorizontalTb enters `rich_layout`; Non-Plain VerticalRl enters `rich_layout_vertical`. The outcome conversion is closed and the former inline-only module/test paths are deleted without aliases. |
| Primary-face propagation through shaped output | implemented, validation pending | `FallbackTextSpan` carries the resolver-selected primary face without a second resolution pass; direct horizontal/vertical and cosmic fallback copy it into `ShapedGlyphRun.primary_face_id`. CJK-first fallback and serde regressions cover identity retention. |
| Canonical fragment, composite baseline, glyph-origin projection | implementation in progress, validation pending | Source-congruent plain final-line fragments now reach `line_box` and direct artifact projection through request-local state. Horizontal Plain virtual fragments now replace sample metrics with their canonical logical-fragment metrics and reuse that run for artifact projection. Remaining work is boundary-context retention, composite baseline/origin adjustment, and rich/vertical adoption. |
| Fixed-width height fast path certification | primary and fallback-chain certificates implemented, validation pending | Primary-only content keeps the one-sample path. Fallback content is admitted only when the generation-local `FontChainLineMetricEnvelope` covers every eligible query/composite/fallback face; otherwise the shortcut returns `None`. The new private regression locks composite face inclusion before coverage filtering and cache hit/miss behavior. |
| Text03 measurement and logical-sidecar owner split | implemented, static validation passed, Cargo pending | `layout_engine.rs` remains below the 800-line review threshold after it moved virtual candidate capture/UAX#9 selection into `layout_engine/virtual_fragment_sequence.rs`; the root keeps layout routing, while the child owns source/display selection and sidecar capture. |
| Performance baseline and power comparison | planned, not measured | `2026-08-26-virtual-projection-profiling-plan.md` defines the 1/100/1k/10k matrix, 60 warm-up, 300 measured frames, three repetitions, counter topology, and separate power-owner requirement. No timing or power receipt exists. |
| WGPU product framebuffer | pending | Actual image must be produced by the product test under `docs/tests/runtime/text`; no old image or textual strategy output qualifies. |

No row in this ledger is an accepted milestone. It exists so that follow-up
implementation can move individual rows only with a durable validation receipt
and an actual render artifact where required.

## Current Validation State

The source slice has passed scoped Rust 2024 formatting and `git diff --check`
for its owned files without whitespace or line-ending warnings. The visual
projection leaf, logical virtual-sequence leaf, artifact root, and their focused
contracts passed scoped formatting after the horizontal logical-sequence branch
was added. Artifact lifetime/selection remains in the 771-line root; shaping and
font-handle projection are isolated in the 281-line `projection.rs` leaf, visual
source/anchor projection is isolated in the 427-line `visual_projection.rs`
leaf, logical display/anchor/order/fragment state is isolated in the 551-line
`layout/logical_virtual_line.rs` leaf, and the layout root is 799 lines with its
virtual-fragment adapter isolated in a 383-line child. The selected-face owner has no remaining
`FaceLineMetrics`, old `resolve`, or Windows-metric-promotion production path. The newly
implemented font-chain envelope and visual-run projection are static-only at
this point: no managed Cargo receipt, profile capture, or render evidence
exists.

The repository-wide `check_conventions.py --only docs` gate is currently red
on the shared checkout because it reports unrelated missing-path references
outside Text03. The tool has no single-document scope, so that global result is
not used as evidence for this record and is not repaired here. Managed Cargo,
the Text09 performance matrix, and the real WGPU framebuffer exporter have not
produced a durable success receipt in this session; all remain pending. The
focused managed lib-test invocation for the styled-rich dispatch also returned
without a durable stdout or result receipt, so it is explicitly not counted as
a passing Cargo test.

On 2026-08-26, the exact ignored WGPU exporter was submitted through
`validate-matrix.ps1` with `--test runtime_text_multilingual_product_framebuffer`,
the one exporter filter, `target-client`, and an `E:` target directory. It
stopped in the managed `cargo.acquire` preflight with coordinator error
`command_post_timeout`; Cargo never started and no `20260823` PNG was written.
This is a validation-infrastructure block, not a successful render, source
compile result, or screenshot receipt. Do not retry by polling the coordinator;
continue independent source work until a new managed lane is available.

## Required Tests

1. A wrapped mixed-script/fallback paragraph has line-local baseline and frame
   height equal to its canonical shaped fragment, and cumulative line frames do
   not overlap.
2. A fallback glyph with larger raw ascent/descent is placed inside the
   line-policy bounds without clipping or source/face drift. A following line
   starts at the resolved line bottom; the test must not assume raw fallback
   ascent itself selects a distinct public baseline.
3. A rich size override plus an inline image preserves the existing rich-line
   baseline behavior through the common physical-line owner.
4. Ligature, combining-mark, Arabic, CJK kinsoku, and RTL wrap-boundary cases
   retain source ranges, visual ordering, and glyph advances while using the
   new metrics path.
5. Ellipsis, soft hyphen, and materialized tatweel stay explicit virtual
   fragments and do not create source-offset or font-identity drift. Cover all
   placement modes, multiple anchors, and mixed RTL before replacing the
   current visual-line projection with the typed fragment sequence.
6. Artifact glyph IDs, faces, advances, and per-line metrics equal the
   canonical layout fragment without a second normal-line shaping miss.
7. The real WGPU product framebuffer test writes only beneath
   `docs/tests/runtime/text` and is pixel-inspected after the functional tests
   pass. A text dump or strategy diagram is not an acceptance image.
8. The fixed-width height shortcut either proves the packaged Latin+CJK
   fallback-chain envelope and equals complete measurement, or returns `None`
   so the existing caller takes complete measurement. Its 1k/10k plain-log
   one-shape regression remains required only for the certified envelope case.

## Measurement Gate

The executable counter matrix and non-claim boundaries are recorded in
`2026-08-26-virtual-projection-profiling-plan.md`. That record is
`planned_not_measured`; it does not replace the durable managed validation
receipt required below.

Before changing cache ownership or making a performance claim, capture the
existing Text03 ignored benchmark with profiling disabled and a stable warmed
session. For Latin, CJK, RTL, ligature, and mixed-fallback inputs at 1, 100,
1,000, and 10,000 graphemes, record 31 cold and warm samples of:

- layout wall-clock p50/p95;
- artifact wall-clock p50/p95;
- shaped-cache hit/miss/insert deltas;
- canonical-fragment request count and boundary-correction count;
- font-handle registration count, lock wait, and lock hold deltas;
- allocated fragment and glyph counts.

The before/after runs use the same machine, font generation, renderer setup,
and source corpus. The CPU recorder is observer-prone, so it may identify
topology but cannot by itself support timing, power, or reference-engine
equivalence claims. Power comparison requires a separately documented platform
sampling window. Until those receipts exist, all performance and power status
is `measurement_pending`.

## Current Next Step

The non-Plain rich dispatch repair is complete in source: every horizontal rich
document, including styled text without an inline object, enters `rich_layout`,
and the matching vertical documents enter `rich_layout_vertical`.

The first canonical fragment cut now spans source-congruent non-virtual plain
lines from final candidate through `line_box` and direct glyph-artifact
projection. It preserves one absolute-range shaped run, metrics, and advances;
the complete-paragraph metric probe is gone. Final-LTR contiguous virtual runs
use a local visual-line artifact path, while RTL/tatweel and mixed-direction
source-congruent Plain runs retain logical display sequencing through first
build and generation rebuild; neither path pollutes source-fragment reuse.
Tab-stop-positioned lines retain the established measurement/artifact path
until a fragment also owns pen-position-dependent tab expansion. Rich/VerticalRl
adoption, composite baseline/origin adjustment, and the viewport prefix-metrics
cache remain separate hard cuts.

The fixed-width height shortcut now admits either a generation-stable
primary-only certificate or a generation-local fallback-chain envelope. The
fallback branch returns a constant only when the requested line height covers
the complete eligible-font bound; otherwise it returns `None` for complete
measurement. Do not replace this policy with unconditional per-line shaping:
the retained-log path has an explicit one-shape topology contract.

The focused Cargo regression and the ignored Windows WGPU product test were
each submitted through the managed validator, but neither returned a durable
receipt. The next independent implementation work is typed virtual-fragment
sequence design for rich/VerticalRl rather than coordinator polling. The expected
`runtime_text_mvp_foundation_product_framebuffer_20260823.png` is absent from
both `docs/tests/runtime/text` and `target`; no old image, text dump, or
strategy graphic is being used as a substitute. After the virtual-fragment cut,
obtain those receipts, inspect the new product framebuffer, and then run the
required 31-sample baseline. The current artifact handoff must demonstrate
that normal-line duplication is absent before any performance claim.

## 2026-08-26 Logical Virtual Sequence Continuation

Status: `implementation_in_progress_validation_pending`.

The horizontal Plain owner now retains `LogicalVirtualLineSequence` before
`visual_order` turns a candidate into physical display text. The sequence holds
logical display text, non-empty local grapheme ranges, source anchors, resolved
bidi levels, and physical visual indices. `virtual_fragment_sequence.rs` is the
only adapter from `CandidateLine`; the sidecar remains request-local through
artifact construction and is retained only inside `ResolvedTextGlyphArtifact`
for font-generation rebuild. It does not enter `UiResolvedTextLayout`, UI cache
keys, or renderer-side source-mapping code.

This completes the original logical-order preservation cut, but did not yet
make that sidecar the canonical metrics or artifact-shaping owner. A
non-isomorphic source run, a glyph spanning ordinary and virtual content,
distinct virtual anchors, a direction-boundary glyph, or non-monotonic backend
output rejects only the artifact and keeps the established visual renderer
fallback. Rich and VerticalRl remain separate hard cuts.

Completed source checks are scoped Rustfmt parsing and `git diff --check`.
Managed Cargo, profiler data, power comparison, and an actual product WGPU
framebuffer under `docs/tests/runtime/text` are still pending. No image was
created, and no `target` output is used as evidence.

## 2026-08-26 Canonical Logical Virtual Fragment Follow-up

Status: `implementation_complete_static_checked_validation_pending`.

The source-congruent Horizontal Plain virtual path now owns
`CanonicalLogicalVirtualLineFragment` inside `LogicalVirtualLineSequence`.
Before UAX#9 turns a `CandidateLine` into physical display text, the owner
shapes preserved logical input once and retains its `Arc<ShapedGlyphRun>`, font
generation, exact `TextLineMetrics`, and grapheme advances. Layout applies the
fragment metrics to the final resolved frame instead of falling back to the
`"Hg"` sample, and visual ordering consumes those same advances. The state
stays private to `zircon_runtime::text`; it does not alter `UiResolvedTextLayout`
or cache identity.

Glyph-artifact construction now projects that current-generation fragment
directly. Font-generation mismatch or a missing retained fragment retains the
existing safe fallback: re-shape preserved logical input and project through
the captured visual order, never physical RTL text. The new counter topology is
one layout logical-fragment request plus one retained artifact projection per
typed virtual line at a stable generation; the artifact logical shape counter
must remain zero in that steady state. The algorithm keeps shaping at `O(S)` for
logical input size and projection at `O(G + C)` for glyph count `G` and cluster
count `C`; no cache-capacity, timing, allocation, power, or reference-engine
equivalence claim is made before the planned managed measurements.

The new focused regressions use a deterministic provider to prove that the
fragment supplies non-sample metrics and advances while receiving exactly one
shape request. The profiling-feature virtual-ellipsis regression additionally
locks the stable-generation layout-to-artifact counter topology. Scoped Rustfmt,
whitespace, production exception, counter, and owner-size checks passed.
Managed Cargo, profile data, and the required real WGPU framebuffer remain
pending at this document revision.

## 2026-08-26 Composite Baseline And Glyph-Origin Hard-Cut Design

Status: `raw_metric_provenance_complete_composite_policy_pending_measurement_pending`.

The next correctness cut is not a replacement of `UiResolvedTextLine.baseline`
alone. The current direct horizontal shaper creates a selected-face raw
content envelope, `rich/materialize.rs` already demonstrates the required
line-local aggregation with `max(ascent)`, `max(descent)`, and
`line_baseline - run_ascent`, and Unreal Slate's `CreateBreakCandidate` /
`CreateLineViewBlocks` computes max-above/max-below-baseline before assigning
each block a `VerticalOffset`. Those are three observations of one ownership
rule: a public line baseline is composite line-policy output, and every
participating glyph needs the matching origin correction.

The renderer audit adds a non-negotiable implementation boundary. Artifact
projection copies `ShapedGlyph` into `TextGlyph`, but the horizontal Native
and SDF routes place glyphs from the resolved line baseline plus
`TextGlyph.offset[1]`; they do not consume `TextGlyph.position[1]` for that
placement. The canonical fragment must therefore retain finite per-glyph
vertical-origin corrections and artifact projection must add them to the
prepared `offset[1]`. Storing only a changed line baseline, or changing a
cached `ShapedGlyphRun` in place, would leave artifact glyphs in their old
origins or contaminate another cache consumer.

The implementation sequence is fixed:

1. Extend only the backend-neutral horizontal shaped-line result with raw,
   immutable selected-face/run vertical metrics needed for later policy. This
   is not a public UI DTO and is not a second FontDatabase resolution pass.
2. Replace the two plain fragment-specific metric constructors with one
   private `HorizontalLineFragment` geometry builder. It keeps the cached raw
   `Arc<ShapedGlyphRun>` immutable and holds the final `TextLineMetrics`,
   grapheme advances, and a glyph-index-aligned origin-adjustment vector.
3. Make the line-policy child aggregate max-above/max-below extents across
   participating text runs. Rich text and inline objects join that owner only
   after the Plain contract is proved; VerticalRl remains outside this cut.
4. Add a fragment-aware artifact projector that zips prepared glyphs with the
   retained origin vector and adjusts `TextGlyph.offset[1]`. The normal,
   logical-virtual, stale-generation, and renderer-fallback branches keep
   their current source and font-generation admission rules.
5. Retire the old physical/logical fragment metric copies only when all
   Horizontal Plain callers use the new geometry owner. No compatibility
   facade, renderer-local baseline calculation, or public cache-key field is
   permitted.

The policy builder is linear in glyphs plus participating run metrics,
`O(G + R)`, and allocates one origin value per retained glyph. It is a
correctness architecture, not a measured optimization. Before making any cost
claim, add capture-only counters for fragment geometry build count,
origin-adjusted glyph count, and artifact origin-projection count; then use the
existing 31-sample Text03 protocol to compare the same warmed fonts and
corpus. No p50/p95, allocation, power, or reference-engine-equivalence value
exists yet.

Required first tests are deterministic and bottom-up: a primary/fallback
fixture must prove a composite baseline and its paired glyph offset; adjacent
physical frames must meet at exact resolved bottoms; a rich font-size override
plus inline object must share the same above/below policy; and artifact
projection must preserve IDs, faces, advances, source ranges, and add only the
approved vertical offset. Managed Windows tests and the real WGPU framebuffer
then verify both Native and SDF routes, including clipping, under
`docs/tests/runtime/text`. This design record is not an implementation or
render-validation receipt.

## 2026-08-26 Raw Horizontal Metric Provenance Foundation

Status: `implementation_complete_static_checked_compile_pending`.

`ShapedGlyphRun` now has a crate-private, serde-skipped/defaulted
`horizontal_line_raw_metrics` sidecar. It is index-aligned with hard lines when
available and each entry is either absent or a finite, non-negative
`HorizontalLineRawMetrics { ascent, descent, line_spacing_gap }`. This keeps
the immutable cached shaped run as the single geometry owner while avoiding a
new public UI DTO. The sidecar remains in the in-memory shaped cache only;
serialized shaped runs omit it and deserialize as an unavailable metric source.

The direct horizontal shaper creates the sidecar from the existing
`SelectedFaceLineExtents` in the same primary/fallback itemization pass that
already determines the current single-fragment envelope. It performs no
additional `FontDatabase` lookup. Every direct hard line receives one entry;
if a backend lacks usable face metrics that entry is explicitly `None`.
Cosmic fallback and VerticalRl emit an empty sidecar rather than inventing
horizontal metrics. The shaped-run cache budget now includes the retained
sidecar capacity, so cache admission does not undercount the new allocation.

Focused source regressions specify raw-extents extraction, direct horizontal
sidecar alignment, index-safe access, legacy serde absence, and cache-capacity
accounting. This cut deliberately does not consume the sidecar to alter
`ShapedTextLine.baseline`, `UiResolvedTextLine.baseline`, glyph origins, rich
layout, or VerticalRl. The next cut is the private `HorizontalLineFragment`
policy owner that aggregates participating runs and emits the already-prepared
glyph-origin sidecar.

Static checks covered formatter parsing of the changed leaf sources,
whitespace diff checks, construction-site audit, no-production-panic scan, and
owner-size checks. Repository-wide formatting is not clean in unrelated module
children, so it is not recorded as a formatting pass. Managed Cargo, profile
samples, power data, and a real WGPU framebuffer remain pending; this entry
makes no rendering or performance claim.

## 2026-08-26 Artifact Glyph-Origin Projection Foundation

Status: `implementation_complete_static_checked_policy_consumed`.

`glyph_artifact/projection.rs` now owns the private sidecar admission point.
The existing projection entry delegates with no sidecar, preserving current
output. The new path accepts exactly one finite y-origin adjustment per shaped
glyph, rejects a mismatched or non-finite vector before font-handle
registration, and applies the adjustment only to projected `TextGlyph.offset`
values. It validates every resulting y offset before mutation, so an overflowing
sidecar cannot publish a partially adjusted artifact. Cached `ShapedGlyphRun`
instances are never modified.

Focused deterministic tests cover valid offset-only projection and fail-closed
length, NaN, and overflow sidecars. No canonical fragment currently emits this
sidecar, so the public line baseline, rendering geometry, and cache identity are
unchanged by this foundation step. The next implementation step remains the
composite line-policy builder; managed Cargo, timing/power evidence, and a real
WGPU framebuffer are still pending.

## 2026-08-26 Horizontal Fragment Geometry Ownership

Status: `implementation_complete_static_checked_composite_policy_pending`.

`text/layout/horizontal_line_fragment.rs` is now the private, shared geometry
builder for both `CanonicalPhysicalLineFragment` and
`CanonicalLogicalVirtualLineFragment`. It retains exactly the shaped
`Arc<ShapedGlyphRun>` supplied by the already-authorized request, then derives
the current `TextLineMetrics` and grapheme advances once. The two surrounding
fragment types continue to own their distinct source-range and virtual-anchor
semantics, respectively; they no longer own parallel metric/advance
constructors.

The focused geometry regression asserts pointer identity with the retained
shaped run and equality of its metrics/advances. This is an ownership and
topology check, not a performance result. The builder intentionally does not
yet emit an origin-adjustment vector or publish a composite baseline: that
requires the next line-policy owner to aggregate raw selected-face metrics,
rich runs, and inline objects before the existing projection-sidecar foundation
can be used. No public UI DTO, cache identity, FontDatabase lookup, glyph-run
copy, renderer-local baseline rule, or compatibility facade was introduced.

Scoped formatter parsing and source audits are the only current evidence.
Managed Cargo, capture-only geometry/origin counters, 31-sample profiling and
power data, and a real product WGPU framebuffer saved only under
`docs/tests/runtime/text` remain pending. No image was created by this change.

## 2026-08-26 Virtual BiDi Renderer-Fallback Correction

Status: `implementation_complete_static_checked_validation_pending`.

Review found that a rejected private virtual-sequence BiDi projection could
previously surface `BidiInvariant` as a whole-layout failure. The correction
marks the retained `LogicalVirtualLineSequence` as artifact-ineligible, clears
its canonical fragment and private advances, and leaves the candidate layout
intact for the established renderer fallback. The marker is intentionally kept
instead of dropping the sequence: both initial artifact construction and
font-generation rebuild explicitly refuse logical and visual artifact
projection for that line, including the final-LTR fast path. Canonical metrics
are likewise not applied once the private route is rejected.

An internal collection-cardinality mismatch remains `LayoutFailed`, because it
is an owner-corruption signal rather than text input. The focused regression
locks the retained rejection marker, cleared advances, and unchanged candidate
text. This closes the review finding at source level only. Managed Cargo,
renderer fallback behavior, profile/power evidence, and the required real WGPU
framebuffer are still unverified; no PNG or `target` artifact is used as proof.

## 2026-08-26 Composite Baseline Structural Re-Review And Measurement Gate

Status: `design_reopened_before_policy_implementation`.

This re-review used the local Unreal reference instead of inferring a policy
from Zircon's current sample metrics. In
`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextLayout.cpp`,
`FTextLayout::CreateBreakCandidate` accumulates each participating run's
maximum above and below baseline. `CreateLineViewBlocks` then places every
block using `MaxAboveBaseline - BlockHeight - BlockBaseline`. The decisive
property is that line height and each block's vertical offset are derived from
the same per-run data; a line baseline alone is insufficient.

Zircon already has the analogous consumer boundary in two places:

- `text/layout/rich/materialize.rs` aggregates text and inline ascent/descent,
  then emits `line_baseline - run_ascent` for each text item;
- `text/glyph_artifact/projection.rs` can atomically add an index-aligned
  origin sidecar to `TextGlyph.offset[1]`; and
- `ui/text/layout_engine/artifact.rs` defines `Ready(None)` as a publishable
  visual-only resolved layout, so a rejected private artifact route must not
  become a layout failure.

The current Plain raw sidecar is deliberately only one selected-face envelope
per hard line. It proves the line's content bound but cannot derive a fallback
glyph's own ascent/descent. Looking up every `ShapedGlyph.font_id` again from
the database would introduce a second metric-resolution path, and assigning the
aggregate envelope to every glyph would make mixed-face baseline adjustment
incorrect. Therefore the composite policy must not be implemented from the
current sidecar alone.

The next code cut is now fixed more precisely:

1. During the existing direct horizontal primary/fallback itemization pass,
   retain crate-private, line-local selected-face metric spans aligned to the
   glyph ranges produced by each segment. They must be immutable cache-owned
   provenance, not a public DTO and not a later `FontDatabase` lookup.
2. Add a narrow Plain `horizontal_line_policy` leaf. It consumes the final
   fragment's raw line envelope plus those spans, produces one composite
   `TextLineMetrics`, and emits origin adjustments only after every final
   adjusted offset is finite. A homogeneous one-span line should retain the
   current geometry without an origin-sidecar allocation.
3. Route the resulting optional sidecar through the existing fragment-aware
   artifact projector for both initial construction and generation rebuild.
   Rich runs and inline objects must join the same policy by an explicit later
   input adapter; VerticalRl, tabs, viewport uniform-height selection, and
   virtual renderer fallback remain outside this first Plain cut.
4. Add deterministic fixtures before implementation: a primary/fallback pair
   with distinct ascents, exact adjacent line bottoms, and artifact glyph
   identity/range/advance preservation with only the approved y-offset change.

No performance bottleneck or power conclusion exists yet: managed profiler
capture is unavailable in this session, and no optimization is being made from
an assumed hot loop. Before enabling the policy in a product path, record
capture-only counters for fragment geometry builds, selected-face metric spans,
origin-adjusted glyphs, and artifact origin projections. The required 31-sample
Windows comparison uses the same warmed font corpus before/after the cut and
reports cache hit/miss, p50/p95 layout and artifact times, allocations where the
managed profiler exposes them, and power only from a separately recorded system
trace. A real Native/SDF WGPU framebuffer is then inspected under
`docs/tests/runtime/text`; no screenshot, text-only strategy image, `target`
artifact, or power estimate substitutes for those receipts.

## 2026-08-26 Selected-Face Metric Span Provenance Foundation

Status: `implementation_complete_static_checked_line_policy_pending`.

`ShapedGlyphRun` now retains a crate-private, serde-skipped flat sequence of
`HorizontalGlyphMetricSpan`. Each span carries a direct-shaping hard-line
index, a contiguous glyph range local to that line, and the already-scaled
selected-face `HorizontalLineRawMetrics`. The accessor exposes a line only
when its spans are ordered and cover every glyph exactly once; an empty,
partial, vertical, or malformed sidecar returns `None`. This fail-closed rule
prevents a later policy from approximating a missing fallback face with the
aggregate line envelope.

Direct horizontal shaping fills the spans at the existing
`SelectedFaceLineExtents::include_face` call for each itemized segment. The
method now returns that same face's scaled metrics while continuing to update
the original aggregate envelope, so the change adds neither a second
`FontDatabase` lookup nor a resolver pass. Hard-break synthetic glyphs and
backends without same-pass provenance remain intentionally ineligible for this
future policy. Cosmic fallback and VerticalRl retain empty span sidecars.

The shaped-run cache budget includes span capacity. Model regressions specify
strict line coverage plus serde absence, and the direct-horizontal regression
requires complete span coverage for normal rasterizable text. The completed
Plain line-policy cut consumes this provenance only to reproduce the existing
shared alphabetic `TextLineMetrics`; it does not change `TextGlyph.offset[1]`.
The later rich/inline block-origin adapter remains responsible for any distinct
origin-vector policy.

Current evidence is scoped formatter parsing, source construction-site audit,
cache-capacity audit, and no-production-exception audit only. Managed Cargo,
counter capture, 31-sample performance/power data, and the required real WGPU
framebuffer under `docs/tests/runtime/text` remain pending. No image was
created and no product evidence was written to `target`.

## 2026-08-26 Plain Composite Baseline Policy Correction

Status: `implementation_complete_static_checked_managed_validation_pending`.

The required renderer audit invalidated the preceding assumption that every
Plain fallback span needs a new artifact `offset[1]` adjustment. Both Native
and SDF artifact paths position a horizontal glyph as `line baseline +
TextGlyph.offset[1]`; the latter is already the shaping offset relative to the
glyph's alphabetic baseline. Direct horizontal shaping already resolves the
same composite line baseline from `max(ascent)`, `max(descent)`, and the
primary face line gap. Adding the Slate block-top term to `offset[1]` would
therefore apply an ascent correction twice and move fallback glyphs away from
their common alphabetic baseline.

`text/layout/horizontal_line_policy.rs` is now the narrow Plain policy leaf.
It consumes only complete selected-face span provenance plus the raw line
envelope, verifies that span maxima agree with that envelope, and reproduces
the composite `TextLineMetrics` without allocation. `HorizontalLineFragment`
uses the result only when that validation succeeds; malformed, partial,
vertical, hard-break, or non-direct provenance retains the shaped line's
existing metrics. A deterministic two-face regression proves the shared
baseline and exact line height, while a partial-span regression proves the
fail-closed fallback.

This is consistent with Slate without copying its coordinate formula blindly:
Slate's `VerticalOffset` is a block-top placement because Slate paints blocks
from their local origins; Zircon's Plain artifact renderer paints individual
glyphs from a resolved line baseline. `rich/materialize.rs` remains the
corresponding block-origin owner (`line_baseline - run_ascent`) for varied
styles and inline objects. A future rich/inline adapter may emit artifact
origin sidecars only after it has a renderer contract that consumes block
origins. VerticalRl, virtual renderer fallback, tabs, and document
virtualization are intentionally outside this Plain cut.

The existing artifact y-origin sidecar foundation remains a fail-closed
capability for such a future adapter; this correction does not activate it for
Plain fallback faces and makes no raster-output change by itself. Scoped
formatter parsing, whitespace checks, construction-site auditing, and static
renderer tracing are the current evidence. Managed Cargo, capture-only
profiling, power data, Native/SDF WGPU framebuffer inspection, a PNG under
`docs/tests/runtime/text`, milestone acceptance, commit, and WeCom reporting
remain pending.

## 2026-08-26 Rich And Inline Baseline Contract Audit

Status: `implementation_complete_static_checked_managed_validation_pending`.

The rich/inline re-review found an existing explicit renderer contract rather
than a missing Plain-origin adapter. `rich/materialize.rs` expands the line
above/below extents and publishes the one resolved line baseline. During paint
planning, every `UiTextPaintRun` retains its source range; the renderer resolves
that range back to its `UiResolvedTextLine` and supplies the absolute
`line.frame.y + line.baseline` to both Native and SDF text batches. Their
horizontal raster paths use that resolved baseline plus the run's shaping
offset, so a font-size override shares the correct alphabetic baseline even
though its run frame has the line's top coordinate. Inline images/icons consume
the same `UiResolvedTextLine.baseline` through `inline_layout_frame`.

The new renderer planning regression uses `[size=26]Large[/size] small` and
requires the two materialized Native batches to retain their distinct 26px and
10px presentation sizes while carrying the identical resolved line baseline.
This validates the DTO-to-renderer handoff without inventing a duplicate
baseline field or a per-glyph sidecar. A future rich change is allowed only if
it changes that block-origin contract deliberately; then it must update both
Native and SDF consumers together and add an equivalent source-range regression.

Only static parser/format/whitespace and source-contract evidence exists here.
Managed Cargo, real Native/SDF WGPU output, visual inspection, profiling/power
data, a new PNG solely under `docs/tests/runtime/text`, and milestone acceptance
remain pending.

## 2026-08-26 Tab Physical-Fragment Metric Correction

Status: `implementation_complete_static_checked_managed_validation_pending`.

`PhysicalLineFragments` no longer rejects a source-congruent Plain line merely
because it contains `\t`. It retains the canonical fragment for actual
selected-face metrics, capacity/frame publication, and source-identity artifact
projection. `grapheme_advances_for_layout(...)` separately declines that
fragment's raw advances when a tab is present, leaving the existing pen-position
tab-stop owner as the sole producer of final x advances. This keeps the tab
placement rule intact while removing its unrelated fallback to the `"Hg"`
metric sample.

The focused regression asserts both sides of the boundary: a tab line has an
absolute source-congruent metric fragment input, while its raw fragment
advances are not layout-safe. Scoped Rust 2024 formatting, whitespace checks,
and source-gate tracing are static evidence only. Managed Cargo, real WGPU
output, profiling/power data, a PNG under `docs/tests/runtime/text`, and
milestone acceptance remain pending.

## 2026-08-26 Base Font And Plain-Layout Re-Audit

Status: `audit_complete_no_runtime_change_justified`.

The packaged default-font activation path is present in production, not merely
in test fixtures. `ScreenSpaceUiTextSystem::new` owns the one default asset
load. Without an active project it resolves `res://fonts/default.font.toml`
from the runtime asset root; with an active project it requires that same URI
to resolve through the project registry to its cooked font payload. Both paths
enter `TextRenderState::replace_font_source`, then project the loaded default
family and `CompositeFontDescriptor` at the same private cache boundary. The
accepted Text01 FR-M3 product proof already exercised the project CompositeFont
route, including the packaged zh-Hans face. Replacing this with a system-family
special case or making the optional default record fatal would weaken the
existing asset contract, so no such change is justified.

The Plain overflow review also rejected a suspected zero-height failure:
`visible_line_capacity` deliberately returns at least one candidate line, so
the tail-preserving ellipsis branch cannot index an empty constraint list. The
line is later clipped by its frame as intended. No production
`panic!/unwrap/expect` path was found in the Plain layout error flow; shaping
and font-generation failures continue through `TextShapingOutcome`.

The virtual-projection profiling plan supplies topology counters and a
60-warmup/300-frame matrix, but this session has no managed profiler receipt.
Accordingly no cache-capacity, allocation, or traversal optimization is being
made from static inference. The next allowed performance change must first
record the prescribed CPU p50/p95, cache deltas, allocation/upload data, and a
separate power trace, then compare the same corpus before and after. This audit
does not add visual evidence or alter the requirement that a new real WGPU
framebuffer be written only beneath `docs/tests/runtime/text`.

## 2026-08-26 Plain Viewport Uniform-Metric Admission Correction

Status: `implementation_complete_static_checked_managed_validation_pending`.

The former Plain viewport shortcut selected hard lines from one sampled primary-face
height before it had established the physical metrics of fallback lines. That is unsafe
for a composite-font document: `FontChainLineMetricEnvelope` may certify one total-height
upper contract, but it does not prove the selected-face baseline of every physical line.
Unreal's `FTextLayout::CreateBreakCandidate` instead aggregates above/below-baseline
extents from the actual runs, and its line views retain those physical results. Zircon
must not use a document-wide sample as if it were equivalent run evidence.

`measurement.rs::certified_plain_viewport_line_height` is now the only admission gate for
the existing fixed-height shortcut. It requires the complete unwrapped, clipped,
horizontal Plain source to be covered by the resolved primary face under one stable font
generation. `viewport.rs` consumes an explicit `Option<f32>` certificate; no certificate
falls back to complete physical-line layout. The existing font-owner regression proves
Latin primary coverage succeeds and CJK fallback coverage fails, while the viewport
regression proves an absent certificate cannot select a partial hard-line window.

This is a correctness cut, not the final large-document optimization. The full coverage
scan is exposed as `text.layout/certify_plain_viewport_line_height`, separately from
`select_visible_plain_lines`, so the managed profiler can measure its scaling before any
cache is added. The intended next structural optimization is a session-owned,
font-generation-aware physical line prefix-metrics cache that publishes cumulative
heights and baselines; cache capacity, invalidation, and traversal policy must be chosen
from 1/100/1k/10k-line p50/p95 and allocation evidence rather than static inference.

Production ownership remains bounded: `layout_engine.rs` is 779 lines, `measurement.rs`
187, and `viewport.rs` 298. Rust 2024 formatting, scoped whitespace checks, symbol/call-site
tracing, and file-budget checks pass. Managed Cargo, profiler/power samples, current-source
Native/SDF WGPU output, a real framebuffer PNG under `docs/tests/runtime/text`, milestone
acceptance, commit, and WeCom synchronization remain pending.

## 2026-08-26 Font-Generation Admission And Publication Fence

Status: `implementation_complete_static_checked_managed_validation_pending`.

The viewport re-review exposed a broader epoch hole below the line-height certificate.
`shape_backend_request_at_stable_generation` already shaped within one stable font generation,
but `SharedTextLayoutSession` discarded that generation before cache admission. If the database
changed after shaping, the session could return the retired run as `Ready` while merely skipping
its cache insertion. Parallel paragraph prewarm likewise attached only the request-time generation
to a worker job and unconditionally cached a later `Ready` result.

The canonical service result now retains a crate-private `GenerationTaggedShapedRun` until its
admission owner. Synchronous session admission and parallel worker completion both require the
lookup/job generation, shaped generation, and current shared generation to agree; any mismatch is
`Deferred(FontGenerationChanged)` and cannot enter the shaped cache. The former untagged shaping
helper is test-only; production worker code can no longer bypass tagged admission. No public DTO
or compatibility route was added.

`LayoutFontGenerationFence` then spans the complete UI publication operation. It validates the
epoch after metric/physical-line resolution and again after Plain glyph-artifact or compiled-rich
artifact attachment. A generation transition therefore cannot publish a layout assembled from
old metrics and a new artifact. The request-local retained fragment result moved to the 22-line
`layout_result.rs` child, leaving `layout_engine.rs` at 779 lines; `layout_session.rs` is 646 and
`parallel/shape_pool.rs` 396. Focused stale-ready regressions cover both synchronous and worker
admission plus the final layout fence. Rust 2024 formatting and scoped whitespace/source checks
pass. Managed Cargo, concurrency stress, profiling/power capture, real WGPU framebuffer output,
milestone acceptance, commit, and WeCom synchronization remain pending.

## 2026-08-26 Rich Prepared-Run Performance Baseline Plan

Status: `instrumentation_complete_static_checked_baseline_capture_pending`.

The current rich pipeline has one canonical parser/layout owner but not one prepared glyph owner.
`rich_layout.rs` measures styled spans through `SharedTextLayoutSession`, then projects only text,
ranges, frames, and grapheme advances into `UiResolvedTextLayout`. Paint materialization reconstructs
`UiTextPaintRun`, and the renderer calls the canonical shaping service again for every non-inline
styled run because no glyph artifact is attached to that run. This is the Runtime84 RRT-P1-034
prepared-run gap and the Runtime11C P1-16 stable-text reuse gap; it is not evidence that a cache or
arena implementation is already justified.

The renderer fallback owner now exposes one `text.render/shape_renderer_fallback` span for every
actual canonical service call. Each rich paint command also publishes
`rich_render_fallback_shape_request_count` and `rich_render_fallback_shape_source_bytes` once, so
the profiler can distinguish layout/cache work from render-time repeat work without recording raw
text. A profiling regression locks the static baseline: a Markdown fixture with plain, strong, and
code runs emits one fallback-shape span per materialized text batch. Inline image/widget work is not
counted as a text-run fallback; icon shaping remains visible through the generic renderer span.

The managed Windows baseline must use the same font collection generation, fallback policy,
markup, viewport, raster scale, render mode, and warmup for both sides. Capture 1/100/1k/10k
materialized styled runs for Latin, CJK fallback, Arabic/RTL, mixed BiDi, font-size/family override,
inline image/icon, wrapped paragraph, and clipped stable-frame corpora. Record 60 warmup and 300
measured frames, CPU p50/p95/p99 for `text.layout` and `text.render`, shaped-cache hit/miss deltas,
fallback request/source-byte counts, allocations/RSS, glyph upload bytes, GPU timestamps, and a
separate same-scene power trace. Report cold layout, first paint, and stable repaint separately.

Only measured stable repaint duplication may admit the structural implementation. The intended
direction is one runtime-private composite sidecar containing the compiled rich artifact plus
generation-bound prepared glyph runs. Renderer batches would consume glyph identities projected
from those runs while retaining per-run material/style and inline-object handling; font-generation
refresh must rebuild an entire prepared run under its original resolved style. Do not add a second
serializable glyph DTO, infer glyph IDs from advances, or use a cache that can return a retired
generation. Acceptance requires renderer fallback requests to reach zero for source-congruent
prepared text runs without increasing layout backend misses, while pixel, caret, baseline, BiDi,
inline, Native, and SDF regressions remain equal. No such implementation or performance claim is
made before the managed baseline receipt.

2026-08-30 current-source correction: the premise above has been superseded by the implemented
`ResolvedRichTextArtifact`. Stable rich runs now carry generation-bound glyph slices through a
composite compiled/glyph/layout-run owner; artifact-routed paint does not invoke renderer fallback
shaping. The remaining RRT-P1-034/036 question is serializable layout/paint string residency and
compiled-style range projection. It requires phase-local allocation/timing evidence before changing
the DTO or owner boundary. See
[`../07/2026-08-30-rich-prepared-run-current-source-review.md`](../07/2026-08-30-rich-prepared-run-current-source-review.md).

The runtime follow-up now supplies a fixed paint-projection scope and twelve aggregate topology/byte
counters without changing the serializable DTO. Segment-cache hits report zero new projection work.
The static Runtime Text suite passes 52/52; managed timing/allocation/RSS/power capture still gates any
owner migration.
