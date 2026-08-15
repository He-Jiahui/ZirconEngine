Plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
Milestone: M2
Status: implementation_forward_repaired_static_second_review_complete_coordinator_atomic_staging_required_managed_validation_pending
Files: ["zircon_runtime/src/graphics/scene/scene_renderer/ui/text/fallback_overlay.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback/tests/overlay.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/dpi_product.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_renderer.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_output.rs"]

# Text04 DPI Raster M2 Milestone Manifest

## Scope

This pending manifest covers the native DPI cache-key product gate, its framebuffer proof wiring,
and the SDF fallback metadata repair needed for the same UI text path. It deliberately does not
claim an accepted milestone, a Cargo result, a WGPU result, or a PNG artifact.

## Current Source Evidence

- Scoped `rustfmt --check` passed for the SDF fallback and proof-assertion modules.
- The product root now resolves `proof_assertions/{msdf_pixels,table_pixels}.rs`; its remaining
  rustfmt output is pre-existing shared formatting in the root test and `proof_commands.rs`.
- The SDF fallback overlay now preserves the route generation, physical `raster_scale`, clip,
  paint style, effects, decorations, and transform while clearing source/layout metadata that
  would be invalid after a native re-shape. The focused regression uses non-default scale and
  non-empty shaped/artifact metadata, so this contract is not an empty-value assertion.
- The native DPI product gate retains one renderer and one device-space frame through the 1x to
  2x transition. It requires a source-cache miss at both scales, zero unresolved raster work,
  changed framebuffer coverage and bounds, and distinct RGBA output. This is a real renderer
  proof design, not a text-policy snapshot.
- The independent source review found no remaining P0/P1/P2 in the SDF fallback/overlay or DPI
  proof wiring after the forward repairs.
- The proof writer encodes before atomically replacing the fixed `docs/tests/runtime/text` path;
  the exporter rejects both the repository target and an absolute configured Cargo target. The
  new `runtime_text_mvp_foundation_product_framebuffer_20260801.png` is not present yet.

## Required Managed Gates

- Run the focused runtime text Cargo regressions after the shared `zircon_runtime` compile path is
  available.
- Run the ignored native DPI WGPU framebuffer gate with one renderer, one device-space frame, and
  scale 1x then 2x.
- Run the multilingual framebuffer exporter and retain any real proof PNG only in
  `docs/tests/runtime/text`; reject proof files under `target`.

## Pending Constraints

The DPI render-extract inputs are owned by the active Runtime09 UI session. This manifest requests
only the Text04-owned snapshot and leaves acceptance pending until the coordinator records the
exact managed validation evidence and all applicable Text04 failure handoffs are closed.
