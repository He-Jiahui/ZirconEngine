---
related_code:
  - zircon_runtime_interface/src/project/retired_asset_ref_migration
  - zircon_runtime_interface/src/project/asset_ref
  - zircon_runtime/src/scene/reflect/json_document/migration.rs
  - zircon_runtime/src/scene/dynamic_scene/document/migration/mod.rs
  - zircon_runtime/src/asset/migration/document.rs
implementation_files:
  - zircon_runtime_interface/src/project/retired_asset_ref_migration/mod.rs
  - zircon_runtime_interface/src/project/retired_asset_ref_migration/error.rs
  - zircon_runtime_interface/src/project/retired_asset_ref_migration/retired_asset_reference.rs
  - zircon_runtime_interface/src/project/retired_asset_ref_migration/migrate.rs
plan_sources:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime_interface/src/project/tests/retired_asset_ref_migration.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/document_migration.rs
  - tests/fixtures/serialization/scene-reflection/v0/reflected-value.json
  - tests/fixtures/serialization/scene-dynamic/v0/dynamic-scene.json
doc_type: module-detail
---

# Retired Asset Reference Value Migration

`zircon_runtime_interface::project` owns the only recursive value walker for the retired exact `{ uuid, url }` persistent-reference shape. The walker does not depend on Runtime, the asset registry, a filesystem, or an editor host. It validates the UUID and resource locator, then projects the value to the discriminated `PersistedAssetReference` contract: `project` embeds the current `AssetRef` DTO, while `builtin` embeds only its stable locator.

Objects with extra or missing keys are ordinary domain values and are not guessed as references. Malformed exact shapes return `RetiredAssetRefMigrationError::InvalidShape`; resolver failures remain typed as `RetiredAssetRefMigrationError::Resolve`. There is no retired-format reader, alias, or second heuristic walker.

Plan 11 uses the context-free wrapper for v0 scene documents, where the old format defines `res://x` as `assets/x`. The Plan 10 commandlet uses the resolver form so the authoritative read-only registry and current multi-root manifest choose the exact GUID, physical project-relative path hint, and optional subasset path. Builtin values bypass the project registry and can therefore survive save/reload without pretending to be project assets.
