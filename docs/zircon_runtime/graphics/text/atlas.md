---
related_code:
  - zircon_runtime/src/text/atlas/mod.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/atlas_resources.rs
plan_sources:
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/superpowers/plans/2026-07-13-runtime-msdf-mtsdf-dynamic-pipeline.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests
  - zircon_runtime/src/text/sdf/font_bake/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload/tests.rs
doc_type: module-detail
---

# Runtime text atlas ownership

## Boundary

`text/atlas/` is the renderer-neutral authority for atlas format/page identity, storage format, shelf allocation, page residency and eviction, dirty rectangles, and upload command math. `TextRenderState` additionally owns CPU bitmap/SDF cache lifetime and produces raster pixels plus metrics in batches. Renderer adapters may allocate GPU textures and execute commands, but they must not duplicate CPU font state, page stride, rect offset, or residency policy.

Distance-field raster modes map into this shared family as follows:

- SDF uses `GlyphAtlasFormat::Sdf` with `R8Unorm` storage.
- MSDF and MTSDF use `GlyphAtlasFormat::Msdf` with `Rgba8Unorm` storage.
- MSDF and MTSDF share a storage family but remain distinct glyph cache keys through `SdfBakeParams.mode`.

Layout identity never includes atlas format or raster mode. The atlas consumes already resolved face/glyph identities and must not reshape, relayout, or retry a paragraph with different metrics.

## Dynamic distance-field pages

`scene_renderer/ui/sdf_atlas.rs` uses one shelf-allocator collection per atlas format. A page key is `(format, page_index)`; page limits, LRU decisions, rebuilt-page invalidation, and allocation failures therefore remain format-aware. `distance_field_atlas_page_keys` returns a stable sorted sequence used by both bake packing and upload offset calculation.

`text/sdf/font_bake.rs` flattens pages into one source byte stream and records `{ page_key, source_offset, byte_len }` for each page. Its atlas keys retain `TextFontFaceHandle { index, generation }`; raw backend IDs are never reconstructed from numeric fields. R8 page length is `width × height`; RGBA page length is `width × height × 4`. Typed glyph-generation failures stay separate from page-limit and oversized-slot failures.

`sdf_upload.rs` maps full pages and dirty rectangles through the generic atlas upload owner. For every command, bytes-per-row and local source offset use the page storage format, then the cumulative preceding-page byte length is added. This is required when SDF page 0 and MSDF page 0 coexist: identical page indices do not imply identical source layers or byte strides.

## GPU consumption

`sdf_render/atlas_resources.rs` allocates storage-compatible WGPU texture arrays rather than reinterpreting R8 data as RGBA. Upload commands select the texture family from `page_key.format`; vertex `page_index` remains local to that family. The shader's flat decode mode selects scalar or median decode without changing UV placement.

## Validation status

Frameworks05 M3 is still `in_progress`. Current production compile and architecture gates have passed earlier iterations, while the final post-review Rust filters, product framebuffer, editor-host proof, and independent `Critical/Important 0/0` review remain required before acceptance. This document records the current hard-cut owner only; deleted `graphics/text` and Graphics-local SDF bake paths are not compatibility surfaces.
