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

Data-only preflight derives `ProjectIdentity = CanonicalDescriptorIdentity + ProjectGuid + ProjectManifestDigest` only from a current v3 document. `CanonicalDescriptorIdentity` carries the physical canonical descriptor path, never a UI display string, and rejects relative or dot-segment paths both at construction and deserialization. Legacy migration candidates keep only their canonical descriptor and migration decision; they have no admission identity.

## Version Model

Manifest format version 3 is the only format written. `format_version` describes the TOML document structure, while the required non-nil `project_guid` is the persistent project identity. `library_version` is a separate business value describing generated asset-library contents.

Readers convert TOML into `serde_json::Value`, determine the source `format_version`, and pass that value through the shared serialization `MigrationChain`. The complete `0..3` chain is validated even when the input is already current. Version 0 is explicitly unsupported, v1 adds the v2 structural defaults, v2 is raised to v3 for a data-only migration receipt, and versions newer than 3 are rejected. There is no v1/v2 Rust manifest structure, `schema_version` alias, compatibility module, or alternate migration executor.

The v1 to v2 value migration sets:

- `format_version = 2`;
- `engine_version_req = null`;
- `asset_roots = ["assets"]` when absent;
- `settings = null` when absent.

The v2 to v3 value migration only raises `format_version`; it never invents a project identity. `ProjectManifestSummary::parse_toml_*` returns `Loaded<T>` and reports `migrated_from = Some(1 | 2)` for legacy input so preflight can present an explicit conversion decision. `ProjectManifest::load_with_report` rejects every migration receipt with `MigrationRequired`; runtime never projects a legacy document into `ProjectManifest`. Save clones a current manifest, preserves its non-nil `project_guid`, forces structural version 3, validates roots, and emits stable pretty TOML.

## Shared Summary Boundary

`ProjectManifestSummary` contains `name`, optional `engine_version_req`, textual `default_scene`, `format_version`, and optional `project_guid`. The GUID is optional only so old manifests can be identified for explicit migration; a current v3 summary without one is invalid. Its parser validates UTF-8, TOML syntax, summary field shapes, library-version shape, safe project-root/settings paths, and `engine_version_req` with `semver::VersionReq`. Invalid requirements retain the original value and `semver::Error` source. Runtime projection and interface parsing use the same v3 repository fixture and are contract-tested for equality.

Hub project validation now uses this summary parser. Consequently, syntactically valid TOML with a malformed project shape, missing current GUID, or a future format version is not considered a current project. Hub and Editor project creation render v3 with a fresh persisted `project_guid` and an explicit `asset_roots = ["assets"]` declaration.

## Asset Root Authority

`RelPath` stores normalized `/`-separated relative paths. It rejects empty input, absolute paths, drive/UNC prefixes, and `.` or `..` components. A manifest must contain at least one unique normalized asset root. After the roots exist, runtime canonicalizes both the project and every declared root; a symlink or junction that resolves outside the canonical project root is rejected with a typed error.

On open, `ProjectManager` registers the manifest roots in declaration order in `PackageAssetRegistry` and creates those directories. `ProjectPaths` has no fixed assets field or zero-argument root accessor; scaffolding must pass a `RelPath`, while live project behavior uses registered roots. Runtime/editor watchers subscribe to every root, font and editor reads require one existing match, and new editor/runtime writes call the explicitly named primary-root API. Existing model imports resolve their unique owning root so sibling animation products remain beside a model in a non-primary root. Editor project-path helpers preserve `AssetImportError` and resource-locator errors in the `EditorError` source chain. If two roots yield the same `res://` URI, typed lookup and scan errors preserve both paths. Package roots remain separately keyed by package id and continue to map to `package://`.

## Project authority and template packaging

Plan10 M1.2 adds `zircon_editor::core::project::ProjectAuthority` as the Editor owner for open/create/template/recent workflows. Editor and Hub consume one build-time packaged `templates/projects/renderable-empty/` truth through the neutral interface pack. Both create into staging and rename at commit; neither generates the template from large Rust strings or searches for a source checkout at runtime.

Recent project records in both applications store `ProjectManifestSummary`; independent display-name persistence was removed. Existing manifests are reparsed during import, Editor sync, and Hub config load so the summary is refreshed from project truth.

All regenerable project data now lives under `.zircon/{cache,registry,autosave,play,thumbnails}`. Physical `library/` ownership and its old path APIs were hard-deleted. `library_version` remains only the manifest's asset-content schema value.

Asset GUID/reference graph work remains owned by Plan10 M2.

## Test Status

The previously recorded interface, Runtime, and Hub command results predate the v3 hard cut and are historical evidence only; they do not validate this contract. The listed tests are implementation contracts. No Cargo build or test command has run for the v3 slice, so a coordinator-managed Windows matrix must rerun the interface, Runtime, Editor, Hub, and template-create paths before this boundary can be accepted.
