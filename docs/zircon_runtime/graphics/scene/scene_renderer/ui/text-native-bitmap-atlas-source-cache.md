---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
plan_sources:
  - user: 2026-07-05 shader cubemap validation continuation; current workspace integration-test compile unblock
  - user: 2026-07-10 runtime text architecture continuation; scroll raster/upload counters
tests:
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - cargo test -p zircon_runtime text_prepare_report_exposes_raster_upload_scroll_counters --lib --no-default-features --locked --jobs 1 (2026-07-10 blocked before compile by zircon_runtime/Cargo.toml and Cargo.lock mismatch)
doc_type: module-detail
---

# Native Bitmap Atlas Source Cache

## Purpose

The native bitmap atlas source cache stores swash glyph images across UI text frames before those images are converted into Zircon bitmap-atlas upload sources. It prevents repeated `SwashCache::get_image_uncached` calls for glyphs that are reused by the same screen-space UI text backend.

This document exists because the current workspace already had an in-progress UI text cache change while the shader cubemap integration tests were being validated. The cache type needed to be visible to the parent `text.rs` backend that owns the persistent `bitmap_source_cache` field.

## Behavior Model

`ScreenSpaceUiTextBackend` owns one `NativeBitmapAtlasSourceCache`. Each frame, `native_bitmap_atlas_frame(...)` calls `begin_frame`, requests glyph images through `image(...)`, then records hit, miss, insert, eviction, and final entry counts through `NativeBitmapAtlasSourceCacheFrameReport`.

The cache is LRU-like: every lookup increments a monotonic tick and stores the last-used tick on each entry. When capacity is full, inserting a new cache key evicts the entry with the oldest tick.

The current native bitmap atlas path also normalizes horizontal glyphon subpixel buckets before cache lookup, worker request registration, pending checks, and insertion. `native_bitmap_atlas_stable_raster_cache_key(...)` clears `CacheKey.x_bin` to `SubpixelBin::Zero` so the same glyph does not keep producing new source images or worker requests when only horizontal placement phase changes. `y_bin` remains part of the key because vertical bucket replacement still uses it for conservative approximate reuse.

`ScreenSpaceUiTextPrepareReport.raster_upload` now consumes this frame report instead of forcing higher-level perf tests to inspect `NativeBitmapAtlasPrepareReport` and renderer upload reports separately. The aggregate report keeps source-cache hit/miss/insert, approximate hit, worker request submitted/pending/unavailable, visible/source/missing/approx glyph, submission upload byte, and renderer upload/requeue/failure counters in one renderer-local DTO.

## Visibility Contract

`NativeBitmapAtlasSourceCacheFrameReport` and `NativeBitmapAtlasSourceCache` are visible within `crate::graphics::scene::scene_renderer::ui`. The actual cached glyph image and cache-entry internals remain narrower implementation details of `native_bitmap_atlas/source_cache.rs`.

This visibility is intentionally not a public graphics API. It allows the `text` module to store the cache in `ScreenSpaceUiTextBackend` and to aggregate frame-report counters into `ScreenSpaceUiTextRasterUploadReport`, while keeping the glyph image data and insertion helpers local to the native bitmap atlas implementation and tests.

## Test Coverage

The shader cubemap validation run exposed this boundary because integration tests compile the runtime library through the normal `text.rs` path. After the visibility fix, these focused commands passed:

```powershell
cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never
```

The change is a compile-boundary correction only. It does not alter UI layout, glyph rasterization policy, renderer color output, or any shader/cubemap behavior.

The 2026-07-10 raster/upload report slice added a focused regression test, `text_prepare_report_exposes_raster_upload_scroll_counters`, plus scoped rustfmt and diff-check evidence under `docs/tests/runtime/text/`. The focused Cargo command could not compile because the current workspace has a pre-existing `zircon_runtime/Cargo.toml` / `Cargo.lock` mismatch while `--locked` is required; that blocker is recorded in `docs/tests/runtime/text/runtime_text_raster_upload_report_cargo_blocker_manifest_lock_20260710.log`.
