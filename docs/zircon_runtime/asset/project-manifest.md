---
related_code:
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/save.rs
  - zircon_runtime/src/asset/project/manifest/validation.rs
  - zircon_runtime/src/core/framework/project/mod.rs
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/mod.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/project/manager/source_uri_for_path.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
implementation_files:
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/save.rs
  - zircon_runtime/src/asset/project/manifest/validation.rs
  - zircon_runtime/src/core/framework/project/mod.rs
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/mod.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/scene/module/mod.rs
plan_sources:
  - user: 2026-07-11 Plan10 M1 slice 1.1 runtime manifest and roots
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
tests:
  - zircon_runtime/src/asset/tests/project/manifest.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/tests.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - tests/fixtures/serialization/project-manifest/v1/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/v2/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/future/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/invalid/zircon-project.toml
doc_type: module-detail
---

# Runtime Project Manifest

## Ownership and Shape

The Asset project subtree owns the filesystem-backed `ProjectManifest`, loading, migration, validation, and save behavior. Shared `plugins`, `export_profiles`, packaging/platform policy, and runtime-profile id fields use the single neutral owner under `core/framework/project`; Asset no longer imports Plugin facade schema. The module is folder-backed: `manifest.rs` is wiring only, while declarations and I/O have focused owners.

Version 2 adds `engine_version_req: Option<String>`, `asset_roots: Vec<RelPath>`, and `settings: Option<RelPath>`. Missing roots migrate/default to `assets`. Structural `format_version` and content `library_version` remain intentionally independent.

## Load and Save Flow

`from_toml_str` delegates TOML value migration to `zircon_runtime_interface::project`, deserializes only the resulting v2 value into the runtime type, then validates cross-field invariants such as non-empty unique roots. `load_with_report` preserves the migration source version. `load` returns the current manifest for existing runtime consumers but uses the same migrating path; it is not a legacy parser.

Save validates the manifest, forces `format_version = 2` on the serialized copy, creates the parent directory, and writes pretty TOML. It never emits a v1 document.

## Root Registration and Scanning

`ProjectManager::open` loads the manifest before creating asset directories. It creates every declared project asset root plus the regenerable `.zircon/{cache,registry,autosave,play,thumbnails}` layout, then canonicalizes the project and roots before registration. A symlink or junction that resolves outside the canonical project root is rejected through typed `AssetImportError` variants. Roots that remain contained are registered in declaration order. `ProjectPaths` exposes explicit derived, cache, asset-artifact, registry, autosave, play, and thumbnail roots; the retired physical `library/` root and its accessors were deleted.

Binary imported artifacts live under `.zircon/cache/assets`, registry state belongs under `.zircon/registry`, and editor previews/atlases use named children below `.zircon/cache`. These roots are not aliases: each describes one derived-data responsibility.

The scanner and runtime watcher cover every registered project root. URI construction strips the specific owning root, so all roots share the `res://` namespace. Existing-source lookup and source-root ownership return only a unique match; missing and ambiguous cases are typed. New destinations use `primary_project_source_path_for_uri` or `existing_or_primary_project_source_path_for_uri`, making first-root selection explicit rather than a hidden fallback. Editor model animation derivation uses source-root ownership, so a model in the second root writes sibling products back to that same root.

Package roots retain their existing `package://<id>/` namespace and are not mixed into project duplicate checks.

## Runtime Session Prepared-Project Ownership

Runtime session startup prepares the filesystem project exactly once before linked-plugin
selection. `RuntimeProjectConfig::prepare` opens one `ProjectManager`, captures its validated
manifest into `RuntimePreparedProject`, and keeps the manager instance until the Asset module is
active. Plugin selection reads that manifest by reference. The Asset module then takes ownership of
the same manager through the abstract `AssetManager::open_prepared_project` service operation;
`ProjectAssetManager` remains its concrete Runtime04 implementation owner. Startup scripts and the
default-scene URI continue to read the same captured manifest.

The public path-based `AssetManager::open_project` route constructs a `ProjectManager` and delegates
to `open_prepared_project`, so importer registration, scanning, resource synchronization, watcher
replacement, and change publication have one behavior owner. There is no second path-based reopen,
compatibility wrapper, or fallback manifest parser inside dynamic session construction.

After activation, Scene loads the default level through `AssetManager::current_project_snapshot`.
The service clones the already scanned `ProjectManager` while holding its read lock, releases that
lock, and only then lets the level owner perform scene I/O. Scene URI resolution therefore uses the
activated manifest, roots, registry, and importer results without another `ProjectManager::open` /
`scan_and_import` cycle, while arbitrary file I/O or re-entrant service calls cannot run under the
manager lock. The one explicit startup snapshot copy is the correctness boundary until the project
registry has an immutable shared-storage representation.

This ownership prevents a project file changed during startup from selecting plugins from one
manifest revision while loading scripts or a scene from another. It removes repeated manifest
parsing and does not clone the project registry during activation transfer; the later scene snapshot
copy is explicit and lock-safe. A second attempted transfer returns a typed `RuntimeProjectError`;
it cannot panic or silently reopen the path.

## Constraints

- A root is a normalized `RelPath`, never an unchecked string.
- The root vector cannot be empty or contain duplicates after normalization.
- Lexically resolved and canonical roots must remain below the corresponding project root; links and junctions cannot escape it.
- Two physical sources cannot silently claim the same `res://` URI.
- Production manifest code contains no v1 DTO, panic/expect/unwrap fallback, or alternate migration chain.

## Test Coverage and Status

Manifest tests cover a real v1 migration report, future rejection, stable v2 persistence, and equality with the interface summary projection. Project tests cover default and explicit ordered root registration, canonical link escape rejection, successful two-root scan, duplicate URI rejection, and a real watcher event emitted from the second root. Editor tests cover animation derivatives remaining beside a model in a non-primary root and typed project-path error sources. The interface managed build/test gate passes 212/212 plus doc-tests. Runtime production build completes, but its lib-test target is currently blocked before these tests by Render 11 source-cubemap test API drift; the failure is archived under that plan and no Runtime test pass is claimed.

The Runtime10 foundation contract `project_session_startup_reuses_one_prepared_project_manager_snapshot`
locks the abstract single prepare/transfer route and rejects a second scene open/scan.
`project_startup_snapshot_survives_disk_manifest_rewrite_before_activation` rewrites the disk
manifest after preparation and verifies that startup consumers retain the first validated scene
selection; `project_startup_snapshot_survives_disk_manifest_rewrite_after_activation` performs the
same mutation after real trait activation and verifies that the service snapshot still observes the
activated revision. The shared `project_startup_snapshot_survives_disk_manifest_rewrite` filter
runs both behavior cases in one focused lib-test invocation.
The managed Windows production check passes. The focused default-feature lib-test executes both
disk-rewrite cases and reports 2 passed / 0 failed / 8185 filtered; exact job and run identifiers are
recorded in Runtime10 status. The earlier independently owned Text fixture compile failure is kept
as historical failed-attempt evidence and is not used as acceptance.
