---
related_code:
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/distance_field.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/text/sdf/font_bake/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs
  - zircon_runtime/assets/fonts/default.font.toml
implementation_files:
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/distance_field.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
plan_sources:
  - user: 2026-07-15 complete and verify the runtime text and layout architecture
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_editor/editor/02/fixed-2026-07-15-sdf-font-bake-cjk-loaded-font-count-regression.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/text/sdf/font_bake/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs
  - cargo test -p zircon_runtime --lib text::sdf::font_bake::tests::sdf_font_bake_rasterizes_materialized_system_cjk_face --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib scene:: --locked
status: accepted
doc_type: module-detail
---

# SDF font bake cache and report semantics

## Purpose

`sdf_font_bake.rs` converts renderer-neutral atlas keys into SDF, MSDF, or MTSDF glyph bitmaps and metrics. It owns the renderer-side materialized `fontsdf::Font` cache, glyph bake cache, offline artifact lookup, dynamic generation fallback, page pixel assembly, and the typed bake report consumed by the screen-space SDF renderer.

The module does not own shaping or fallback policy. Shaping supplies an authoritative `font_id` and optional backend glyph ID whenever that identity exists. Only scalar-only callers, or stale/mismatched handles that cannot be trusted, ask `FontDatabase` to resolve a face from the exact FontObject owner, its CompositeFont, owner-local typeface, locale, and base fallback chain.

## Related files

- `sdf_atlas/text_keys.rs` preserves shaped glyph ID, base face ID, instance ID, locale, family, and distance-field mode in `SdfAtlasGlyphKey`.
- `sdf_font_bake/distance_field.rs` performs dynamic SDF/MSDF/MTSDF generation against the selected face.
- `sdf_font_bake/offline_source.rs` resolves deterministic prebuilt `.zsdf` glyph pages before dynamic generation.
- `sdf_font_bake/tests.rs` owns cache, face identity, real Windows CJK, bitmap, metric, and report regressions.

## Behavior model

Each atlas build records the number of materialized faces already present before slot processing. Slot keys are then resolved in this order:

1. An authoritative `font_id` is used directly after the `FontDatabase` proves that its standalone face bytes are available.
2. Without a valid face ID, the requested/default font asset is loaded or attached before resolution.
3. `FontDatabase` applies the registered request owner's precompiled CompositeFont, script/range/locale routing, owner-local family, and base fallback chain. When no request owner exists, project/runtime defaults remain authoritative.

Resolved faces are deduplicated by `FontFaceId`. `ensure_sdf_font` cannot insert the same face twice because the cache is keyed by that ID. A successful offline or dynamic glyph bake stops face iteration; a later candidate is materialized only if earlier candidates cannot provide the glyph.

## Report semantics

`SdfAtlasBakeReport` distinguishes two cache facts:

- `resident_font_count` is the number of reusable materialized faces retained after this build.
- `loaded_font_count` is the number of faces newly materialized by this build call.

The second build of an unchanged atlas may therefore report the same resident count and zero newly loaded faces. This distinction prevents a project CompositeFont face and a previously materialized system face from being misreported as duplicate loads.

## Design rationale

The face ID carried by shaping is stronger than a family string. A family-only CJK key is allowed to select the checked-in project CompositeFont face even when a different system CJK face was materialized earlier. Tests that claim to exercise a particular system face must therefore populate `font_id`, matching the production shaped-key path; forcing family fallback to reuse an unrelated preloaded face would violate Text01/02 face identity authority.

Recovery is intentionally weaker than a valid shaped handle but must use the same owner selection policy. It resolves a scalar through `FontDatabase` and never reuses the stale glyph ID. This matches Unreal's primary path, where the SDF atlas consumes `FShapedGlyphEntry` face data and glyph index directly, while keeping malformed or legacy scalar-only inputs deterministic without inventing a renderer-local fallback order.

The resolver retains candidate provenance: the requested typeface is owner-local, while CompositeFont and authored fallback families may use an external face if the owner does not provide one. Font bake receives the resolved face and does not apply a second same-name-family retry.

The cache remains local to the SDF bake owner. It does not modify shared `FontDatabase` fallback ordering, add a renderer special case, or introduce a compatibility path around CompositeFont.

## Edge cases and constraints

- An empty atlas build loads no faces and reports zero newly loaded faces; its resident count reflects the cache state that already existed.
- A missing outline records a typed generation failure and may try the next resolved face.
- Different `FontFaceId` values are distinct residents even if they share a family label or source container.
- `font_instance_id` remains part of glyph cache identity, while face materialization continues to use the authoritative base face bytes.
- The code and folder-backed test owners must remain below the repository's structure budgets; report policy must not move into the scene renderer root.

## Test coverage

The Text05 failure repair adds deterministic coverage for first-build versus cached-build counts and updates the Windows CJK test to use the authoritative Microsoft YaHei UI face ID. A current-source Windows lib-test executable passed all 13 `sdf_font_bake` tests and all 44 runnable `sdf_render` tests. The originating `scene::` gate ran 1,714 tests: 1,705 passed, 6 were ignored, and 3 unrelated renderer-owner tests failed because concurrent shadow-binding work left source/pipeline-layout guards inconsistent. No text, SDF bake, layout, dynamic-scene, or Editor02-owned assertion failed.

The 2026-08-29 owner-scoped recovery regression is implemented but has not been executed in Cargo or WGPU. It requires an unshaped CJK scalar from a two-face FontAsset to select that owner's CompositeFont CJK face; the earlier accepted counts remain historical evidence only.

## Plan sources

This behavior closes the Text05 handoff `sdf-font-bake-cjk-loaded-font-count-regression` while preserving the Text01 CompositeFont ordering and the Text02 shaped face-ID contract. It also follows the structure convention by keeping report semantics and tests in the existing SDF bake owner instead of adding an Editor02 or renderer-root workaround.
