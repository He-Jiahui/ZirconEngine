---
related_code:
  - zircon_runtime/src/core/framework/render/text/font/face.rs
  - zircon_runtime/src/core/framework/render/text/font/database.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/graphics/text/font/instance.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/shaping/horizontal
  - zircon_runtime/src/graphics/text/shaping/vertical/backend.rs
  - zircon_runtime/src/graphics/text/raster/swash
  - zircon_runtime/src/graphics/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs
implementation_files:
  - zircon_runtime/src/graphics/text/font/instance.rs
  - zircon_runtime/src/graphics/text/font/instance/tests.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/font/database/tests.rs
  - zircon_runtime/src/graphics/text/shaping/horizontal/backend.rs
  - zircon_runtime/src/graphics/text/shaping/horizontal/projection.rs
  - zircon_runtime/src/graphics/text/shaping/horizontal/tests.rs
  - zircon_runtime/src/graphics/text/shaping/vertical/backend.rs
  - zircon_runtime/src/graphics/text/raster/swash/request.rs
  - zircon_runtime/src/graphics/text/raster/swash/rasterizer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/graphics/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/graphics/text/sdf/offline/identity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/distance_field.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/offline_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - text_font_instance_roundtrips_canonical_face_and_coordinates
  - text_font_instance_identity_normalizes_order_duplicates_and_negative_zero
  - text_font_effective_variations_clamp_axes_and_drop_default_coordinates
  - text_font_effective_variations_drop_axes_not_exposed_by_static_face
  - text_font_database_effective_variations_merge_descriptor_axes_and_ui_weight
  - text_font_database_instance_identity_quantizes_real_axis_to_f2dot14_bucket
  - text_font_database_asset_key_deduplicates_same_f2dot14_instance_bucket
  - text_horizontal_backend_skips_vertical_requests
  - text_horizontal_backend_skips_static_face_for_empty_language_tag
  - text_horizontal_rustybuzz_backend_applies_real_variable_width_axis
  - text_horizontal_rustybuzz_backend_applies_real_per_run_locl_language
  - text_msdf_dynamic_generation_applies_real_variable_width_axis
  - sdf_atlas_plan_separates_variable_font_instances_on_same_face
doc_type: module-detail
status: in_progress
---

# Runtime variable-font instance lineage

## Ownership and identity

`core/framework/render/text/font/` defines neutral `VariationCoords` and `InstancedFaceId` contracts. `graphics/text/font/instance.rs` is the sole implementation owner for canonical coordinate ordering, duplicate-tag resolution, finite-value validation, face-axis clamp/default removal, OpenType normalized F2DOT14 quantization, deterministic BLAKE3 identity, collision detection, and reverse lookup from instance ID to `(FontFaceId, VariationCoords)`. `FontDatabase` creates one descriptor-default instance at face registration and owns all additional instances; shaping and raster leaves consume database records rather than reconstructing coordinates from opaque hashes. Coordinates that produce the same normalized OpenType bucket therefore share an instance and cannot split shaping/atlas caches without changing the rendered outline.

The effective-coordinate projection reads the selected face's actual `fvar` axes. It applies `UiResolvedStyle.font_weight` only when `wght` exists, clamps values to axis bounds, removes default-valued coordinates, and drops tags that the face does not expose. Static fonts therefore retain an empty variation set, while equivalent default requests share the offline/cache identity instead of producing false variants.

## Shaping and raster flow

Horizontal layout keeps cosmic-text as the paragraph, BiDi, fallback, and initial cluster owner. The folder-backed `graphics/text/shaping/horizontal/` leaf groups adjacent glyph clusters by actual face, instance, and direction, then re-shapes segments whenever effective variation coordinates or a per-run language are present. RustyBuzz receives language, OpenType features, kerning policy, size, and effective coordinates, so `locl` and variable axes share one authoritative face/cluster projection instead of a renderer-side exception. It projects real glyph IDs, cluster ranges, advances, and offsets back into the shared `ShapedGlyphRun`. The leaf rejects vertical requests; vertical shaping applies the same coordinates and language directly in the TTB/BTT backend.

`ShapedGlyph` carries both base `FontFaceId` and `InstancedFaceId`. The base face remains the byte/fallback owner; the instance prevents equal glyph IDs from different coordinate selections sharing atlas entries. Screen-space extraction preserves both identities for horizontal and VerticalRl batches.

Native bitmap worker requests carry arbitrary Swash variation settings rather than a weight-only option. The glyphon cache key's weight overrides the descriptor `wght` coordinate, while descriptor custom axes remain present. Dynamic SDF/MSDF/MTSDF sets the same coordinates on `ttf_parser::Face` before outline, bounds, advance, and ascent extraction. The SDF atlas key contains base face, instance, weight, and bake mode, so neither variable instances nor distance-field modes alias.

## Offline artifact interaction

Runtime `.zsdf` lookup derives `variation_hash` from effective, sorted coordinates. Default-valued coordinates normalize to the existing empty variation hash, preserving V1 default artifacts. A non-default instance can only consume an artifact with the exact coordinate hash; otherwise the established dynamic generator is used. The V1 build request currently accepts the hash rather than design coordinates, so producing non-default offline artifacts remains an explicit follow-up; runtime never treats a hash-only artifact as proof that a coordinate-specific outline was baked.

## Validation state

The implementation is source-complete for instance registry, horizontal/vertical RustyBuzz, native Swash, dynamic distance fields, atlas identity, and runtime offline selection. Windows exact tests use the real `C:\Windows\Fonts\bahnschrift.ttf` `wdth` axis to require different shaped advances and SDF pixels, and real `C:\Windows\Fonts\calibri.ttf` Russian/Serbian Cyrillic forms to require distinct `locl` glyph IDs. Managed job `d4821ebfeef1445eb743515c9439948c` built the current Runtime lib-test in 37m04s and ran `text_horizontal_` 5/5 in 26.98s; its external ephemeral target was deleted. Database F2DOT14/static-axis tests, dynamic SDF pixels, atlas separation, upward regression, and product framebuffer remain pending, so this document remains `in_progress` and no completion claim is made.
