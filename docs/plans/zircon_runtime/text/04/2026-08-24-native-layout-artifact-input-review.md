---
kind: architecture_review
status: implementation_complete_static_validation_complete_runtime_validation_pending
origin_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
related_plan: docs/plans/zircon_runtime/text/04/2026-08-24-mixed-storage-frame-plan-and-profiling.md
---

# Text04 Native Layout Artifact Input Review

## Finding

The normal plain-text path is currently:

`UiResolvedTextLayout -> ResolvedTextGlyphArtifact -> ScreenSpaceUiTextBatch`.

`ResolvedTextGlyphArtifactLine` already owns the canonical glyph identity, face and
instance handles, visual-order advances, offsets, and the resolved-line baseline.
However, `ScreenSpaceUiTextBatch::requires_sdf_layout_fidelity` currently routes
every batch with an artifact to SDF before `ScreenSpaceUiTextBackend::prepare`.
The native bitmap projector only reads `shaped_glyphs`, so the normal layout path
cannot reach the native atlas. The ignored CJK product scene currently avoids that
route by constructing a fresh shaped batch, which is not sufficient acceptance for
Text03 to Text04 integration.

## Evidence And Reference Alignment

- `ui/text/layout_engine.rs` builds and attaches the plain-text artifact after
  layout. It is the single owner of the shaped result.
- `ui/render/resolved_layout.rs` preserves the artifact on the planned text batch.
- `ui/sdf_render/artifact_vertices.rs` consumes the artifact directly, including its
  line baseline and glyph offsets, without reshaping.
- `ui/text/native_glyph_run.rs` currently consumes only a renderer-local shaped
  batch, despite the artifact exposing the same key inputs.
- Unreal Slate's `FShapedGlyphSequence` is retained by layout/cache owners and is
  consumed by the element batcher; it is not re-shaped by each raster backend.

## Implemented Hard Cut

1. The native glyph-run projection selects the artifact glyph slice when one
   exists, otherwise retain its existing canonical shaped-batch path.
2. The artifact's existing `(font_face, font_instance)` handle pairs resolve in one
   batch, just as the shaped-batch path does. Do not construct text, grapheme, or
   layout buffers in the native backend.
3. The artifact line baseline and glyph offsets use the same horizontal cursor
   semantics as the SDF artifact renderer. Native support remains horizontal only.
4. Artifact-backed horizontal batches route to native when the requested render mode
   is `Native` or resolves to native. Keep vertical writing, visual fallback lines,
   and distance-field effects on SDF.
5. The CJK product proof enters via `layout_text -> UiResolvedTextLayout ->
   renderer planner -> native_bitmap_atlas_glyph_runs`; it must not construct a
   `ScreenSpaceUiTextBatch` directly.

## Complexity And Invariants

For a batch of `G` glyphs, the projection retains one `O(G)` pass to form font-handle
pairs, one batched registry resolution, and one `O(G)` atlas-key projection. It adds
no text re-shaping pass, no per-glyph registry lookup, and no glyph-cache owner. The
existing handle resolver's pair de-duplication remains the only registry lookup
authority. Native frame ownership and the Text04 `O(F + B)` upload dispatch are
unchanged.

The correction is an input-contract repair, not a performance result. CPU timing,
WGPU timestamps, RenderDoc, power data, and the real framebuffer PNG remain pending
managed validation.

## Completed Static Gates

- `rustfmt --check` accepts the changed Rust sources.
- `git diff --check` reports no whitespace error in the scoped change set.
- The CJK product-source contract rejects direct `ScreenSpaceUiTextBatch` construction,
  renderer-local shaping, and retired glyphon inputs.
- Renderer planning regressions cover CJK wrapping and horizontal RTL artifact routing
  into native batches while preserving the original artifact.

No Cargo, WGPU, screenshot, timing, GPU, or power claim follows from these gates.

## 2026-08-26 Artifact Fallback Cross-Path Re-Audit

Status: `audit_complete_no_runtime_change_justified`.

The Native and SDF artifact projectors consume the same `TextGlyph` sequence,
advance order, line baseline, and per-glyph offsets. The Native path resolves
the artifact's face/instance handles in one batch and never calls the
renderer-local shaping adapter when an artifact is present. On a font
generation change it rebuilds the preserved canonical artifact line, rather
than re-shaping its rendered visual string.

The SDF-atlas failure path has an explicit artifact guard:
`has_shaped_glyph_geometry` treats `glyph_artifact_line` as shaped geometry.
An artifact-backed failed run therefore rejects character/grapheme-based local
Native overlay splitting and takes the existing whole-line Native fallback.
That whole-line fallback is safe because `native_bitmap_atlas_glyph_runs`
projects the artifact directly. This preserves ligatures, RTL visual order,
offsets, and layout advances across a raster failure without making the old
`shaped_glyphs` field a second artifact owner.

This is a source-contract re-audit, not runtime proof. Managed Cargo, real
WGPU framebuffer inspection under `docs/tests/runtime/text`, and the measured
CPU/GPU/power protocol remain pending; no PNG or timing result was created.
