---
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
implementation_files:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs::font_asset_parse_reports_typed_toml_error_source
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_font_asset_uses_typed_error_source
  - "2026-06-25 static: scoped rustfmt/static scans/docs-status-session anchors passed; Cargo deferred due active cargo/rustc lanes"
doc_type: module-detail
---

# Font Asset Records

`asset/assets/font.rs` owns the runtime DTO for typed `.font.toml` source manifests. `FontAsset` stores the font source path, optional family name, and optional UI text render mode used by runtime UI font import paths.

## Parse Error Contract

Runtime 15 F5 font asset typed errors (`runtime_15_font_asset_typed_errors_static_passed_cargo_deferred`) converted `FontAsset::from_toml_str(...)` to return `FontAssetResult<T>`.

`FontAssetError::Parse` wraps `toml::de::Error` as the source, so callers can inspect the TOML parser failure without parsing display strings. `asset/assets/mod.rs` and `asset/mod.rs` export `FontAssetError` and `FontAssetResult`.

The built-in importer keeps the existing public `AssetImportError::Parse` boundary in `asset/importer/ingest/import_font_asset.rs`, where the typed font error is formatted into the importer diagnostics string.

## Regression Coverage

`asset/tests/assets/font.rs::font_asset_parse_reports_typed_toml_error_source` covers invalid TOML input and requires `FontAssetError::Parse` to expose an error source.

`review_f5_font_asset_uses_typed_error_source` locks the font source, facade exports, importer boundary, this document, and Runtime 15/status docs anchors. It also rejects reintroducing `Parse(String)`, the old explicit `Result<Self, FontAssetError>` signature, or lossy `error.to_string()` inside `asset/assets/font.rs`.
