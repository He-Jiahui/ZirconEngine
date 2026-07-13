---
related_code:
  - zircon_runtime/src/graphics/text/atlas/mod.rs
  - zircon_runtime/src/graphics/text/atlas/page.rs
  - zircon_runtime/src/graphics/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/graphics/text/atlas/page_residency.rs
  - zircon_runtime/src/graphics/text/atlas/dirty.rs
  - zircon_runtime/src/graphics/text/atlas/upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/atlas_resources.rs
plan_sources:
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/superpowers/plans/2026-07-13-runtime-msdf-mtsdf-dynamic-pipeline.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload/tests.rs
doc_type: module-detail
---

# Runtime text atlas ownership

## Boundary

`graphics/text/atlas/` is the renderer-neutral authority for atlas format/page identity, storage format, shelf allocation, page residency and eviction, dirty rectangles, and upload command math. Renderer adapters may allocate GPU textures and execute commands, but they must not duplicate page stride, rect offset, or residency policy.

Distance-field raster modes map into this shared family as follows:

- SDF uses `GlyphAtlasFormat::Sdf` with `R8Unorm` storage.
- MSDF and MTSDF use `GlyphAtlasFormat::Msdf` with `Rgba8Unorm` storage.
- MSDF and MTSDF share a storage family but remain distinct glyph cache keys through `SdfBakeParams.mode`.

Layout identity never includes atlas format or raster mode. The atlas consumes already resolved face/glyph identities and must not reshape, relayout, or retry a paragraph with different metrics.

## Dynamic distance-field pages

`scene_renderer/ui/sdf_atlas.rs` uses one shelf-allocator collection per atlas format. A page key is `(format, page_index)`; page limits, LRU decisions, rebuilt-page invalidation, and allocation failures therefore remain format-aware. `distance_field_atlas_page_keys` returns a stable sorted sequence used by both bake packing and upload offset calculation.

`sdf_font_bake.rs` flattens pages into one source byte stream and records `{ page_key, source_offset, byte_len }` for each page. R8 page length is `width × height`; RGBA page length is `width × height × 4`. Typed glyph-generation failures stay separate from page-limit and oversized-slot failures.

`sdf_upload.rs` maps full pages and dirty rectangles through the generic atlas upload owner. For every command, bytes-per-row and local source offset use the page storage format, then the cumulative preceding-page byte length is added. This is required when SDF page 0 and MSDF page 0 coexist: identical page indices do not imply identical source layers or byte strides.

## GPU consumption

`sdf_render/atlas_resources.rs` allocates storage-compatible WGPU texture arrays rather than reinterpreting R8 data as RGBA. Upload commands select the texture family from `page_key.format`; vertex `page_index` remains local to that family. The shader's flat decode mode selects scalar or median decode without changing UV placement.

## Validation status

Current production `graphics` and `target-client` checks pass. The real WGPU multilingual product proof successfully generated, uploaded, sampled, and displayed both SDF and MSDF pages. Focused mixed-page byte-length, cache-reuse, dirty-row-stride, source-offset, typed-fallback, and shader/vertex tests are authored; their monolithic lib-test execution is temporarily blocked by unrelated runtime-plugin test API drift and therefore is not yet recorded as accepted.
