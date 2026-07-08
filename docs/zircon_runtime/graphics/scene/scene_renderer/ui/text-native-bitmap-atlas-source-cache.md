---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs
plan_sources:
  - user: 2026-07-05 shader cubemap validation continuation; current workspace integration-test compile unblock
tests:
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked
doc_type: module-detail
---

# Native Bitmap Atlas Source Cache

## Purpose

The native bitmap atlas source cache stores swash glyph images across UI text frames before those images are converted into Zircon bitmap-atlas upload sources. It prevents repeated `SwashCache::get_image_uncached` calls for glyphs that are reused by the same screen-space UI text backend.

This document exists because the current workspace already had an in-progress UI text cache change while the shader cubemap integration tests were being validated. The cache type needed to be visible to the parent `text.rs` backend that owns the persistent `bitmap_source_cache` field.

## Behavior Model

`ScreenSpaceUiTextBackend` owns one `NativeBitmapAtlasSourceCache`. Each frame, `native_bitmap_atlas_frame(...)` calls `begin_frame`, requests glyph images through `image(...)`, then records hit, miss, insert, eviction, and final entry counts through `NativeBitmapAtlasSourceCacheFrameReport`.

The cache is LRU-like: every lookup increments a monotonic tick and stores the last-used tick on each entry. When capacity is full, inserting a new cache key evicts the entry with the oldest tick.

## Visibility Contract

`NativeBitmapAtlasSourceCacheFrameReport` and `NativeBitmapAtlasSourceCache` are visible within `crate::graphics::scene::scene_renderer::ui`. The actual cached glyph image and cache-entry internals remain narrower implementation details of `native_bitmap_atlas/source_cache.rs`.

This visibility is intentionally not a public graphics API. It only allows the `text` module to store the cache in `ScreenSpaceUiTextBackend` while keeping the glyph image data and insertion helpers local to the native bitmap atlas implementation and tests.

## Test Coverage

The shader cubemap validation run exposed this boundary because integration tests compile the runtime library through the normal `text.rs` path. After the visibility fix, these focused commands passed:

```powershell
cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never
```

The change is a compile-boundary correction only. It does not alter UI layout, glyph rasterization policy, renderer color output, or any shader/cubemap behavior.
