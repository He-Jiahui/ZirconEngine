---
related_code:
  - zircon_runtime/src/core/framework/navigation/asset/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/assets/mod.rs
  - zircon_runtime/src/asset/tests/assets/navigation.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
implementation_files:
  - zircon_runtime/src/core/framework/navigation/asset/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/assets/mod.rs
  - zircon_runtime/src/asset/tests/assets/navigation.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/asset/tests/assets/navigation.rs::navmesh_binary_roundtrip_reports_typed_errors
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_navigation_asset_uses_typed_error
  - "2026-06-25 static: scoped rustfmt/static scans/docs-status-session anchors passed; Cargo deferred due active cargo/rustc lanes"
doc_type: module-detail
---

# Navigation Asset Integration

`core/framework/navigation/asset/` is the unique owner of the runtime DTOs for baked navigation data and authored navigation settings. `NavMeshAsset` stores versioned mesh vertices, indices, polygons, tiles, area costs, and off-mesh links. `NavigationSettingsAsset` stores the default runtime agent and area settings consumed by navigation managers. The asset subsystem owns `ImportedAsset`, `Asset` marker integration, cache payload routing, project loading, and regression fixtures; it does not define or re-export a second navigation schema.

## Binary Error Contract

Runtime 15 F5 navigation asset typed errors (`runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred`) converted the `NavMeshAsset::to_bytes(...)` and `NavMeshAsset::from_bytes(...)` helpers from `Result<_, String>` to `NavigationAssetResult<T>`.

`NavigationAssetError` keeps bincode failures typed at the neutral navigation contract boundary:

- `NavigationAssetError::Serialize` wraps navmesh serialization failure sources.
- `NavigationAssetError::Deserialize` wraps navmesh deserialization failure sources.

The public `core/framework/navigation/mod.rs` surface exports `NavigationAssetError` and `NavigationAssetResult`, so callers can match the binary failure kind without parsing display strings. The retired `zircon_runtime::asset::{NavMeshAsset, NavigationSettingsAsset, NavigationAssetError}` path is not preserved as a compatibility facade.

## Regression Coverage

`asset/tests/assets/navigation.rs::navmesh_binary_roundtrip_reports_typed_errors` covers a binary roundtrip and an invalid-byte failure path that must return `NavigationAssetError::Deserialize`.

`review_f5_navigation_asset_uses_typed_error` locks the neutral source, framework export, test module mount, this document, and the Runtime 15/status docs anchors. It also rejects reintroducing `Result<Vec<u8>, String>`, `Result<Self, String>`, or lossy `error.to_string()` in the navigation asset binary helpers.
