---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
plan_sources:
  - user: 2026-07-05 shader cubemap validation continuation; current workspace integration-test compile unblock
  - user: 2026-07-10 runtime text architecture continuation; scroll raster/upload counters
tests:
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - cargo test -p zircon_runtime text_prepare_report_exposes_raster_upload_scroll_counters --lib --no-default-features --locked --jobs 1 (2026-07-10 blocked before compile by zircon_runtime/Cargo.toml and Cargo.lock mismatch)
doc_type: module-detail
---

# Native Bitmap Atlas Source Cache

## Purpose

The native bitmap atlas source cache stores swash glyph images across UI text frames before those images are converted into Zircon bitmap-atlas upload sources. It prevents repeated worker raster requests for canonical `GlyphRasterKey` identities reused by the same screen-space UI text backend.

This document exists because the current workspace already had an in-progress UI text cache change while the shader cubemap integration tests were being validated. The cache type needed to be visible to the parent `text.rs` backend that owns the persistent `bitmap_source_cache` field.

## Behavior Model

`TextRenderState` owns one `NativeBitmapAtlasSourceCache`. Each frame, `native_bitmap_atlas_frame(...)` calls `begin_frame`, drains face-epoch-compatible worker completions, looks up prepared `GlyphRasterKey` images, requests misses through the bounded raster pool, then records hit, miss, insert, eviction, and final entry counts through `NativeBitmapAtlasSourceCacheFrameReport`.

The cache uses an indexed intrusive LRU: hit, touch, insert, and least-recent eviction are amortized O(1). Entry count and CPU-byte hard caps are enforced together; a budget eviction returns the linked persistent raster key so the atlas/page owner can invalidate the matching slot rather than leaving stale residency.

The current native bitmap atlas path uses `GlyphRasterKey` directly for cache lookup, worker request registration, pending checks, and insertion. Its horizontal and vertical phase bins are part of the text-owned physical raster identity; approximate reuse probes at most three alternative vertical bins and never scans the cache.

`ScreenSpaceUiTextPrepareReport.raster_upload` now consumes this frame report instead of forcing higher-level perf tests to inspect `NativeBitmapAtlasPrepareReport` and renderer upload reports separately. The aggregate report keeps source-cache hit/miss/insert, approximate hit, worker request submitted/pending/unavailable, visible/source/missing/approx glyph, submission upload byte, and renderer upload/requeue/failure counters in one renderer-local DTO.

## Visibility Contract

`NativeBitmapAtlasSourceCacheFrameReport` and `NativeBitmapAtlasSourceCache` are crate-internal text renderer contracts. The actual cached glyph image, worker font snapshots, and cache-entry internals remain narrower implementation details of `native_bitmap_atlas/source_cache.rs`.

This visibility is intentionally not a public graphics API. It allows `TextRenderState` to own the cache and the screen-space text module to aggregate frame-report counters into `ScreenSpaceUiTextRasterUploadReport`, while keeping glyph image data and insertion helpers local to the native bitmap atlas implementation and tests.

## Test Coverage

The shader cubemap validation run exposed this boundary because integration tests compile the runtime library through the normal `text.rs` path. After the visibility fix, these focused commands passed:

```powershell
cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never
```

The change is a compile-boundary correction only. It does not alter UI layout, glyph rasterization policy, renderer color output, or any shader/cubemap behavior.

The 2026-07-10 raster/upload report slice added a focused regression test, `text_prepare_report_exposes_raster_upload_scroll_counters`, plus scoped rustfmt and diff-check evidence under `docs/tests/runtime/text/`. The focused Cargo command could not compile because the current workspace has a pre-existing `zircon_runtime/Cargo.toml` / `Cargo.lock` mismatch while `--locked` is required; that blocker is recorded in `docs/tests/runtime/text/runtime_text_raster_upload_report_cargo_blocker_manifest_lock_20260710.log`.
