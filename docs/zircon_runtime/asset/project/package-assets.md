---
related_code:
  - zircon_runtime/src/asset/project/manager/package_assets.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
implementation_files:
  - zircon_runtime/src/asset/project/manager/package_assets.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - zircon_runtime/src/asset/tests/project/zmeta/package_roots.rs
  - tools/tests/test_frameworks_05_layer_direction.py
---

# Package Asset Roots

`PackageAssetRegistry` maps canonical package ids to physical filesystem asset roots for `package://` resource resolution. Its input is the minimal asset projection: package id, declared relative asset roots, and package filesystem root. It validates exactly one contained relative root, resolves the deepest existing filesystem ancestor once, and stores that physical identity; junction, SUBST, and symlink aliases therefore stay below the resolver boundary.

The registry no longer accepts `PluginPackageManifest`. Plugin/editor assembly reads `package_id()` and `asset_roots_or_default()` at its own boundary, then passes those values to `ProjectManager::register_package_asset_roots(...)`. This keeps package manifest ownership and validation in plugin while asset owns only filesystem/resource-locator policy.

The former `register_package_manifest_asset_roots(...)` and `register_manifest_roots(...)` APIs were deleted without aliases or compatibility forwarding. Tests use manifest fixtures only at the upper test assembly boundary or pass explicit invalid roots directly to the asset API.

## Validation

`test_asset_package_roots_do_not_accept_plugin_manifests` passed after an explicit RED run; retired API scans are empty and rustfmt passed. Together with the native command-host inversion, this removed three production `asset -> plugin` references. The corrected grouped-use audit still reports two project-document references, which remain open and are not hidden by this slice.
