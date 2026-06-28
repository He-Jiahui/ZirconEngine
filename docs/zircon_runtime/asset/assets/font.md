---
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/font/asset_registration.rs
  - zircon_runtime/src/graphics/text/font/test_font_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
implementation_files:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/graphics/text/font/asset_registration.rs
  - zircon_runtime/src/graphics/text/font/test_font_fixtures.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs::font_asset_parse_reports_typed_toml_error_source
  - zircon_runtime/src/asset/tests/assets/font.rs::importer_decodes_font_assets_from_font_toml
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs::tests::text_font_cmap_coverage_bitset_matches_face
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs::tests::text_font_static_face_reports_no_variable_axes
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs::tests::text_font_parse_ttf_extracts_os2_name_metadata
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_font_asset_uses_typed_error_source
  - "2026-06-25 static: scoped rustfmt/static scans/docs-status-session anchors passed; Cargo deferred due active cargo/rustc lanes"
  - "2026-06-28 FR-M2: scoped rustfmt --check and cargo metadata --locked passed; runtime cargo check/focused text_font timed out during compile without Rust diagnostics"
  - "2026-06-28 FR-M2 render strategy: scoped rustfmt/git diff/cargo metadata passed; focused render_strategy runtime test timed out during compile without Rust diagnostics"
  - "2026-06-28 FR-M2 asset registration: scoped rustfmt/git diff/cargo metadata passed; focused font-asset logical-face registration runtime test timed out during compile without Rust diagnostics"
  - "2026-06-28 FR-M2 UI registry schema convergence: scoped rustfmt, cargo metadata, and warm zircon_runtime no-default cargo check passed; focused registry test hit Cargo target write error once, then timed out during compile without Rust diagnostics"
doc_type: module-detail
---

# Font Asset Records

`asset/assets/font.rs` owns the runtime DTO for typed `.font.toml` source manifests. `FontAsset` stores the font source path, optional family name, optional UI text render mode, explicit face index, parsed family members, variable font instances, fallback families, render strategy, and importer-produced metadata used by runtime UI font import paths.

## FR-M2 Metadata Contract

`asset/importer/ingest/import_font_asset/mod.rs` is the manifest owner. It parses `.font.toml`, resolves the relative source file next to the manifest, reads the font bytes, records the source dependency for `res://` imports, and fills missing family/family-member/variable-instance fields from parsed metadata.

`asset/importer/ingest/import_font_asset/parse_sfnt.rs` is the parser isolation leaf. It uses `ttf-parser` to extract sfnt/TTC face count, name-table family data, OS/2 weight and width class, style, `fvar` axes and best-effort named instances, and compact cmap coverage ranges. `ttf-parser` types do not leave this leaf; exported asset data remains the serde DTOs in `asset/assets/font.rs`.

`graphics/text/font/database.rs` also reads selected-face family, weight, style, and stretch directly from registered source bytes. `graphics/text/font/asset_registration.rs` projects `FontAsset.family_members` into logical face descriptors and builds the asset source key from path, face index, family, weight, style, stretch, and variation coordinates. `FontDatabase::register_font_asset` consumes those descriptors and `fallback_families`, extending the database fallback chain without collapsing multiple declared logical faces that share one physical source face. Native glyphon and SDF bake manifest paths now pass full `.font.toml` assets into this registration entry; direct font-file paths still use `register_font_file`.

`FontAsset::effective_render_mode()` is the asset-level helper that resolves legacy `render_mode`, `render_strategy.default_mode`, and `allow_native` / `allow_sdf` constraints. `graphics/scene/scene_renderer/ui/font_asset.rs` and `ui/text/font_registry.rs` both consume that helper instead of duplicating schema branching. The legacy `render_mode` field remains the priority source for backwards-compatible manifests; strategy defaults only fill the absent case and are clamped before they leave the asset boundary.

WOFF2 is currently detected and reported as unsupported. TTC metadata can be enumerated through `ttf-parser`, but a real TTC fixture and render-path proof are still open. Runtime SDF rendering still builds `fontsdf` fonts from shared bytes without passing an explicit face index; the bake path now rejects non-zero face indices and falls back to the default font instead of silently using face 0. Multi-face TTC SDF rendering must not be treated as complete.

## Parse Error Contract

Runtime 15 F5 font asset typed errors (`runtime_15_font_asset_typed_errors_static_passed_cargo_deferred`) converted `FontAsset::from_toml_str(...)` to return `FontAssetResult<T>`.

`FontAssetError::Parse` wraps `toml::de::Error` as the source, so callers can inspect the TOML parser failure without parsing display strings. `asset/assets/mod.rs` and `asset/mod.rs` export `FontAssetError` and `FontAssetResult`.

The built-in importer keeps the existing public `AssetImportError::Parse` boundary in `asset/importer/ingest/import_font_asset/mod.rs`, where the typed font error and parser errors are formatted into importer diagnostics strings.

## Regression Coverage

`asset/tests/assets/font.rs::font_asset_parse_reports_typed_toml_error_source` covers invalid TOML input and requires `FontAssetError::Parse` to expose an error source.

The FR-M2 importer tests copy the runtime Fira font fixture next to a temporary `.font.toml`, import it through `AssetImporter` and `ProjectManager`, assert that parsed metadata is attached, cmap coverage contains `A`, and family members are populated, and build an in-test TTC fixture to assert multi-face metadata enumeration. The graphics text database tests include temporary patched-font/TTC fixtures that verify best-match reads OS/2 weight from source bytes and keeps distinct `face_index` registrations, plus a font-asset registration test that verifies family-member overrides, fallback families, repeated asset registration dedupe, and two logical declarations sharing one physical face. The patched-font/TTC fixture construction now lives in `graphics/text/font/test_font_fixtures.rs` and the family-member projection lives in `graphics/text/font/asset_registration.rs` instead of the database owner. The UI font manifest and UI registry tests cover render-strategy default selection, legacy `render_mode` precedence, disallowed Auto clamping, UI registry fallback-family merge/dedupe, and the old direct `FontAsset` literal shape is no longer used. The SDF bake tests cover fallback when `fontsdf` cannot open a requested non-zero face index, and their temporary manifest/source pair now cleans up both paths through a Drop helper. On 2026-06-28 the warmed `zircon_runtime --lib` no-default cargo check passed with existing warnings; the focused UI registry test first failed on a Cargo target fingerprint path write error and then timed out during compile in a separate target directory, so it is not recorded as passed.

`review_f5_font_asset_uses_typed_error_source` locks the font source, facade exports, importer boundary, this document, and Runtime 15/status docs anchors. It also rejects reintroducing `Parse(String)`, the old explicit `Result<Self, FontAssetError>` signature, or lossy `error.to_string()` inside `asset/assets/font.rs`.
