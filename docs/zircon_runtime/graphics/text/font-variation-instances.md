---
related_code:
  - zircon_runtime/src/text/model/font/face.rs
  - zircon_runtime/src/text/model/font/database.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/font/instance.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/text/shaping/horizontal
  - zircon_runtime/src/text/shaping/vertical/backend.rs
  - zircon_runtime/src/text/raster/swash
  - zircon_runtime/src/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
implementation_files:
  - zircon_runtime/src/text/font/instance.rs
  - zircon_runtime/src/text/font/instance/tests.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/database/tests.rs
  - zircon_runtime/src/text/shaping/horizontal/backend.rs
  - zircon_runtime/src/text/shaping/horizontal/projection.rs
  - zircon_runtime/src/text/shaping/horizontal/tests.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/text/shaping/vertical/backend.rs
  - zircon_runtime/src/text/raster/swash/request.rs
  - zircon_runtime/src/text/raster/swash/rasterizer.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/text/sdf/offline/identity.rs
  - zircon_runtime/src/text/sdf/font_bake/distance_field.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_project_fixture.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
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
  - text_horizontal_rustybuzz_backend_preserves_serbian_locl_in_mixed_script_text
  - text_font_asset_load_failure_does_not_create_a_negative_cache_record
  - text_font_refresh_recomputes_internal_vertical_advances
  - text_font_refresh_preserves_resolved_layout_vertical_advances
  - export_runtime_multilingual_text_product_framebuffer_png
  - text_msdf_dynamic_generation_applies_real_variable_width_axis
  - sdf_atlas_plan_separates_variable_font_instances_on_same_face
doc_type: module-detail
status: complete
---

# Runtime variable-font instance lineage

## Ownership and identity

`text/model/font/` defines neutral `VariationCoords` and `InstancedFaceId` contracts. `text/font/instance.rs` is the sole implementation owner for canonical coordinate ordering, duplicate-tag resolution, finite-value validation, face-axis clamp/default removal, OpenType normalized F2DOT14 quantization, deterministic BLAKE3 identity, collision detection, and reverse lookup from instance ID to `(FontFaceId, VariationCoords)`. `FontDatabase` creates one descriptor-default instance at face registration and owns all additional instances; shaping and raster leaves consume database records rather than reconstructing coordinates from opaque hashes. Coordinates that produce the same normalized OpenType bucket therefore share an instance and cannot split shaping/atlas caches without changing the rendered outline.

The effective-coordinate projection reads the selected face's actual `fvar` axes. It applies `UiResolvedStyle.font_weight` only when `wght` exists, clamps values to axis bounds, removes default-valued coordinates, and drops tags that the face does not expose. Static fonts therefore retain an empty variation set, while equivalent default requests share the offline/cache identity instead of producing false variants.

## Shaping and raster flow

Horizontal layout keeps cosmic-text as the paragraph, BiDi, fallback, and initial cluster owner. `fallback_text_spans.rs` resolves every grapheme through the authoritative Zircon `FontDatabase` and carries the selected logical family together with `FontFaceId` and `InstancedFaceId`; adjacent spans merge only when all three identities match. Cosmic/glyphon may still select a physical backend face for paragraph layout, but `cosmic.rs` projects each glyph back through the matching authoritative span before the horizontal RustyBuzz leaf runs. This prevents multiple logical variable members backed by one physical family from collapsing to glyphon's default face/instance.

The folder-backed `text/shaping/horizontal/` leaf groups adjacent glyph clusters by actual face, instance, direction, and resolved ISO15924 script, then re-shapes segments whenever effective variation coordinates or a per-run language are present. Each strong script is explicitly set on the RustyBuzz buffer before `guess_segment_properties`; Common, Inherited, and Unknown tags deliberately retain the backend guess path. RustyBuzz therefore receives one script together with language, OpenType features, kerning policy, size, and effective coordinates, so `locl` and variable axes share one authoritative face/cluster projection without allowing an adjacent Latin cluster to suppress Cyrillic localization. It projects real glyph IDs, cluster ranges, advances, and offsets back into the shared `ShapedGlyphRun`. The leaf rejects vertical requests; vertical shaping applies the same coordinates, script, and language directly in the TTB/BTT backend.

Screen-space preparation loads each distinct explicit auto/native/SDF font asset in `text/resolved_batches.rs` before atlas planning. `font_assets.rs` returns separate successful-load and actual face-count-change signals: a failed manifest is not inserted into the cache, so a later project import can retry instead of inheriting a permanent negative record; native atlas invalidation is driven by the authoritative database face delta rather than map length. A successful new asset refreshes batch glyphs through `render/text_advances.rs`, the existing shaping owner, so render-command extraction cannot leave stale default-font face/instance data in the atlas request.

The refresh path distinguishes advance provenance. Resolved layout-line batches retain their externally authoritative `source_range` and `glyph_advances`, because their frame was computed from those values. Raw render-command batches have no source range; their internally derived advances are cleared before re-shaping, so VerticalRl cannot pair a new project face with fallback-font spacing. The root `text.rs` remains an assembly owner and sibling report/tests import their canonical child modules directly.

`ShapedGlyph` carries both base `FontFaceId` and `InstancedFaceId`. The base face remains the byte/fallback owner; the instance prevents equal glyph IDs from different coordinate selections sharing atlas entries. Screen-space extraction preserves both identities for horizontal and VerticalRl batches.

Native bitmap worker requests carry arbitrary Swash variation settings rather than a weight-only option. The glyphon cache key's weight overrides the descriptor `wght` coordinate, while descriptor custom axes remain present. Dynamic SDF/MSDF/MTSDF sets the same coordinates on `ttf_parser::Face` before outline, bounds, advance, and ascent extraction. The SDF atlas key contains base face, instance, weight, and bake mode, so neither variable instances nor distance-field modes alias.

## Offline artifact interaction

Runtime `.zsdf` lookup derives `variation_hash` from effective, sorted coordinates. Default-valued coordinates normalize to the existing empty variation hash, preserving V1 default artifacts. A non-default instance can only consume an artifact with the exact coordinate hash; otherwise the established dynamic generator is used. The V1 build request currently accepts the hash rather than design coordinates, so producing non-default offline artifacts remains an explicit follow-up; runtime never treats a hash-only artifact as proof that a coordinate-specific outline was baked.

## Validation state

The instance lineage has a real Windows product framebuffer gate. The exporter itself is Windows-gated until a repository-owned cross-platform variable font fixture exists, so another platform cannot emit a variable-font-named PNG while silently omitting both instance samples and their assertions. The first managed compile/GPU pair (`a33160e6543e4419a334c6422b8f7f37`, `37c142f4696747e0b63df8161787f7d2`) established the 1/1 physical evidence and exposed the review surface. After all four review findings were fixed, compile job `d4fd827abfc3450090d20275d91b57ee` exited 0 and the final exact managed GPU job `d80d6dabac754907b50aa3ae2c1c1056` exited 0 with 1 passed / 0 failed in 1409.03 seconds. Original-resolution inspection and pixel analysis of the 1080×1840 PNG found narrow 256px/3187px, wide 346px/3747px, and 4984 differing pixels; the artifact is 353953 bytes with 2442 colors and SHA256 `754A7C1CC64D98B50D6FB798F702353C4BABB7EAAA5B722657529B4641BB9C40`. Repository `target` and the D/E/F Cargo target roots contain no duplicate. Managed focused job `61aaa263af684ab7b028956c772e0a20` passed `text_font` 41/41 and `text_horizontal_` 6/6; exact job `deb789dcbdbe43c3b17fea6a234c9079` passed the dynamic SDF width-axis, atlas instance-separation, and arbitrary-axis Swash tests 1/1 each. Independent re-review returned `Accept` with no Critical or Important issue. This completes the FR-M2 variable-font instance lineage; FR-M3 CompositeFont and the cross-platform fixture remain separate follow-up work.
