---
related_code:
  - zircon_runtime/src/asset/registry/mod.rs
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime/src/asset/registry/asset_registry_entry.rs
  - zircon_runtime/src/asset/registry/asset_registry_filter.rs
  - zircon_runtime/src/asset/registry/asset_registry_diagnostic.rs
  - zircon_runtime/src/asset/registry/asset_registry_error.rs
  - zircon_runtime/src/asset/registry/query.rs
  - zircon_runtime/src/asset/registry/rebuild.rs
  - zircon_runtime/src/asset/registry/incremental.rs
  - zircon_runtime/src/asset/registry/persistence.rs
  - zircon_runtime/src/asset/registry/dependency_extractors/mod.rs
  - zircon_runtime/src/asset/project/manager/mod.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/foundation/persistence/atomic_file.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
implementation_files:
  - zircon_runtime/src/asset/registry/mod.rs
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime/src/asset/registry/asset_registry_entry.rs
  - zircon_runtime/src/asset/registry/asset_registry_filter.rs
  - zircon_runtime/src/asset/registry/asset_registry_diagnostic.rs
  - zircon_runtime/src/asset/registry/asset_registry_error.rs
  - zircon_runtime/src/asset/registry/query.rs
  - zircon_runtime/src/asset/registry/rebuild.rs
  - zircon_runtime/src/asset/registry/incremental.rs
  - zircon_runtime/src/asset/registry/persistence.rs
  - zircon_runtime/src/asset/registry/dependency_extractors/mod.rs
  - zircon_runtime/src/asset/registry/dependency_extractors/scene.rs
  - zircon_runtime/src/asset/registry/dependency_extractors/material.rs
  - zircon_runtime/src/asset/registry/dependency_extractors/model.rs
plan_sources:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/asset/tests/registry_index/queries.rs
  - zircon_runtime/src/asset/tests/registry_index/persistence.rs
  - zircon_runtime/src/asset/tests/registry_index/incremental.rs
  - zircon_runtime/src/asset/tests/registry_index/dependency_extractors.rs
  - zircon_runtime/src/asset/tests/registry_index/scan_safety.rs
  - zircon_runtime/src/asset/tests/project/zmeta/schema_v7.rs
  - zircon_runtime/src/asset/tests/project/zmeta/metadata_lifecycle.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
doc_type: module-detail
---

# Asset Registry Index

## Purpose

`AssetRegistryIndex` is the runtime-owned, payload-free authority for project asset identity, discovery, and dependency queries. It replaces the three UUID/path lookup maps formerly embedded in `ProjectManager`; callers now enter through `ProjectManager::asset_registry()` and query the index directly. There is no compatibility module, forwarding API, or fallback to the retired `project/manager/asset_lookup.rs` surface.

The registry is available to editor, runtime loading, cook, deletion checks, and future fix-up tooling without deserializing imported asset bodies. Its persistent copy is regenerable state under `.zircon/registry/asset-registry.json`.

## Data Model

Each `AssetRegistryEntry` stores the stable `AssetUuid`, canonical `AssetUri` path (including `package://` and subasset labels), `AssetKind` type marker, sorted tag set, direct dependency UUIDs, and the v7 sidecar `source_digest`. `AssetMetaDocument::tags` is authoritative for a root asset and `AssetMetaEntry::tags` is authoritative for each labeled subasset. Existing current-v7 sidecars decode an absent tag set as empty; the next sidecar save writes `tags = []` or the sorted values explicitly. This is one current schema, not an old-version reader, alias, or migration layer.

Import preserves root and labeled-entry tags when artifacts are restored, reimported, or retained after an import failure. Registry rebuild and corruption recovery always repopulate tag filters from the sidecars; the regenerable registry JSON is never the authoring authority for tags. The in-memory index owns UUID and path maps, while reverse references are derived from dependency UUIDs without loading payloads.

`AssetRegistryDiagnostic` is the typed warning surface. It currently reports corrupt persistence recovery, duplicate GUID reminting, and unresolved dependency paths. A duplicate GUID is never silently overwritten. During an incremental copy/import, the UUID owner already present in the current index keeps its identity and only the new path is reminted, even when that new path sorts first. During an initial rebuild with no prior owner, deterministic root/path order selects the owner. The replacement is written back to the copied `.zmeta` sidecar before resource IDs are derived.

Root and subasset tags are validated before deserialization into sets. Empty tags, surrounding whitespace, control characters, and duplicate serialized values are rejected with typed metadata errors. `AssetMetaDocument` and `AssetMetaEntry` deserialize through private raw DTOs and custom `Deserialize` boundaries, so direct `toml::from_str` calls cannot bypass validation; `BTreeSet` is constructed only after the raw authoring document passes.

## Query Contract

The public read surface implements the six UE-style offline signatures:

- `get_assets_by_type` for type/class discovery;
- `get_assets` for compound type, tag, path-prefix, and package filtering;
- `get_dependencies_by_uuid` and `get_dependencies_by_path`;
- `get_referencers_by_uuid` and `get_referencers_by_path`.

Identity helpers (`resolve_asset_id_by_uuid`, `resolve_asset_id_by_path`, `resolve_asset_id_for_reference`, and `resolve_reference_by_asset_id`) return typed `Result` values from the registry. The old ambiguous `resolve_* -> Option` surface and `ProjectManager` forwarding wrappers were deleted; these fallible resolvers retain typed failure detail. `resolve_asset_id_for_reference` keeps the persistent-reference rule of UUID first and path hint second, while stale path reporting is explicit through `stale_path_for_uuid`.

## Rebuild and Persistence Flow

Opening a project loads the versioned JSON document when it is valid. Missing or malformed persistence triggers a metadata-only rebuild from every manifest project asset root. Package roots registered later are included in scan-time registry rebuilds. A rebuild reads `.zmeta` documents, normalizes duplicate identities, creates entries, resolves dependency paths through the complete path map, records unresolved edges, and rewrites the regenerable JSON file.

The persistence schema uses `deny_unknown_fields` and an exact format version. Old registry paths and old persistence schemas are not searched. Corrupt content is not partially trusted or merged; it is replaced by a clean rebuild and a typed recovery diagnostic. Decode failures retain the `serde_json` source, and version failures report both the found and supported versions.

Registry persistence and v7 sidecars both use the foundation-owned atomic-file transaction in `foundation/persistence/atomic_file.rs`: a unique sibling staging file is written, flushed, and synchronized before the formal file is replaced with the platform transaction (`ReplaceFileW` on Windows and rename-overwrite on Unix). Write, sync, and replace fault-injection tests prove that the prior formal file remains readable and unchanged. Asset code does not own a forwarding module or a second ad-hoc atomic-write implementation.

Metadata discovery rejects symlinks and Windows reparse points (including junctions), verifies every canonical path remains under its configured asset root, and tracks visited canonical directories to reject traversal cycles. Registry rebuild therefore never follows an asset-root escape.

## Watch Incremental Flow

`ProjectAssetManager` sends folded `AssetChange` events to `ProjectManager::scan_and_import_watch_changes`. Import still owns source/artifact refresh, while a cloned registry candidate removes changed or previous source-path entries, reloads their sidecars, and refreshes dependency edges from metadata. Removed and renamed sources delete every labeled subasset sharing that source path. Duplicate normalization additionally returns every sidecar whose identity it reminted, so those paths are replaced even when the copied asset sorts before the previously registered asset. The candidate is persisted first and swapped into live state only after persistence succeeds.

`ProjectManager` also builds its `ResourceRegistry` as a candidate. Neither the resource registry nor the asset registry becomes visible when registry persistence fails, so the two lookup authorities cannot diverge after a failed watch transaction.

Refreshing edges scans sidecar metadata rather than asset payloads. This makes incremental state byte-for-byte comparable to a later full metadata rebuild while keeping registry entry mutation scoped to watcher deltas.

## Dependency Extraction

The first wave is deliberately handwritten under `dependency_extractors/`: scene extraction walks `SceneAsset::direct_references`, material extraction includes shader, parent, and texture slots, and model extraction includes direct mesh references. `finish_successful_import` appends these references to importer dependencies before v7 sidecars are saved. This does not wait for runtime plan 13's future generic reflection extraction and does not add type-name reflection branches.

Other importer-owned dependency declarations remain valid. The registry consumes the single sidecar dependency representation after import and resolves it to UUID edges.

## Failure Boundaries

Filesystem and persistence failures retain their paths and sources through `AssetRegistryError`, which enters the existing asset error chain as `AssetImportError::RegistryIndex`. A duplicate UUID or path passed directly to `from_entries` is a typed hard error. Duplicate sidecar GUIDs discovered during project scanning are recoverable only through mandatory reminting and diagnostic emission; no later entry overwrites an earlier one.

Scene-project loading accepts locator-derived IDs only for explicit `builtin://` references. A non-builtin reference that resolves neither by UUID nor canonical path returns `SceneProjectError::DanglingAssetReference`; scene loading does not synthesize `ResourceId::from_locator` as a compatibility fallback.

## Test Coverage

The implementation slice includes source tests for all six signatures, corrupt persistence recovery, atomic write/sync/replace rollback, multiple manifest roots, traversal-link rejection, strict root/subasset tags, duplicate GUID sidecar reminting (including a copied asset that sorts before the original), watcher incremental/full-rebuild equivalence, complete subasset and reverse-edge removal, rename identity preservation, candidate rollback in both registry and `ProjectManager`, authoritative sidecar tag filtering after both rebuild and corruption recovery, typed dangling scene references, and scene/material/model dependency extraction. Existing package and scene-project tests were hard-cut to the registry-owned identity surface. The milestone test stage remains responsible for the full managed Runtime test gate; this implementation slice is closed with scoped formatting and static checks when the shared managed target is unavailable.

## Plan Sources

This module implements Plan 10 M2.2. Its folder-backed shape follows the engine structure convention and June review findings: `mod.rs` is wiring-only, declarations are separated from behavior families, and no production file approaches the repository's 1000-line emergency ceiling.
