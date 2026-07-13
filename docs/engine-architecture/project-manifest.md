---
related_code:
  - zircon_runtime_interface/src/project/mod.rs
  - zircon_runtime_interface/src/project/manifest_summary/summary.rs
  - zircon_runtime_interface/src/project/manifest_summary/migration.rs
  - zircon_runtime_interface/src/project/rel_path/value.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
implementation_files:
  - zircon_runtime_interface/src/project/manifest_summary/migration.rs
  - zircon_runtime_interface/src/project/manifest_summary/parse.rs
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/save.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_hub/src/projects/validation.rs
plan_sources:
  - user: 2026-07-11 Plan10 M1 slice 1.1 project manifest hard cut
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
tests:
  - zircon_runtime_interface/src/project/tests/manifest_summary.rs
  - zircon_runtime_interface/src/project/tests/rel_path.rs
  - zircon_runtime/src/asset/tests/project/manifest.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/tests.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - tests/fixtures/serialization/project-manifest/v1/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/v2/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/future/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/invalid/zircon-project.toml
doc_type: module-detail
---

# Project Manifest Architecture

## Purpose

`zircon-project.toml` is the shared project-identity authority for runtime, editor launch flow, and Zircon Hub. The runtime owns the complete business document, while `zircon_runtime_interface::project` owns neutral path and summary contracts so Hub can inspect a project without depending on `zircon_runtime`.

## Version Model

Manifest format version 2 is the only format written. `format_version` describes the TOML document structure. `library_version` is a separate business value describing generated asset-library contents; its historical `schema_version` input alias does not change the structural version.

Readers convert TOML into `serde_json::Value`, determine the source `format_version`, and pass that value through the shared serialization `MigrationChain`. The complete `0..2` chain is validated even when the input is already current. Version 0 is explicitly unsupported, version 1 is migrated by inserting the v2 defaults, and versions newer than 2 are rejected. There is no v1 Rust manifest structure, compatibility module, or alternate migration executor.

The v1 to v2 value migration sets:

- `format_version = 2`;
- `engine_version_req = null`;
- `asset_roots = ["assets"]` when absent;
- `settings = null` when absent.

`ProjectManifest::load_with_report` and `ProjectManifestSummary::parse_toml_*` return `Loaded<T>` and report `migrated_from = Some(1)` for real v1 input. Ordinary runtime load projects the migrated current value into `ProjectManifest`. Save clones the current value, forces structural version 2, validates roots, and emits stable pretty TOML.

## Shared Summary Boundary

`ProjectManifestSummary` contains only `name`, optional `engine_version_req`, textual `default_scene`, and `format_version`. Its parser validates UTF-8, TOML syntax, summary field shapes, library-version shape, safe project-root/settings paths, and `engine_version_req` with `semver::VersionReq`. Invalid requirements retain the original value and `semver::Error` source. Runtime projection and interface parsing use the same repository fixture and are contract-tested for equality.

Hub project validation now uses this summary parser. Consequently, syntactically valid TOML with a malformed project shape or a future format version is not considered a valid project. Hub project creation writes v2 with an explicit `asset_roots = ["assets"]` declaration.

## Asset Root Authority

`RelPath` stores normalized `/`-separated relative paths. It rejects empty input, absolute paths, drive/UNC prefixes, and `.` or `..` components. A manifest must contain at least one unique normalized asset root. After the roots exist, runtime canonicalizes both the project and every declared root; a symlink or junction that resolves outside the canonical project root is rejected with a typed error.

On open, `ProjectManager` registers the manifest roots in declaration order in `PackageAssetRegistry` and creates those directories. `ProjectPaths` has no fixed assets field or zero-argument root accessor; scaffolding must pass a `RelPath`, while live project behavior uses registered roots. Runtime/editor watchers subscribe to every root, font and editor reads require one existing match, and new editor/runtime writes call the explicitly named primary-root API. Existing model imports resolve their unique owning root so sibling animation products remain beside a model in a non-primary root. Editor project-path helpers preserve `AssetImportError` and resource-locator errors in the `EditorError` source chain. If two roots yield the same `res://` URI, typed lookup and scan errors preserve both paths. Package roots remain separately keyed by package id and continue to map to `package://`.

## Project authority and template packaging

Plan10 M1.2 adds `zircon_editor::core::project::ProjectAuthority` as the Editor owner for open/create/template/recent workflows. Editor and Hub consume one build-time packaged `templates/projects/renderable-empty/` truth through the neutral interface pack. Both create into staging and rename at commit; neither generates the template from large Rust strings or searches for a source checkout at runtime.

Recent project records in both applications store `ProjectManifestSummary`; independent display-name persistence was removed. Existing manifests are reparsed during import, Editor sync, and Hub config load so the summary is refreshed from project truth.

All regenerable project data now lives under `.zircon/{cache,registry,autosave,play,thumbnails}`. Physical `library/` ownership and its old path APIs were hard-deleted. `library_version` remains only the manifest's asset-content schema value.

Asset GUID/reference graph work remains owned by Plan10 M2.

## Test Status

The managed `zircon_runtime_interface` build and full test gate pass with 212 unit tests plus doc-tests. The first run exposed that the interface dependency allow-list had not yet registered `semver`; that current-plan failure was fixed and the complete gate reran green.

Runtime production build completed, but the runtime lib-test target currently stops before Plan10 tests because Render 11 source-cubemap tests call a removed `source_texel` API. Hub production build also passes, while its integration suite stops before the Summary unit tests because a Hub 07 contract still calls retired `HubMessage::legacy`. Both failures are recorded in their owning plans; this document does not claim the Runtime/Hub behavior gates passed.

The listed tests were added or updated as implementation contracts. Per the requested slice cadence, Cargo build and test commands were not run in this slice; milestone testing must execute the Plan10 M1 test matrix.
