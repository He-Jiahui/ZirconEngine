# Rich style shaping projection and letter-spacing review

Date: 2026-08-30

Status: `RRT-P1-023_italic_and_feature_projection_static_complete /
letter_spacing_architecture_review_complete /
letter_spacing_release_profile_harness_static_implemented /
managed_31_sample_baseline_pending / letter_spacing_implementation_not_started /
managed_product_validation_pending`

## Scope

This record splits RRT-P1-023 into two dependency-ordered slices. The first closes the missing
italic and OpenType-feature projection from a compiled rich run into font selection, shaping, and
shaped-cache identity. The second reviews letter-spacing as an inline-axis cluster algorithm before
implementation. It does not claim that letter-spacing, rendered output, package power, or the whole
Text07 milestone is complete.

## Current-source defect review

`StyleOverride` already carried `italic`, `letter_spacing`, and `features`, but
`resolve_rich_run_style` projected only weight, size, and family. `TextStyle` had no italic or feature
identity, `font_query_for_text_style` always requested `FontStyle::Normal`, both
`BackendShapeRequest` constructors started with an empty feature list, the shaped cache did not
distinguish italic, and Cosmic fallback attributes did not select italic. This produced a semantic
artifact mismatch: markup retained the requested style while shaping, cache reuse, and glyph
selection behaved as if the request were normal with default OpenType features.

Rich layout already has the correct ownership boundary. `source_spans` resolves each parser run into
one `TextStyle`, coalesces adjacent equal non-inline styles, and sends that span to the canonical shape
provider. The repair therefore belongs in `TextStyle` and `BackendShapeRequest`, not in renderer
paint code or a second rich-only shaper.

## Implemented italic and feature slice

- `TextStyle` now carries `italic: bool` and immutable `Arc<[OpenTypeFeature]>`; defaults remain
  normal and feature-empty.
- Rich style resolution applies explicit true/false italic overrides and run feature lists.
- Horizontal and vertical backend requests inherit style features before canonical normalization;
  an explicit request-level `with_features` remains the public service override.
- Canonical feature normalization now retains one value per four-byte tag, applies last-declaration
  precedence, and emits tags in stable order. Cache identity and backend execution therefore consume
  the same conflict-free list.
- Font queries select `FontStyle::Italic` for italic runs. The neutral service also maps
  `TextFontRequest::italic` into the same backend style.
- The shaped-cache exact key, direction-alias fingerprint, equality, and hash include italic. Feature
  identity continues to use the canonical normalized request slice.
- Cosmic fallback maps italic to `cosmic_text::Style::Italic`; direct RustyBuzz shaping observes the
  italic face selected by the shared font query.

This adds one immutable feature owner per resolved style, not a per-glyph field or renderer copy.
No synthetic-oblique claim is made: the current slice selects a real italic face when the font
database can resolve one.

## Unreal and backend reference review for tracking

Local Unreal `SlateTextShaper.cpp` treats tracking as shaping geometry, not paint decoration:

- it scales `LetterSpacing * font size / 1000`;
- nonzero tracking disables the standard `liga` feature before HarfBuzz shaping;
- spacing is applied to the previous glyph when a next glyph exists, so it is an inter-glyph gap and
  does not add a trailing run-end gap;
- the HarfBuzz path bypasses tracking for right-to-left text and selected unsupported input cases;
- substitute whitespace and ellipsis glyphs receive the same spacing policy.

`cosmic-text 0.18.2` exposes `Attrs::letter_spacing`, but its current shape path adds the supplied
value to every shaped glyph's `x_advance`, including the last glyph. Using it directly would create a
different line width, wrap boundary, and trailing-edge contract from Unreal. Zircon's direct
RustyBuzz path has no tracking input at all, so direct and fallback output would also diverge.

Godot's advanced text server retains glyph spacing as part of its font-variation/cache identity and
applies it while shaping/layout computes advances. That supports the same ownership conclusion: the
value must participate in shaped identity and neutral geometry, not be patched into draw batches.

## Required Zircon letter-spacing design

The implementation must use one backend-neutral tracking policy after backend cluster formation and
before line measurement/artifact publication:

1. Add a finite, unit-explicit tracking value to `TextStyle` and the shaped request/cache identity.
   Text07 currently describes logical pixels; no percent/em reinterpretation is allowed without a
   public contract migration.
2. Resolve effective OpenType features once. Duplicate tags now use implemented last-declaration
   precedence before cache lookup and backend execution. Nonzero tracking must append its forced
   `liga=0` after user features and then canonicalize, producing exactly one disabled `liga` entry.
3. Do not call Cosmic's per-glyph `letter_spacing` shortcut. Direct and fallback results must enter
   the same neutral cluster-gap function.
4. For supported horizontal LTR spans, charge a gap only between adjacent shaped clusters by adding
   it to the preceding cluster's terminal advance. Do not add a trailing span or line gap. Adjacent
   parser runs already coalesce when the complete resolved style is equal.
5. RTL, mixed-direction boundaries, combining/ZWJ clusters, discretionary replacement glyphs,
   inline objects, and vertical upright/sideways runs require explicit golden geometry. Unsupported
   cases must preserve zero tracking rather than publish backend-dependent geometry.
6. Current layout assumes non-negative cluster advances. Negative tracking therefore needs either a
   signed-overlap geometry contract across measure, caret, selection, hit-test, wrap, and artifact
   projection, or typed admission. Silent clamp is not acceptable.
7. The final shaped run is the single authority consumed by measurement, wrap, glyph artifacts,
   renderer batches, caret, and hit-test. No rich-only width correction or renderer offset pass may
   coexist with it.

## Profiling and validation gate before implementation

Run an E-drive release harness with 31 isolated samples per lane. Use 32, 256, and 4,096 grapheme
clusters for Latin ligatures, CJK, combining marks, emoji ZWJ, Arabic RTL, mixed bidi, and
horizontal/vertical writing. Compare zero and nonzero tracking across direct and Cosmic fallback,
cold and warm shaped-cache paths, and single-style versus alternating rich spans.

Record p50/p95/p99 request time, backend shape calls, glyph/cluster counts, allocations/requested
bytes, shaped-cache hits/misses, measured width, wrap line count, and first-sample working-set delta.
The target algorithm is `O(glyphs + clusters)` with no second segmentation pass and no per-glyph
retained tracking payload. A latency, RSS, power, or Unreal-parity claim requires matched product
capture after the isolated harness.

The release-only harness is now present under `text/shaping/tests`. It builds exact 32/256/4,096
grapheme fixtures outside the timed region, records 31 raw samples plus p50/p95/p99 and working-set
deltas, captures direct backend-call counters and actual direct/alternate/hybrid receipts, and
compares single-style with bounded alternating spans. Its test-only candidate disables `liga` and
uses one `cluster_start` traversal, with no second segmentation in the candidate transform and no
retained per-glyph tracking field. It also records the current zero-tracking cache cold/warm report.

This isolated harness does not replace the existing
`graphics/tests/render_profiling/text_baseline/multilingual_text.rs` product owner. That owner keeps
the 60-frame warm-up, 300-frame WGPU/GPU-timestamp, CJK/Arabic/rich/VerticalRl and renderer-cache
baseline. The shaping harness answers only the candidate algorithm's cluster-scale question; final
latency, GPU time, RSS and power comparison must return to the existing product baseline after
tracking is admitted.

The harness deliberately reports `candidate_cache_identity_supported=false`: current cache identity
can distinguish the forced `liga=0` feature from zero tracking but cannot distinguish two nonzero
tracking magnitudes. Candidate cache comparison therefore cannot be admitted before the production
identity exists. Likewise, the forced Cosmic fallback lane remains pending because current source
has no legal fault-injection selector; the harness reports the backend route actually used rather
than presenting a normal direct route as fallback evidence. Allocation count is reported as
unavailable while requested bytes and estimated glyph payload bytes remain explicit. These gaps keep
the managed baseline gate open and prevent the static harness from being mistaken for profile data.

Required Rust behavior tests include italic face query, request-to-style projection, feature
normalization/cache separation, Cosmic italic attributes, `fi` ligature suppression under tracking,
no trailing gap, run coalescing, line-wrap threshold, fallback parity, RTL bypass, cluster safety,
vertical policy, and cache separation by tracking value. Real WGPU text must be captured under
`docs/tests/runtime/text`, never under `target`.

## Evidence and remaining gates

The new cross-layer static contracts failed before implementation and now pass as part of the
47/47 Runtime Text static suite, including conflicting same-tag last-wins and idempotence coverage.
Targeted Rust 2021 formatting and scoped `git diff --check` pass.
The dependency source confirms the used `cosmic-text 0.18.2` italic API. Rust behavior tests are
written but unrun because the Windows validator timed out acquiring its managed Cargo lane before
Cargo/rustc started; the lane is not being polled or bypassed.

The letter-spacing release harness is statically implemented but has not run under managed Cargo, so
no timing, RSS, route, cache, wrap, or allocation result is claimed. Production letter-spacing code
has not started. Managed Cargo, a legal forced-fallback fixture, real font-face behavior, WGPU
framebuffer/PNG, matched Unreal workload, RSS, and package power remain open, so RRT-P1-023 and
Text07 remain `in_progress`.
