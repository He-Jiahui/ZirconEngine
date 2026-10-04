---
related_code:
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
implementation_files:
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs
  - tools/tests/test_frameworks_05_layer_direction.py
---

# Native Asset Import Command Boundary

The asset domain owns native importer request/response envelopes and the neutral command-host contract. `NativeAssetImportCommandHost` exposes only a stable host id and `invoke_asset_import_command(...)`; `NativeAssetImportCommandReport` returns an asset-owned status enum, diagnostics, and an optional payload.

`NativeAssetImporterHandler` stores `Arc<dyn NativeAssetImportCommandHost>`. It does not know `LoadedNativePlugin`, native plugin ABI status constants, plugin behavior reports, or plugin lifecycle state. The plugin native loader implements the neutral host contract for `LoadedNativePlugin` and maps its ABI report/status into the asset-owned report before crossing the boundary.

This inversion keeps native library loading and behavior invocation in plugin while keeping import envelope validation, output-kind validation, migration diagnostics, and `ImportedAsset` construction in asset. No alias or compatibility constructor retains the old asset-to-plugin dependency.

## Validation

`test_asset_native_importer_uses_neutral_command_host` passed after an explicit RED run. Rustfmt parsed/formatted the touched asset and plugin adapter files. The production dependency audit no longer reports `asset/importer/native.rs -> plugin`; managed Cargo execution remains deferred behind shared Windows validation lanes.
