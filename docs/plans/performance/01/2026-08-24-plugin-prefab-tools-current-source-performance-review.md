---
title: Plugin Prefab Tools Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/prefab_tools
  - zircon_runtime/src/asset
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstanceSubsystem.cpp
  - dev/godot/scene/resources/packed_scene.cpp
---

# Plugin Prefab Tools Current Source Performance Review

## 1. Coverage and product state

The primary package review covers **11/11 Rust files**, **992 physical / 896 non-empty lines**, **37,610 bytes**, **14 test markers** and **1 ignored performance test**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `a08bc381417c2d0ccba8b1d62934c67f9b870ac6da7386082b6f69f083f2e202`.

| Module/folder | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| `runtime` | 4 | 266 | Registers one component descriptor and one diagnostic-only prefab importer; no instantiation or propagation system. |
| `editor` | 6 | 628 | Registers authoring descriptors and exposes standalone override helper functions; no operation handlers or documents. |
| `dist` | 1 | 98 | Publishes registration/lifecycle metadata; no callable prefab bridge. |

Four Editor/README files already contain shared uncommitted work and were preserved. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **9/11** files; shared `editor/src/tests.rs` and otherwise unmodified `runtime/src/plugin.rs` retain formatting differences.

The Runtime built-in prefab importer, `PrefabAsset`/`PrefabInstanceAsset` schemas, artifact conversion, typed loading, scene references, operation paths, first-party catalogs and all package resources were inspected. Rust tests, WPR/ETW and RenderDoc were not run because the managed validator is unavailable and no launchable current-source product exists.

## 2. Structural performance findings

### P0: PrefabTools is not selected by either first-party product catalog

`RuntimePluginId::PrefabTools` exists, but the linked first-party Runtime catalog has no dependency, feature or provider branch for this package. The first-party Editor catalog only links Navigation and Neural. PrefabTools therefore cannot contribute its Runtime or Editor registration through the normal selected product.

The Editor registration references `plugins://prefab_tools/editor/authoring.zui`, `plugins://prefab_tools/editor/prefab_instance.zui` and `plugins://prefab_tools/templates/default_prefab.toml`; the package contains **zero matching physical resources**. Five commands, five menu items, one toolkit, one creation template and one inspector customization are metadata only. Their operation paths have no implementation outside this package's descriptor/tests.

Capability is marked Partial, which is more accurate than Complete, but Beta maturity and native packaging still overstate a product that is unreachable and cannot open its declared surfaces. Readiness must require linked Runtime/Editor providers, resolved resources and executable handlers.

### P0: the plugin collides with the real built-in prefab importer

Runtime already registers `zircon.builtin.toml.prefab` as a callable `.prefab.toml` importer. PrefabTools registers `prefab_tools.prefab` for the identical full suffix as `DiagnosticOnlyAssetImporter`. Both descriptors retain the default priority 0. `AssetImporterRegistry` rejects duplicate matchers at equal priority, so coexistence can fail registration rather than choose the functional importer.

The fix is a hard ownership cut: keep exactly one executable prefab importer and attach optional authoring/runtime services to the same provider identity. A missing backend belongs in capability state, not as a colliding diagnostic importer. Dist similarly exposes `invoke_command: None`, no bridge methods and a message that services remain hosted by Runtime; native parity is absent.

### P0: no Runtime prefab instance exists beyond serialized metadata

`PrefabAsset` owns a complete `SceneAsset`; `PrefabInstanceAsset` only stores an asset reference, local transform and string/JSON overrides. `SceneEntityAsset` may embed that DTO, but production Runtime references only serialize, clone, count and report it. There is no source generation lease, archetype/template state, entity instantiation, stable source-to-instance mapping, nested prefab expansion, cycle/depth guard, shared component storage, visibility/streaming policy, override application, teardown or hot-reload propagation.

Artifact preparation deep-converts the entire prefab scene, and generic `load_prefab_asset` returns another owned clone. Without a shared immutable archetype/lease, any future naive instance implementation will repeatedly deserialize or clone complete scene trees. The architecture must precede a local helper optimization.

### P0: current apply, revert and break helpers cannot perform their named operations

`apply_prefab_overrides` receives only the instance, validates it, clones effective overrides into a report and clears the instance list. It cannot load or mutate the source `PrefabAsset`, verify source generation, write a transaction, update other instances or publish an artifact. `break_prefab_instance` returns only local transform plus override values; it never expands the source scene, so it cannot leave an ordinary equivalent scene graph behind. Create-from-selection and open have no implementation at all.

These helpers are useful DTO experiments, not an authoring service. Wiring them directly would risk destructive clears with no atomic source/document commit. Apply/revert/break must be transaction plans over a source-generation-qualified document/world snapshot, with rollback and stale-generation rejection.

### P0: override identity is unstable text and each query rebuilds the complete index

Overrides use `(entity_path: String, property_path: String, serde_json::Value)`. Entity renames/reparenting and component/property schema migrations can invalidate paths silently. No source entity GUID, component type/instance identity, property field ID, schema version or source generation participates in identity.

`validate_prefab_instance` builds a `BTreeSet` over all paths. `effective_prefab_overrides` independently builds a `BTreeMap`, then clones every surviving string/value into a new vector. Apply and break therefore perform two `O(n log n)` traversals plus `n` output clones. The ignored 8,192-override benchmark runs eight queries, so even its optimized side still constructs eight trees and clones **65,536 complete override objects per sample**; it only removes temporary key-string clones.

Normalize and validate one typed override delta at mutation time. Store it in a generation-bound indexed representation with deterministic iteration, and expose borrowed/leased effective views. Stable frames and inspector reads must not rebuild or clone all overrides.

### P0: nested-instance, reload and edit coordination are absent

Prefab direct-reference reporting delegates to the embedded scene, but no graph service computes transitive nested dependencies, rejects cycles, bounds depth/expanded entities, coalesces simultaneous loads or propagates source changes. There is no edit lease preventing two conflicting source sessions, no instance generation reconciliation, and no network/save identity contract.

Unreal's `LevelInstanceSubsystem` maintains registered/loading/loaded identities, coalesces requested load/update/unload work in maps/sets, early-outs when no work exists, suppresses redundant reloads, avoids streaming changes during undo, validates ancestor/edit state and commits/discards explicit edit sessions. This is the primary behavioral constraint.

Godot's `PackedScene` is a useful secondary data-layout reference: it instantiates dense indexed node records, interned name/variant tables and nested scene references, with a diagnosed slow recovery path when stable node identity is lost. Zircon needs its own typed ECS-oriented representation, but path-string scans and whole-scene clones cannot remain the instance boundary.

### P1: tests qualify descriptor/helper fragments rather than the product

Tests cover registration metadata, override ordering/validation and the borrowed BTree key microbenchmark. They do not instantiate a prefab, resolve nested dependencies, apply typed deltas, preserve stable IDs, commit/rollback a document transaction, break to an equivalent scene, propagate reload, coalesce duplicate loads or measure memory/main-thread time.

Required scales include 1/1,000/100,000 instances, 10/1,000/100,000 source entities, nested depth/cycle fixtures, 0/10/10,000 overrides, cold/warm load, repeated spawn/despawn, source reimport and simultaneous apply/revert/break. Receipts must bind source/instance generations and report shared versus unique bytes.

## 3. Dependency-ordered optimization plan

### M0: establish one provider and truthful product closure

Remove the colliding diagnostic importer and keep one callable `.prefab.toml` owner. Link PrefabTools Runtime/Editor or absorb its contributions into the canonical Prefab owner. Resolve all physical resources and operation handlers before readiness. Source/library/native forms share one contribution bundle or explicitly report unsupported behavior.

### M1: define archetype, instance and typed override identities

Give every source entity/component/property a stable versioned identity. Define immutable prefab generation, nested dependency graph, instance ID, source-to-instance entity map and typed override delta. Keys include source/dependency hashes, schema and recipe versions. Reject cycles, excessive depth/entity expansion and stale targets before publication.

### M2: implement bounded Runtime instantiation

Load one immutable archetype lease per prefab generation. Allocate/initialize ECS entities in admitted batches, share immutable defaults/resources, apply compact deltas once and retain only instance-unique state. Coalesce load/update/unload requests, support cancellation and teardown, and keep main-thread structural publication within a declared entity/time budget.

### M3: implement transaction-safe authoring

Create-from-selection, open, apply, revert and break become real operations over Editor document/world authorities. Capture source/instance generation and selection snapshot; validate before mutation; emit one undoable transaction; atomically save/publish or roll back. Break materializes equivalent ordinary scene entities and removes the link only after success.

### M4: add incremental propagation and reconciliation

On source change, diff old/new archetype generations by stable IDs, update only affected non-overridden fields, preserve valid instance state, diagnose orphaned overrides and retain last-good instances on failure. Nested changes use dependency-order scheduling and one coalesced terminal result per generation.

### M5: converge runtime, editor, network and save behavior

Define which fields are shared defaults, per-instance mutable state, replicated state and saved deltas. Mount/unmount revokes new work, cancels queued jobs and waits for instance leases. Editor preview uses the same instantiation/reconciliation path as Runtime rather than cloning a second prefab system.

### M6: instrument and dynamically qualify

Record source/expanded entity counts, nested depth, override count, shared/unique bytes, load/instantiate/apply/propagate/despawn p50/p95/p99, queue/main/worker CPU, allocations/peak RSS, duplicate coalescing, stale/cancel results and energy. WPR/ETW owns CPU, scheduling, memory and power. RenderDoc only validates render extraction/instancing, draw/resource counts, pixels and VRAM after a current-source executable exists.

## 4. Acceptance gates

| Gate | Required evidence |
|---|---|
| A1 | Exactly one `.prefab.toml` importer executes; Runtime/Editor catalogs link the selected provider and every declared resource/operation resolves. |
| A2 | Source entities/components/properties and instances use stable, generation-qualified typed identities; path-string mutation authority is removed. |
| A3 | One prefab generation is loaded once and shared across instances; stable instance memory is proportional to unique deltas/state, not complete scene size. |
| A4 | Nested cycles/depth/entity expansion are bounded and diagnosed before partial publication. |
| A5 | Stable inspector/frame queries rebuild zero complete override indexes and clone zero complete override lists. |
| A6 | Create/open/apply/revert/break execute one undoable transaction with rollback and stale-generation rejection; break preserves equivalent scene state. |
| A7 | Source reload diffs by stable identity, updates only affected fields, preserves overrides and coalesces one propagation per generation. |
| A8 | Runtime load/update/unload queues suppress duplicates, honor cancellation and publish bounded ECS batches; no full prefab clone occurs on each instance. |
| A9 | Scale receipts report cold/warm p50/p95/p99, entities/s, shared/unique bytes, peak RSS, queue/main/worker CPU, cancellations and energy. |
| A10 | WPR/RenderDoc evidence comes from the reviewed current-source executable; helper microbenchmarks cannot qualify the prefab product. |

## 5. Validation record

- Static package coverage: complete, 11/11 Rust files; built-in importer, asset/artifact/load, scene use and catalogs reviewed to terminal ownership.
- Static product gates: Runtime catalog unlinked; Editor catalog unlinked; physical resources 0/3; operation handlers absent; duplicate priority-0 prefab matcher identified.
- Formatting: 9/11 pass; shared Editor test and Runtime plugin formatting debt preserved.
- Source changes: none. A local BTree optimization or direct helper wiring would entrench a non-executable, non-transactional boundary.
- Rust tests and dynamic tools: pending for the managed current-source executable; no raw Cargo, WPR or RenderDoc substitute was used.
- Protected ledgers, milestone commit and quantified WeCom completion notice remain pending until dynamic acceptance.
