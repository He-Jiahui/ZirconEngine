# UI Asset Schema Migration Design

## Summary

This spec covers M7 from `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`: make UI asset schema migration and version policy a production runtime subsystem instead of a test-only helper. The first repair slice targets `zircon_runtime::ui::template::asset` and deliberately avoids editor workspace hot reload, Runtime UI data-source adapters, graphics, plugin, physics, and animation areas owned by active sessions.

The desired outcome is a formal `UiAssetSchemaMigrator` that can classify current tree assets, older tree assets, flat node-table assets, legacy template fixtures, and future-version assets; migrate supported old forms into current tree authority; produce a structured migration report; and reject unsupported future schema versions with enough information for editor read-only handling later.

## Goals

- Establish UI asset source schema version policy in production code.
- Move flat node-table and legacy template fixture migration out of test-only support into a runtime-owned schema module.
- Keep `asset.version` as the source TOML schema version, separate from future compiled artifact and importer versions.
- Provide a migration report that editor and importer code can use without parsing error strings.
- Preserve the hard rule that current compiled/runtime consumers receive a valid `UiAssetDocument` with recursive tree authority.

## Non-Goals

- Do not implement M16 compiled artifact headers, dependency manifests, runtime/editor stripping, or package validation reports in this slice.
- Do not implement M5 workspace hot reload, `ExternalConflict`, reload/keep-local/diff flows, or stale import UI behavior.
- Do not migrate representative editor panes or runtime screens in this slice.
- Do not restore legacy `.slint` business sources or old flat `[nodes]` as a long-term accepted authoring format.

## Current Baseline

- `UiAssetDocument` already has `UiAssetKind::{Layout, Widget, Style}` and recursive tree/component/slot data under `zircon_runtime/src/ui/template/asset/document.rs`.
- `UiAssetLoader::load_toml_str(...)` currently parses directly to `UiAssetDocument` and validates tree authority.
- Flat `[nodes]` migration currently lives in `zircon_runtime/src/ui/tests/asset_migration_support.rs`, which means production importer/editor code cannot share the behavior.
- Legacy template fixture conversion is also test-support owned, even though it encodes real migration knowledge.
- `asset.version` is a plain `u32` defaulting to `1`; there is no current/min/future schema policy, no migration report, and no read-only future-version state.

## Architecture

Add a folder-backed schema submodule under `zircon_runtime/src/ui/template/asset/schema/`.

Initial module responsibilities:

- `policy.rs`: source schema constants and version policy.
- `report.rs`: migration report, source kind, step, and diagnostic data structures.
- `migrator.rs`: public `UiAssetSchemaMigrator` facade.
- `flat_nodes.rs`: flat node-table importer/migration implementation.
- `legacy_template.rs`: legacy template fixture importer/migration implementation.

The parent `asset/mod.rs` re-exports only stable public types such as `UiAssetSchemaMigrator`, `UiAssetMigrationReport`, and `UiAssetSchemaSourceKind`. Internal helper structures stay private to the schema folder.

## Public Types

The first implementation introduces these stable concepts:

- `UiAssetSchemaVersionPolicy`: source schema policy with current source version, minimum supported version, and future-version decision helpers.
- `UiAssetSchemaMigrator`: production entry point for migration and classification.
- `UiAssetMigrationOutcome`: contains `document: UiAssetDocument` and `report: UiAssetMigrationReport` for successful migrations.
- `UiAssetMigrationReport`: records detected source kind, source schema version, target schema version, migration steps, diagnostics, and whether structural editing is allowed.
- `UiAssetSchemaSourceKind`: at minimum `CurrentTree`, `OlderTree`, `FlatNodeTable`, `LegacyTemplateFixture`, and `FutureVersion`.
- `UiAssetMigrationStep`: stable labels for no-op current load, source version bump, flat tree materialization, and legacy template conversion.
- `UiAssetSchemaDiagnostic`: structured warning/error entries for migration notes that must not be reduced to free-form strings.

Compiled artifact and importer versioning are not fully implemented here. The slice can add lightweight constants or newtypes such as `UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION` and `UI_ASSET_CURRENT_IMPORTER_VERSION`, but artifact headers remain an M16 concern.

## Loader Contract

The loader exposes both document-only and report-returning paths.

- `UiAssetLoader::load_toml_str(...)` remains the simple API for callers that only need a current `UiAssetDocument`.
- `UiAssetLoader::load_toml_str_with_migration_report(...)` returns the document and report for editor/importer callers.
- `UiAssetSchemaMigrator::migrate_toml_str(...)` is the direct schema subsystem API.

Supported older source forms are migrated before final `validate_tree_authority()`. Future versions must not silently downgrade. They return a clear schema-version error with source and current version data through the report-capable migrator path.

## Version Policy

- `version == current`: return the parsed document unchanged except for normal validation; report a no-op current-load step.
- `version < current`: migrate to current source schema, update `asset.version` to the current version, validate tree authority, and report every step.
- flat `[nodes]` shape: classify as `FlatNodeTable`, materialize recursive `root.children` and component roots, remove flat authority tables from emitted canonical source, then validate.
- legacy template fixture shape: classify as `LegacyTemplateFixture`, convert to a layout `UiAssetDocument`, and mark the report as importer-style migration rather than a formal authoring format.
- `version > current`: reject structured load/edit. The report must preserve the future version and current supported version so the editor can later open source read-only and offer save-as/downgrade choices only when a dedicated migrator exists.

## Error Handling

Add structured schema/version errors to `UiAssetError` rather than forcing editor callers to inspect `ParseToml` or `InvalidDocument` text.

Required additions:

- `UnsupportedSchemaVersion { asset_id, version, current }`
- `SchemaMigrationFailed { asset_id, detail }` if a supported migration path detects invalid source data

If returning a full report inside `UiAssetError` would make the error type too large, keep the error concise and provide the richer report through `UiAssetSchemaMigrator` outcome APIs.

## Testing Design

Tests move from test-only migration helpers to production migration APIs.

Required focused tests:

- current tree asset returns a no-op migration report and validates.
- version-1 tree asset migrates to current source schema version and reports the version bump.
- flat `[nodes]` asset migrates to recursive tree authority and no longer emits flat tables in canonical output.
- future-version asset is rejected with structured version data.
- legacy template fixture conversion goes through `UiAssetSchemaMigrator`, produces a layout document, and reports legacy-template source kind.

Per the repository milestone-first policy, implementation can add these tests during the slice, but broad Cargo execution belongs to the M7 testing stage. The M7 testing stage runs focused `zircon_runtime` UI asset tests and a scoped runtime check before the milestone can be marked complete.

## Documentation Updates

- Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` with the schema migration and future-version read-only policy.
- Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` after implementation to change M7 only when production APIs and focused tests exist.
- If the schema module grows beyond the first slice, add a source-mirrored module document under `docs/zircon_runtime/ui/template/asset/schema.md` or a folder-backed equivalent.

## Acceptance Criteria

- Production code exposes `UiAssetSchemaMigrator` and migration report types from the runtime UI asset module.
- Test-only flat and legacy conversion helpers are removed or reduced to wrappers around production APIs.
- Future schema versions are rejected through a structured version policy, not generic parse or invalid-document errors.
- Supported old/flat/legacy inputs produce current tree-authority `UiAssetDocument` values and structured reports.
- M7 remains separate from M5 hot reload/conflict handling and M16 artifact packaging.
