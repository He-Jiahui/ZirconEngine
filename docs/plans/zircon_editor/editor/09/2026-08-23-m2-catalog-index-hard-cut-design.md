---
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
milestone: M2
slice: catalog-index-import-ingress-hard-cut
status: source_updated_static_green_cargo_pending
related_code:
  - zircon_editor/src/core/asset/index.rs
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_runtime/src/asset/project/manager/registry_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/AssetRegistry/AssetData.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetData.cpp
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Public/IAssetTools.h
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetTools.cpp
---

# Editor09 M2 Catalog/Index Hard-Cut Design

## Current-source diagnosis

`ProjectManager::asset_registry()` is the runtime authority for project asset metadata.
`EditorAssetIndex` is designed to project that authority but has no production construction site.
Meanwhile `DefaultEditorAssetManager` independently retains catalog records, UUID/locator maps
and source generation. Catalog details now query Runtime's reverse-reference registry directly,
but retained-host model/material actions still call Runtime `AssetManager::import_asset` directly.
This produces two editor-side asset state models and leaves the planned job/import/logging path
unreachable.

Unreal's `FAssetData` is registry metadata queried by downstream consumers; it is not a second
browser-owned import database. Zircon follows that separation: Runtime owns registry/import facts;
the Editor manager owns the sole mutable projection and immutable UI snapshot publication.

## Target ownership

1. `DefaultEditorAssetManager` owns one `EditorAssetIndex` projection, initialized from the active
   `ProjectManager::asset_registry()` and its `.zmeta` documents. It is the only index supplied to
   `EditorAssetImportFlow`.
2. `EditorAssetCatalogGeneration` becomes an immutable UI snapshot derived from the index and
   preview data. It is not a second mutable UUID/path/reference authority.
3. The same manager constructs `EditorAssetImportFlow` from its versioned Runtime
   `ProjectAssetManager` handle and the editor job system. It emits completed import facts through
   the canonical `EditorLogService` cursor projector.
4. `EditorAssetManager` exposes typed import submission. Retained-host model/material ingress
   calls that service only; it no longer calls Runtime `AssetManager::import_asset` or runs a
   synchronous workspace refresh loop.
5. Project close atomically retires index, import flow generation and preview generation before it
   publishes the empty catalog. Old catalog/index/import results cannot be read or commit after
   the new generation is visible.

## Compound Import Transaction Prerequisite

The former retained-host model action copied an external model, wrote derived skeleton/clip
sources, called `AssetManager::import_asset` for each source, imported the default material, then
instantiated the mesh. The editor-side skeleton/clip writer and derived import loop have now been
hard-cut: Runtime glTF ingestion alone publishes those subassets in its `AssetImportOutcome`.
External source staging, the model-root import, default-material import and mesh instantiation are
still separate operations with no shared receipt. A later failure can therefore still leave an
unbounded prefix of source files, artifacts, registry generations and resources published.

Runtime already has the essential single-source shape: `ProjectManager::prepare_targeted_generation`
mutates only a candidate manager, produces artifact/meta/registry writes, and
`ProjectAssetManager::commit_targeted_project_resource_sync` delays resource publication until
the durable file commit succeeds. It is not yet a compound transaction. Each preparation emits a
registry persistence write for its intermediate candidate and each service-contract import builds
its own resource batch and project generation publication.

More importantly, targeted preparation invokes `stage_environment_ibl_import` before returning
the prepared generation. That staging can materialize cache data before the durable transaction.
Combining existing prepared generations, or adding a public `import_assets` loop, would therefore
claim all-or-nothing publication while still leaking products on preparation or commit failure.
It is prohibited.

Unreal's `IAssetTools::ImportAssetTasks` provides an explicit task collection, per-task result and
optional save path. Zircon adopts the task/receipt boundary but requires a stronger Runtime
transaction for one model's source plus deterministic derived products: an Editor request must not
publish a prefix of that logical model import.

### Required Runtime Order

1. Move external source copy and OBJ sidecar discovery out of retained-host helpers into a Runtime
   `ImportSourcePlan`. Runtime glTF ingestion already owns skeleton/clip subasset derivation;
   planning reads/parses and produces deterministic `PreparedFileWrite` values only, without
   writing a destination file.
2. Split environment IBL work into an immutable prepared output and a transaction-owned durable
   write. No cache or registry mutation may occur before the complete source/product plan is
   accepted.
3. Add `PreparedProjectImportBatch`, which prepares every source against one cloned
   `ProjectManager`, rejects duplicate source URIs, collects every artifact/meta write, retains
   only the final candidate's registry persistence write, and locks all meta paths in the existing
   deterministic stripe order.
4. Derive one `ResourceMutationBatch` from the final candidate, then perform exactly one durable
   project transaction, candidate install, resource publication and project-generation event.
   Resource records must remain invisible if durable commit or generation validation fails.
5. Return a typed batch receipt containing the source URI, committed generation sequence and every
   committed root/derived record. Retained Host may instantiate a mesh only after observing the
   matching receipt; undo of world instantiation remains a separate editor command.

### Non-Negotiable Contracts

- One logical model import performs one Runtime durable file commit and one registry persistence
  write, not `animation_count + 3` independent import commits.
- Parse, duplicate URI, source authority, source-copy, derivation, artifact preparation and
  generation-CAS failure publish no registry/resource/catalog delta and leave no final destination
  file. Recovery owns interrupted journal cleanup.
- Project close or a newer project generation rejects the entire receipt before resource publication.
- The Editor import job is an observer of this Runtime transaction. It neither writes model source
  files nor composes a second registry/resource batch.

## Hard-cut sequence

1. Add manager-owned index/flow lifecycle state and a generation-bound completion cursor.
   Seed/update it from Runtime catalog input during `sync_from_project`; delete any state map that
   duplicates an index query once all projection consumers move.
2. Build catalog snapshots from the single index. Migrate details, delete preflight, relocation,
   preview admission and reference consumers in the same slice; remove old UUID/locator/reference
   fields rather than keeping a fallback read path.
3. Add the typed import operation to `EditorAssetManager`; module construction resolves the
   existing `ProjectAssetManager` as `AssetManager`, job system and log service. Import completion
   is projected once by cursor, never by retained UI polling or a second worker receiver.
4. Migrate retained-host model/material actions to submit typed requests and continue their scene
   action only from the matching completed result. Remove direct Runtime import calls and the
   post-import synchronous refresh route.
5. On deactivation, invalidate the import/index generation before the empty catalog event;
   late job completion and preview completion must fail closed.

## Required contracts

- Project open, watcher refresh and import result converge to one index generation.
- A repeated observation of one import completion creates one `LogSource::import()` record with
  its asset jump, while resync does not duplicate it.
- No production `EditorAssetImportFlow` or `EditorAssetIndex` can be unreachable after the cut.
- Source searches have zero retained-host direct `AssetManager::import_asset` calls and zero
  duplicate catalog UUID/locator/reference mutable owners.
- No compatibility constructor, empty fallback flow, UI-owned completion store or parallel
  Runtime importer is retained.

## 产出记录与时间

| 日期 | 状态 | 产出与证据 |
| --- | --- | --- |
| 2026-08-24 | source_updated_static_green_cargo_pending | Runtime commits the project generation before broadcasting watcher changes; retained-host refresh forwards that batch through `EditorAssetManager::project_runtime_asset_changes`, which only updates the manager-owned `EditorAssetIndex` transient state and emits batched immutable `AssetStateChanged` rows. Candidate catalog construction inherits valid dirty/importing state before row projection, and final replacement moves state observed during the build so watcher/refresh races cannot erase it. The model ingress completion consumes one `ProjectImportReceipt`, logs the committed import, refreshes the Runtime-authoritative catalog, and instantiates only the resolved ready resource; it does not revive direct host imports. Static contracts covering catalog/index/content projection, receipt routing, and import flow passed 48/48; scoped source parsing/formatting and `git diff --check` passed for the new watcher owner. Cargo, runtime latency/allocation/RSS/power traces, accepted milestone review, commit, and WeCom notification remain pending and are not implied. |
| 2026-08-24 | runtime_ibl_project_transaction_staging_and_catalog_index_cutover_complete_m2_import_receipt_pending | Re-read Runtime targeted preparation, durable commit, resource publication and retained-host model ingress; checked Unreal `IAssetTools::ImportAssetTasks` task/result boundary. Recorded the IBL pre-commit side-effect blocker and the required compound-transaction sequence above. `ProjectManager` now retains `Arc<AssetRegistryIndex>` generations and exposes the exact shared Runtime snapshot to `EditorAssetIndex::from_runtime_project`; the factory validates catalog-input metadata atomically without copying registry rows. `DefaultEditorAssetManager` now owns one optional `EditorAssetIndex`, derives catalog records/details from that index's Runtime catalog-input and registry generations, and holds no persistent mutable UUID/locator/source-generation maps. Preview refresh reads the immutable generation's private catalog row and COW-publishes exactly that row; project deactivation clears the index before publishing the empty catalog. The sync failure boundary is `EditorAssetSyncError`, preserving Runtime and index failures as typed causes. The retained-host `.zranim` skeleton/clip source writer and its per-derived-source import loop are deleted; `import_gltf` owns animation/skin subassets in the model-root Runtime import outcome. The Editor `ReferenceGraph` module, mutable state and orphan mutable-record lookup are deleted; catalog details query `AssetRegistryIndex::get_referencers_by_uuid` directly. Runtime now returns prepared IBL file writes from equirectangular and external-cubemap ingestion, appends them to both targeted and full project durable transactions, rejects conflicting cache bytes, and accepts only validated IBL cache targets during project recovery. `python -m unittest` static contracts: 35 passed (catalog/index, import-flow, and IBL transaction routing); targeted `rustfmt --check` and `git diff --check` pass. Quantified structural result: 0 persistent manager UUID/locator catalog maps, 1 shared Runtime registry `Arc` per editor index, and 1 immutable published catalog generation; the existing O(N) snapshot/detail construction is intentionally retained pending runtime profiler sampling. No Cargo validation, runtime latency/power trace, accepted milestone, commit, or WeCom push is implied. External source staging, default material and model root are still not one batch receipt. |
