---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/retry_frame.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/retry_frame.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs zircon_runtime/src/text/native_bitmap_atlas.rs zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs zircon_runtime/src/text/native_bitmap_atlas/tests.rs zircon_runtime/src/text/native_bitmap_atlas/tests/retry_frame.rs (2026-07-05: passed)
  - docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_frame_state_preview_20260705.png (SHA256 4E4F6035CE84D6501DCF272D59F50156193D608EC1C3BFCE7365AEE4A8071041)
  - docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_frame_state_validation_20260705.log (SHA256 87B2631FE1AC2C87433F548FF97D75FF0AE75979BD4590B7F39F99B1E5C983AB)
status: in_progress
---

# Native Bitmap Atlas Retry Frame

The native bitmap atlas retry frame owner connects atlas blocked-glyph retry data to the production screen-space UI text backend.

`ScreenSpaceUiTextBackend` owns one `GlyphAtlasBitmapRetryFrameState` beside the native bitmap source cache and frame index. The state is cleared when font faces are invalidated and when a prepare frame has no native text, so old font/source data is not retried after the visible text set changes materially.

`native_bitmap_atlas/retry_frame.rs` owns the per-frame handoff:

- select queued blocked glyphs only when their `GlyphAtlasBitmapSource` is still visible in the current native frame;
- call the shared retry frame driver with the current page, viewport, and clip configuration;
- remap source images and bytes into the retry-aware submission input order;
- drop stale queued blocked sources before upload/draw handoff.

This keeps source-origin matching and byte remapping out of `text.rs` and prevents `native_bitmap_atlas.rs` from becoming a retry-state owner. `NativeBitmapAtlasPrepareReport` carries retry submission and committed state reports so diagnostics can distinguish retried, newly submitted, deferred, and still-blocked sources.

Open follow-ups remain: face validity requeue, global atlas slot invalidation, async raster worker integration, complete glyphon `TextAtlas` cutover, focused Cargo green rerun, and live editor-window typography QA.
