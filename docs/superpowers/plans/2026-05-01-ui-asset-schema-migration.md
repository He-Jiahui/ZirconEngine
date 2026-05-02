# UI Asset Schema Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement M7 by turning UI asset schema migration/version policy into production runtime code with structured reports and future-version rejection.

**Architecture:** Add a folder-backed `zircon_runtime::ui::template::asset::schema` owner under the existing runtime UI asset module. The schema owner classifies source TOML shapes, migrates supported old/flat/legacy inputs to current recursive tree authority, returns structured reports, and keeps artifact packaging and editor hot-reload concerns out of this milestone.

**Tech Stack:** Rust, Serde, TOML, existing `zircon_runtime::ui::template::asset` types, existing runtime UI asset tests, Markdown docs.

---

## Affected Files

- Create `zircon_runtime/src/ui/template/asset/schema/mod.rs`: structural schema module root and public re-exports.
- Create `zircon_runtime/src/ui/template/asset/schema/policy.rs`: source schema constants and version decision helpers.
- Create `zircon_runtime/src/ui/template/asset/schema/report.rs`: migration report, source kind, steps, diagnostics, and outcome types.
- Create `zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs`: production flat `[nodes]` to recursive tree migration.
- Create `zircon_runtime/src/ui/template/asset/schema/legacy_template.rs`: production legacy `UiTemplateDocument` to `UiAssetDocument` conversion.
- Create `zircon_runtime/src/ui/template/asset/schema/migrator.rs`: `UiAssetSchemaMigrator` facade and TOML shape classification.
- Modify `zircon_runtime/src/ui/template/asset/mod.rs`: add `schema` module and re-export stable schema API.
- Modify `zircon_runtime/src/ui/template/mod.rs`: re-export stable schema API from the public template surface.
- Modify `zircon_runtime/src/ui/template/asset/loader.rs`: route loader through schema migrator and add report-returning APIs.
- Modify `zircon_runtime/src/ui/template/asset/document.rs`: add structured schema-version/migration errors.
- Create `zircon_runtime/src/ui/tests/asset_schema_migration.rs`: focused tests for current, older, flat, future, and legacy migration paths.
- Modify `zircon_runtime/src/ui/tests/mod.rs`: register the new test module.
- Modify `zircon_runtime/src/ui/tests/asset.rs`: remove direct use of test-only migration helpers where production APIs now own behavior.
- Delete or reduce `zircon_runtime/src/ui/tests/asset_migration_support.rs`: keep only wrappers if existing tests still need them; otherwise remove module registration.
- Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md`: record schema migration/version policy.
- Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`: record M7 progress only after implementation and validation evidence exists.

## Milestone 1: Production Schema Migration

- Goal: Runtime UI asset loading has a production schema/version policy and a structured migration report.
- In-scope behaviors: current tree no-op report, older tree version bump to source schema version `3`, flat `[nodes]` materialization, legacy template conversion, future-version rejection, loader report API.
- Dependencies: existing `UiAssetDocument`, `UiTemplateDocument`, `UiAssetLoader`, tree authority validation, and runtime UI asset tests.

### Implementation Slices

- [ ] Add focused M7 test code under `zircon_runtime/src/ui/tests/asset_schema_migration.rs`.
- [ ] Add schema report and policy types with `UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION = 3`.
- [ ] Move flat node-table migration into production `flat_nodes.rs`.
- [ ] Move legacy template conversion into production `legacy_template.rs`.
- [ ] Implement `UiAssetSchemaMigrator::migrate_toml_str(...)` and shape classification.
- [ ] Wire `UiAssetLoader` through the migrator and add `load_toml_str_with_migration_report(...)` / `load_toml_file_with_migration_report(...)`.
- [ ] Add structured `UiAssetError::UnsupportedSchemaVersion` and `UiAssetError::SchemaMigrationFailed` variants.
- [ ] Remove or shrink test-only migration support so tests use production APIs.
- [ ] Update UI asset protocol docs and the M7 archive entry with actual implementation files and validation commands.

### Testing Stage

- Run targeted formatting for changed Rust files:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/asset/loader.rs zircon_runtime/src/ui/template/asset/document.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/asset.rs zircon_runtime/src/ui/tests/asset_schema_migration.rs zircon_runtime/src/ui/template/asset/schema/*.rs
```

- Run focused runtime UI asset tests:

```powershell
cargo test -p zircon_runtime --lib ui::tests::asset_schema_migration --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-schema-migration --message-format short --color never -- --nocapture
```

- Run existing runtime UI asset tests that depend on loader/migration behavior:

```powershell
cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-schema-migration --message-format short --color never -- --nocapture
```

- Run scoped runtime type check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-schema-migration --message-format short --color never
```

- Run whitespace check for touched docs/source paths:

```powershell
git diff --check -- zircon_runtime/src/ui/template/asset zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md ".codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md" docs/superpowers/specs/2026-05-01-ui-asset-schema-migration-design.md docs/superpowers/plans/2026-05-01-ui-asset-schema-migration.md
```

### Debug / Correction Loop

- If `asset_schema_migration` fails, fix the lowest schema classification or migration helper first, then rerun only `asset_schema_migration`.
- If `asset` fails, inspect whether loader version bump changed a test expectation; update tests only when the new source-schema policy is the intended behavior.
- If `cargo check` fails outside `zircon_runtime::ui::template::asset`, confirm whether it is active-session drift before editing unrelated modules.

### Exit Evidence

- `asset_schema_migration` focused tests pass.
- Existing `ui::tests::asset` pass with the production migrator.
- `cargo check -p zircon_runtime --lib` passes or any blocker is explicitly documented as unrelated active-session drift.
- Docs and archive record M7 status, implementation files, and validation evidence.

## Acceptance Boundaries

- M7 can move from `Mostly missing` only after production schema APIs, report types, migration behavior, future-version rejection, focused tests, and docs are present.
- M7 is not complete until the testing stage evidence above is recorded.
- M5 hot reload/conflict handling and M16 artifact/package validation remain separate milestones.
