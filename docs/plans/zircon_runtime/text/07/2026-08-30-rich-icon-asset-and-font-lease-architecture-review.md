# Rich icon asset and font lease architecture review (2026-08-30)

## Status

`RRT-P1-028_typed_image_icon_asset_hard_cut_static_complete /
intrinsic_metric_revision_readiness_font_icon_and_managed_validation_pending`

This record now includes the first static implementation slice, but is not an accepted milestone.
No Cargo test, profiler capture, WGPU framebuffer, PNG, RSS, power, commit, or WeCom receipt is
claimed here.

## Current-source finding

Before this slice, `InlineObjectRef::Icon { glyph, font }` stored only a Unicode scalar and a
font-family string. The owner chain was split:

1. `text/layout/rich/metrics.rs` reserves a square from the surrounding text ascent/descent without
   shaping the icon glyph or resolving its face.
2. `text/layout/rich_vertical.rs` repeats an em-square approximation.
3. The compiled dependency closure ignores icon fonts.
4. `render/rich_text.rs` later creates a new text batch with `font = None`, the family string, and no
   resolved glyph artifact. Rendering therefore shapes against whichever collection is current at
   paint time.
5. The default family was the unqualified string `Zircon Icons`; no tracked product icon-font asset
   was found. The product proof inputs also used a system-family glyph before the hard cut.

The reserved advance, final glyph advance, face generation, fallback outcome, and resource readiness
do not share one authority. This is a correctness defect before it is a performance problem.

## Unreal reference and decision

The reviewed Unreal owners are:

- `SlateCore/Public/Textures/SlateIcon.h`: `FSlateIcon` identifies an icon through a style set and
  style name, not an anonymous font-family string.
- `Slate/Private/Framework/Text/TextDecorators.cpp`: the rich image decorator resolves the style
  brush before creating the run.
- `Slate/Public/Framework/Text/SlateImageRun.h` and its implementation: the run owns one resolved
  brush/dynamic-brush lifetime and uses the same image size for measure and paint.

Zircon adopts that ownership rule. The primary rich icon contract must be an icon asset resolved to a
typed image/brush dependency with one intrinsic metric and readiness outcome. A font-backed icon may
exist only as an explicit font-icon asset that names its font asset, face/family selection, glyph,
fallback/alternative text, baseline, and generation lease. The current family-only success path is
not a compatibility contract.

## Implemented static foundation

The anonymous path has been hard-cut rather than retained as a compatibility branch:

1. `RichIconAssetId(ResourceId)` is the public strong identity and `InlineObjectRef::Icon` now owns
   that asset plus explicit size, baseline, and optional alternative text.
2. The built-in BBCode decorator accepts a controlled engine resource locator and rejects the old
   glyph/family form. Resource admission is shared with HTML/BBCode images through the dedicated
   `resource_admission.rs` owner.
3. Horizontal layout, VerticalRl layout, and renderer placement consume the same stored size and
   baseline. The renderer emits a `ScreenSpaceUiImageBatch`; it no longer resolves a family or calls
   text shaping for an icon during paint.
4. `RichTextDependency::IconAsset` enters the sorted/deduplicated compiled dependency closure.
5. Alternative text enters metadata quota, residency accounting, and compiled semantic projection;
   `Some("")` remains the explicit decorative contract.

The focused red/green static contract and the complete Runtime Text infrastructure suite pass 40/40
in the final 0.222 s rerun. The existing grapheme-alignment performance contract remains 3/3 in
0.002 s. Rust
behavior tests cover parser metrics, dependency deduplication, semantic storage, and image-batch
planning, but remain unrun because managed Cargo acquisition is still unavailable.

The multilingual product proof now uses the checked-in `res://ui/rich-inline-checker.png` fixture for
both its BBCode block icon and table-cell icon, with authored `24x24` center-baseline metrics and
alternative text. Its framebuffer gate derives each icon frame from the resolved layout and requires
all four checker color quadrants, so a retained U+FFFC placeholder or solid fallback cannot pass as a
rendered icon. The first managed library-test attempt stopped in a transient third-party `zstd-sys`
`cl.exe` invocation before Runtime compiled; the corrected-source retry was not admitted because the
single CPU lane was reserved for another Session. No Cargo, WGPU, or PNG pass is claimed.

## Current image generation and readiness owner

Current-source review after the cutover found that the shared UI image pipeline already owns the
render-side generation boundary:

1. `ui_texture_ids()` discovers both `ImageTexture` and `IconAsset` dependencies from the compiled
   artifact before scene-resource preparation.
2. `ResourceStreamer` resolves and uploads those resources through the same 2D texture path.
3. `ScreenSpaceUiImagePrepareTextureCache` retains the exact `Arc<ResourceManagementGeneration>`;
   a new generation clears the locator-to-texture resolution cache.
4. Every prepare refreshes the actual `Arc<GpuTextureResource>` and binding. Missing, unloaded, or
   non-2D resources use the shared fallback texture rather than an unqualified stale pointer.

Therefore an authored-size image icon does not need to copy or pin the global resource generation
inside `CompiledRichText`. Parser/cache artifacts may outlive many resource generations, while the
image renderer must consume the current generation at frame preparation. Duplicating that generation
in the text artifact would create a second invalidation owner without improving geometry correctness.

Intrinsic-size icons are different: if asset dimensions are allowed to determine layout, a future
layout admission step must resolve a qualified texture revision/metric snapshot and make a revision
change invalidate the affected layout. That contract is not satisfied by the current authored 16 px
default and must not be inferred from renderer fallback state.

## Remaining implementation slice

The cutover must land as one dependency-ordered change:

1. Split the current image-backed identity into the final resolved icon asset/brush payload if the
   asset service requires region, tint, or atlas metadata; do not restore a family string fallback.
2. Resolve and retain image intrinsic metrics/readiness, with explicit loading/missing/error fallback,
   instead of relying only on author-supplied size.
3. Bind intrinsic metrics to a qualified texture revision and invalidate layout when that revision
   changes; authored-size icons continue to use render preparation's current generation.
4. Measure a future font-backed icon through the canonical shaper and retain its font asset/face,
   glyph, fallback, and collection revision lease.
5. Store the admitted metric/glyph result in the same layout artifact consumed by paint.
6. Make renderer readiness fail closed to the declared fallback; renderer must not resolve a new
   family or reshape after layout.
7. Add qualified accessibility child identity/action while keeping current alternative/decorative
   text as the fallback projection.
8. Validate horizontal/VerticalRl wrap, fallback, unload/reload generation, Native/SDF parity, real
   WGPU pixels, and missing/error outcomes.

Until these items share one owner chain, changing the square heuristic or adding a cache would only
optimize an invalid result. Dynamic optimization work remains profile-gated after correctness lands.

## Work that can continue independently

Parser diagnostics, typed link metadata, semantic projections, table geometry admission, and other
non-icon infrastructure do not depend on this asset schema and should continue without waiting on
the icon cutover or acceptance queue.
