---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/project/mod.rs
  - zircon_runtime_interface/src/project/asset_ref/value.rs
  - zircon_runtime_interface/src/project/asset_ref/construction.rs
  - zircon_runtime_interface/src/project/asset_ref/serde.rs
  - zircon_runtime_interface/src/project/asset_ref/validation.rs
  - zircon_runtime_interface/src/project/rel_path/value.rs
  - zircon_runtime_interface/src/project/rel_path/parse.rs
  - zircon_runtime_interface/src/project/rel_path/deserialize.rs
  - zircon_runtime_interface/src/project/manifest_summary/summary.rs
  - zircon_runtime_interface/src/project/manifest_summary/parse.rs
  - zircon_runtime_interface/src/project/manifest_summary/migration.rs
  - zircon_runtime_interface/src/project/template_pack/mod.rs
  - zircon_runtime_interface/src/project/template_pack/embedded.rs
  - templates/projects/renderable-empty/zircon-project.toml
  - zircon_runtime_interface/src/serialization/migration/execute.rs
implementation_files:
  - zircon_runtime_interface/src/project/asset_ref/value.rs
  - zircon_runtime_interface/src/project/asset_ref/construction.rs
  - zircon_runtime_interface/src/project/asset_ref/serde.rs
  - zircon_runtime_interface/src/project/asset_ref/validation.rs
  - zircon_runtime_interface/src/project/rel_path/value.rs
  - zircon_runtime_interface/src/project/rel_path/parse.rs
  - zircon_runtime_interface/src/project/manifest_summary/summary.rs
  - zircon_runtime_interface/src/project/manifest_summary/parse.rs
  - zircon_runtime_interface/src/project/manifest_summary/migration.rs
  - zircon_runtime_interface/src/project/template_pack/render.rs
  - zircon_runtime_interface/src/project/template_pack/embedded.rs
  - zircon_runtime_interface/src/serialization/migration/execute.rs
plan_sources:
  - user: 2026-07-11 Plan10 M2 slice 2.1 AssetRef and zmeta v7 hard cut
  - user: 2026-07-11 Plan10 M1 slice 1.1 neutral summary contract
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
tests:
  - zircon_runtime_interface/src/project/tests/asset_ref.rs
  - zircon_runtime_interface/src/project/tests/manifest_summary.rs
  - zircon_runtime_interface/src/project/tests/rel_path.rs
  - zircon_runtime_interface/src/project/tests/template_pack.rs
  - zircon_runtime_interface/src/serialization/tests/migration_contract.rs
  - tests/fixtures/serialization/project-manifest/v1/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/v2/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/future/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/invalid/zircon-project.toml
doc_type: module-detail
---

# Runtime Interface Project Contracts

## Purpose

The `project` module provides project data that tools may consume without linking the runtime business implementation. Its public surface includes the safe `RelPath`, canonical persistent `AssetRef`, `ProjectManifestSummary`, the current manifest-format constant, and the shared TOML-to-value migration adapter used by runtime.

## RelPath

`RelPath` normalizes `\` to `/` and collapses repeated separators. It rejects empty results, absolute paths, drive or UNC prefixes, and any `.` or `..` component. Serialization is a string; deserialization always re-runs validation. `join_to` builds a platform path only after those invariants hold.

This type is deliberately neutral: it does not know whether the path names assets, settings, or another project-owned file. Runtime adds the root ownership and duplicate-vector rules.

## AssetRef

`AssetRef` is the neutral persistent reference DTO `{ guid, path_hint, sub }`. `guid` is the existing `AssetUuid` identity, `path_hint` is a validated `RelPath`, and `sub` is an optional subasset path. Construction and deserialization reject empty subpaths, `#` fragment delimiters, control characters, project-path traversal, and unknown human-readable keys. JSON locks the exact three-key shape and deterministic bincode roundtrip locks the binary DTO path. Fields stay invariant-protected behind accessors; there is no URI-derived identity fallback, conversion, alias, or re-export involving the separate `resource::AssetReference` surface retained until Plan10 M2.3.

## ProjectManifestSummary

The summary carries:

- project `name`;
- optional `engine_version_req`;
- textual `default_scene` URI;
- structural `format_version`.

`parse_toml_str` and `parse_toml_bytes` are lightweight, typed-error entry points. They still validate the required library-version shape and safe asset-root/settings fields even though those fields are not retained in the summary. Extra full-manifest fields such as plugins, scripts, and export profiles remain runtime-owned and are ignored by the summary projection.

When `engine_version_req` is present it must parse as `semver::VersionReq`. The typed failure carries both the rejected string and the original `semver::Error`; Hub therefore rejects syntactically valid TOML with an invalid engine requirement.

## Migration Adapter

The TOML adapter converts `toml::Value` to `serde_json::Value`, reads the structural source version, rejects future input, and calls `MigrationChain::migrate_value`. That serialization API is public specifically for non-JSON format adapters and always validates the entire declared chain before executing a suffix of steps. It prevents TOML consumers from creating a second migration framework or bypassing chain validation for current documents.

The v1-to-v2 function operates only in the value domain. No legacy Rust manifest type exists. The adapter returns `Loaded<Value>`, allowing both summary and runtime callers to expose `migrated_from` consistently.

## Packaged project templates

`project::template_pack` is the neutral build-time packaging owner shared by Editor and Hub. It embeds the single version-controlled `templates/projects/renderable-empty/` directory, exposes safe `RelPath` entries, rewrites only the manifest `name` through typed TOML values, and validates the rendered result with `ProjectManifestSummary`. Consumers never locate repository source paths at runtime and do not carry duplicate template strings.

The interface boundary continues to forbid arbitrary `include_bytes!`; the only reviewed exception is `project/template_pack/embedded.rs`. Its test compares the embedded entry set with every source file and rejects source links/reparse points.

## Failures

Errors distinguish invalid UTF-8, invalid TOML, invalid field shape, invalid values, invalid/future versions, and migration-chain failure. Future-version refusal occurs before current-type deserialization.

## Test Status

Contracts share repository v1/v2/future/invalid fixtures across interface, runtime, and Hub. They cover migration reporting, projection equality, future refusal, semver source preservation, malformed shapes, safe roots, and complete-chain validation. M2.1 adds source tests for AssetRef exact JSON keys, unknown-key rejection, deterministic bincode roundtrip, guid/path/sub semantics, invalid subpaths, and traversing `path_hint`; Cargo execution is deferred to the milestone testing stage. The earlier managed interface gate remains 212/212 plus doc-tests; Runtime and Hub upward gates remain unclaimed because their test targets are blocked by failures archived to Render 11 and Hub 07 respectively.
