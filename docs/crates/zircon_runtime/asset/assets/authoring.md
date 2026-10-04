---
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
implementation_files:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
tests:
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
---
# Authoring Assets

Fresh 2026-06-25 Runtime 15 F5 asset authoring typed-error evidence: status anchor `runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred`. `asset/assets/authoring.rs` now owns `AssetAuthoringError` and `AssetAuthoringResult`; `TerrainAsset::validate_dimensions(...)`, `TileMapAsset::validate_layers(...)`, and `MaterialGraphAsset::validate_output_node(...)` no longer return `Result<(), String>` or build `Err(format!(...))` branches. The typed variants are `TerrainSampleCount`, `TileMapLayerTileCount`, and `MaterialGraphMissingOutput`.

`asset/importer/ingest/import_authoring_asset.rs` keeps the existing importer contract by converting `AssetAuthoringError` to `AssetImportError::Parse(error.to_string())` only at the import boundary. `asset/tests/assets/authoring.rs::authoring_asset_validation_reports_typed_errors` locks behavior by matching the three variants directly, and `tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_asset_authoring_uses_typed_error` locks the source, facade exports, importer boundary, and status/docs anchors.

Verification for this slice: scoped rustfmt/static scans and docs/status/session anchor scans passed; Cargo was deferred because external cargo/rustc lanes were active, so no Cargo pass is claimed.
