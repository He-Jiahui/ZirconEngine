---
related_code:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc
  - zircon_runtime/assets/fonts/OFL-NotoSansSC.md
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/font_source.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/fixtures.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/composite_resolve.rs
  - zircon_runtime/src/text/font/asset_registration.rs
  - zircon_runtime/src/text/font/test_font_fixtures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - tools/tests/test_text_01_composite_activation.py
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
implementation_files:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc
  - zircon_runtime/assets/fonts/OFL-NotoSansSC.md
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/font_source.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/fixtures.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/database/tests.rs
  - zircon_runtime/src/text/font/composite_resolve.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/text/font/asset_registration.rs
  - zircon_runtime/src/text/font/test_font_fixtures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
plan_sources:
  - user: 2026-07-10 complete the runtime text/font/layout architecture and rendered evidence
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs::font_asset_parse_reports_typed_toml_error_source
  - zircon_runtime/src/asset/tests/assets/font.rs::importer_decodes_font_assets_from_font_toml
  - zircon_runtime/src/asset/tests/assets/font.rs::importer_preserves_woff2_decode_error_source
  - zircon_runtime/src/asset/tests/assets/font.rs::runtime_default_font_manifest_declares_culture_aware_composite_font
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs::text_font_cmap_coverage_bitset_matches_face
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs::text_font_static_face_reports_no_variable_axes
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs::text_font_parse_ttf_extracts_os2_name_metadata
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs::text_font_variable_axes_roundtrip
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs::text_font_woff2_decodes_to_sfnt
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs::text_font_malformed_woff2_preserves_decode_failure
  - zircon_runtime/src/text/font/database/tests.rs::text_font_database_decodes_woff2_once_for_native_and_sdf_consumers
  - zircon_runtime/src/text/font/database/tests.rs::text_font_ttc_nonzero_face_materializes_for_real_sdf_raster
  - zircon_runtime/src/text/font/database/tests.rs::text_font_variations_hash_normalizes_coordinate_order
  - zircon_runtime/src/text/font/database/tests.rs::text_composite_font_resolves_default_and_subfont_ranges
  - zircon_runtime/src/text/font/database/tests.rs::text_font_database_composite_activation_is_explicit_and_replaceable
  - zircon_runtime/src/text/font/database/tests.rs::text_font_runtime_default_composite_selects_checked_in_zh_hans_face
  - tools/tests/test_text_01_composite_activation.py
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_font_asset_uses_typed_error_source
  - "2026-06-25 static: scoped rustfmt/static scans/docs-status-session anchors passed; Cargo deferred due active cargo/rustc lanes"
  - "2026-06-28 FR-M2: scoped rustfmt --check and cargo metadata --locked passed; runtime cargo check/focused text_font timed out during compile without Rust diagnostics"
  - "2026-06-28 FR-M2 render strategy: scoped rustfmt/git diff/cargo metadata passed; focused render_strategy runtime test timed out during compile without Rust diagnostics"
  - "2026-06-28 FR-M2 asset registration: scoped rustfmt/git diff/cargo metadata passed; focused font-asset logical-face registration runtime test timed out during compile without Rust diagnostics"
  - "2026-06-28 FR-M2 UI registry schema convergence: scoped rustfmt, cargo metadata, and warm zircon_runtime no-default cargo check passed; focused registry test hit Cargo target write error once, then timed out during compile without Rust diagnostics"
  - "2026-06-29 Runtime 15 M2 font/UI asset schema naming hard cutover: runtime_15_font_ui_asset_schema_names_use_current_policy_terms locks asset/assets/font.rs schema_v1_render_mode and asset/importer/ingest/ui_v2_document_import.rs profile removal; status runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred; Cargo deferred by Runtime 15 implementation-slice cadence"
  - "2026-06-29 Runtime 15 M2 font render-mode priority fixture naming hard cutover: runtime_15_font_render_mode_priority_fixture_uses_schema_v1_name locks graphics/scene/scene_renderer/ui/font_asset.rs::schema_v1_render_mode_takes_priority_over_strategy_default_mode; status runtime_15_font_render_mode_priority_fixture_naming_hard_cutover_static_passed_cargo_deferred; scoped rustfmt passed; narrowed naming-boundary harness exacts 2/2; plan_status exacts 2/2; module_convention_gate classified-and-clear; migration_debt_count=0; risk_count=0; Cargo deferred by Runtime 15 implementation-slice cadence and active cargo/rustc lanes"
doc_type: module-detail
---

# Font Asset Records

`asset/assets/font.rs` owns the runtime DTO for typed `.font.toml` source manifests. `FontAsset` stores the font source path, optional family name, optional UI text render mode, explicit face index, parsed family members, variable font instances, fallback families, an optional neutral `CompositeFontDescriptor`, render strategy, and importer-produced metadata used by runtime UI font import paths.

## FR-M2 Metadata Contract

`asset/importer/ingest/import_font_asset/mod.rs` is the manifest owner. It parses `.font.toml`, resolves the relative source file next to the manifest, reads the font bytes, records the source dependency for `res://` imports, and fills missing family/family-member/variable-instance fields from parsed metadata.

`asset/assets/font_source.rs` is the single raw-container boundary. It recognizes WOFF2, decodes it once to neutral SFNT/TTC bytes, preserves the original source-format tag for metadata, and exposes opaque typed failures without leaking decoder types. Because the decoder handles untrusted asset bytes and contains a few internal assertions, this isolation boundary also converts a malformed-input decoder panic into `FontSourceDecodeError::DecoderPanic` instead of letting it unwind through the asset pipeline. The importer and `FontDatabase` both consume this owner, so glyphon and SDF share the same decoded bytes instead of maintaining separate WOFF2 branches. The same owner can materialize a selected TTC face as a standalone SFNT buffer for backends such as `fontsdf` that do not accept a collection index; it validates the face, rewrites table offsets, and recomputes `head.checkSumAdjustment`.

`asset/importer/ingest/import_font_asset/parse_sfnt.rs` is the metadata parser isolation leaf. It uses `ttf-parser` to extract sfnt/TTC face count, name-table family data, OS/2 weight and width class, style, `fvar` axes and named-instance coordinates, compact cmap coverage ranges, selected typographic line metrics, Windows clipping metrics, and underline/strikeout metrics. `ttf-parser` types do not leave this leaf; exported asset data remains the serde DTOs in `asset/assets/font.rs`. Its behavior tests and generated TTC/variable-font fixtures are folder-backed under `parse_sfnt/tests/`, keeping the production owner focused and below the structure budget.

`text/font/database.rs` decodes a registered source through the same font-source owner before it reads selected-face family, weight, style, and stretch. `text/font/asset_registration.rs` projects `FontAsset.family_members` into logical face descriptors and builds the asset source key from path, face index, family, weight, style, stretch, and variation coordinates. `FontDatabase::register_font_asset` consumes those descriptors and `fallback_families`, extending the database fallback chain without collapsing multiple declared logical faces that share one physical source face. Variation instance IDs hash a canonical tag/value ordering, so equivalent coordinate maps do not fork shaping/atlas cache identities. Native glyphon and SDF bake manifest paths pass full `.font.toml` assets into this registration entry; direct font-file paths still use `register_font_file`. SDF requests call `standalone_face_bytes`, allowing a non-zero TTC face to produce real `fontsdf` pixels instead of silently falling back to face zero or the default font.

`FontAsset::effective_render_mode()` is the asset-level helper that resolves schema-v1 `render_mode`, `render_strategy.default_mode`, and `allow_native` / `allow_sdf` constraints. `graphics/scene/scene_renderer/ui/font_asset.rs` consumes that helper instead of duplicating schema branching. The `schema_v1_render_mode` input remains the priority source for `.font.toml` schema-v1 manifests; strategy defaults only fill the absent case and are clamped before they leave the asset boundary.

## FR-M3 CompositeFont Contract

`FontAsset.composite_font` serializes the neutral core descriptor directly: one default family and ordered sub-font entries with script, inclusive Unicode ranges, and optional normalized BCP-47 culture tags. Culture matching accepts exact tags and configured-parent matches such as `zh-Hans` selecting `zh-Hans-CN`, allowing Han faces to distinguish simplified Chinese, traditional Chinese, Japanese, and Korean without placing locale policy in UI code. Asset registration is deliberately side-effect free with respect to project-default selection: it registers declared faces and retains the descriptor under the exact asset owner, but it does not replace the project or runtime default CompositeFont and does not inject CompositeFont families into global fallback. A text request whose `font` is that asset URI uses the owner's compiled CompositeFont and optional `font_family` typeface selector. `ScreenSpaceUiTextSystem` still explicitly activates the loaded default record as project policy. Loading a secondary font asset cannot replace that policy, and a missing default record explicitly clears stale project CompositeFont state.

Font-object owner identity is part of shaping and layout generation. Owners may share one physical face/source, but attaching, changing, or removing an owner advances the font generation because the owner is independently selectable. Owner fallback declarations are scoped to that object and the database base/platform chain; they do not consume another loaded asset's fallback list. Composite descriptors are compiled when the database generation is published and retained as `Arc<CompositeFontIndex>`, so shaping does not parse an asset URI or hash the authored sub-font table per glyph or per request.

`assets/fonts/default.font.toml` is the runtime default package declaration. Its checked-in `ZirconDefaultComposite-subset.ttc` contains Fira Mono at face 0 and the deterministic `Zircon Noto Sans CJK SC Proof` subset at face 1. The first `zh-Hans` route selects face 1 before the optional host Noto/YaHei families, so project Chinese text has a repository-owned glyph source on every machine. `OFL-NotoSansSC.md` carries the corresponding SIL Open Font License. Actual system discovery remains explicit: `SystemFontPolicy` defaults to `Disabled`, while the screen-space UI renderer opts into `Discover`; headless/default databases therefore do not enumerate host fonts accidentally.

WOFF2 decode, deterministic variable-axis metadata, TTC enumeration, selected non-zero TTC face extraction, and real SDF raster construction have focused regression coverage. FR-M3 additionally parses the checked-in TTC, requires face 1 to cover the product Chinese string, and proves the active `zh-Hans` candidate resolves to that face. The ignored multilingual product exporter is the window-level acceptance gate; unit metadata alone is not treated as rendered evidence.

## Parse Error Contract

Runtime 15 F5 font asset typed errors (`runtime_15_font_asset_typed_errors_static_passed_cargo_deferred`) converted `FontAsset::from_toml_str(...)` to return `FontAssetResult<T>`.

`FontAssetError::Parse` wraps `toml::de::Error` as the source, so callers can inspect the TOML parser failure without parsing display strings. `asset/assets/mod.rs` and `asset/mod.rs` export `FontAssetError` and `FontAssetResult`.

The built-in importer no longer flattens font failures into `AssetImportError::Parse(String)`. It uses dedicated `FontDocument`, `FontSourceIo`, `FontSourceDecode`, `FontMetadata`, and `FontSourcePath` variants, preserving the TOML, I/O, WOFF2, and face-parse source chain with the failing path attached. This is the Text 01 application of the repository E1 typed-error rule.

## Regression Coverage

`asset/tests/assets/font.rs::font_asset_parse_reports_typed_toml_error_source` covers invalid TOML input and requires `FontAssetError::Parse` to expose an error source.

The FR-M2 importer tests copy the runtime Fira font fixture next to a temporary `.font.toml`, import it through `AssetImporter` and `ProjectManager`, assert that parsed metadata is attached, cmap coverage contains `A`, and family members are populated. Folder-backed parser tests build deterministic TTC and synthetic `fvar` fixtures, and encode a transformed-glyf WOFF2 source to verify decode, metadata, variable coordinates, line/decorative metrics, malformed-input source preservation, and panic containment at the decoder boundary. The graphics database tests verify WOFF2 bytes are decoded once and shared by native/SDF consumers, variation coordinate order is canonical, and a selected TTC face 1 becomes a standalone font that produces non-empty SDF pixels. FR-M3 tests lock explicit CompositeFont activation/replacement/clear behavior, parse the checked-in default package, prove the product Chinese string has real face-1 glyphs, select culture-specific Han candidates, keep Latin on the default family, and prove disabled system discovery is the database default. `test_text_01_composite_activation.py` guards registration/activation ownership and the folder-backed candidate resolver. Window-level evidence comes only from the real WGPU product exporter and its framebuffer PNG under `docs/tests/runtime/text`.

FR-M3 acceptance used managed GPU job `f320e76017714cfe97b9b52f92f69b52` (exact ignored exporter 1/1). The real 1080×1840 framebuffer is `docs/tests/runtime/text/runtime_text_composite_font_cjk_product_framebuffer_20260714.png`, SHA256 `754A7C1CC64D98B50D6FB798F702353C4BABB7EAAA5B722657529B4641BB9C40`; approved target roots contain no duplicate. Independent review returned Critical 0 / Important 0 / Accept.

`review_f5_font_asset_uses_typed_error_source` locks the font source, facade exports, importer boundary, this document, and Runtime 15/status docs anchors. It also rejects reintroducing `Parse(String)`, the old explicit `Result<Self, FontAssetError>` signature, or lossy `error.to_string()` inside `asset/assets/font.rs`.
