---
title: Plugin Tilemap 2D Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/tilemap_2d
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/99e-runtime-sprite2d-canvas2d-sprite-atlas-tileset-tilemap-batching-sorting-lighting-physics-streaming-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperTileMap.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperTileMapComponent.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperTileLayer.cpp
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/data.rs
---

# Plugin Tilemap 2D Current Source Performance Review

## 1. Coverage and execution truth

The package review covers **11/11 Rust files**, **1,182 physical / 1,079 non-empty lines**, **42,585 bytes**, **15 test markers** and **1 ignored performance test**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `8e85a5266e603e3914137314ff8fd7fa791e5b20d169c469d1bca66d50028af6`.

| Module/folder | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| `runtime` | 4 | 262 | Registers a component descriptor and diagnostic-only Tiled importer; no typed component, chunk compiler, renderer or derived-system backend. |
| `editor` | 6 | 822 | Registers metadata and contains standalone dense-map validation/paint helpers; no document/controller/operation handler. |
| `dist` | 1 | 98 | Publishes registration metadata; no commands, state or native Tilemap bridge. |

Five README/Editor files already contain shared uncommitted changes and were preserved. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **8/11** files; `editor/src/lib.rs`, `editor/src/tests.rs` and `runtime/src/plugin.rs` retain formatting differences. Rust tests and dynamic tools were not run because the managed Windows validator is unavailable and no launchable current-source Tilemap product exists.

## 2. Structural performance findings

### P0: product, resource and operation closure is absent

Neither linked first-party Runtime nor Editor catalog selects Tilemap 2D. The Editor plugin references `plugins://tilemap_2d/editor/authoring.zui` and `plugins://tilemap_2d/editor/tilemap_component.zui`; **0/2** resources exist. Import Tiled, create Tilemap, create Tileset, open and paint operation IDs have no production handler outside package descriptors/tests. The open failure handoff for a factory-backed paint scene mode remains unresolved.

The package correctly reports Partial capability, but Beta/native packaging still cannot establish a usable product. Readiness must require a selected executable provider, physical resources, document/controller ownership and typed operation receipts.

### P0: generic JSON importer ownership prevents clean registration

Core registers callable `zircon.builtin.data.json` for `.json` at priority 0. Tilemap registers diagnostic-only `tilemap_2d.tiled` for `.tmx`, `.tsx` and `.json`, also at priority 0. The importer registry explicitly rejects same-priority duplicate matchers. Selecting Tilemap alongside core therefore makes JSON registration conflict instead of inspecting a Tiled signature or using explicit import context.

TMX/TSX also fail at execution because the backend is not installed. The target is one contextual Tiled decoder/build provider whose signature/version/settings/dependencies determine a Tilemap artifact; generic JSON must remain owned by Data. A diagnostic importer must not reserve a broad production extension.

### P0: dense source DTO is also cache/runtime payload and typed loads deep-clone it

Each `TileMapLayerAsset` stores a full `Vec<Option<u32>>` of `width * height` cells. `TileMapAsset` is reused directly by imported assets and bincode cache payloads, and generic typed loading returns `asset.as_ref().clone()`. Every load can therefore deep-clone every layer name and dense cell vector before any future runtime preparation.

Logical scale alone shows the problem:

| Map/layers | Logical cells | occupied `u32` payload lower bound | 8-byte `Option<u32>` layout estimate |
|---|---:|---:|---:|
| 256 x 256 x 4 | 262,144 | 1 MiB | 2 MiB |
| 1024 x 1024 x 8 | 8,388,608 | 32 MiB | 64 MiB |
| 4096 x 4096 x 16 | 268,435,456 | 1 GiB | 2 GiB |

The estimate excludes vector capacity, TOML/JSON source, parse DOM, duplicate cache/load copies, tile metadata, GPU data, collision/navigation and undo. Dynamic acceptance must bind the actual ABI layout, but no plausible layout makes full-map cloning or scanning an acceptable hot path.

Hard-cut the DTO into a versioned editable source/document and compiled sparse/chunk artifacts. Runtime instances hold shared generation-qualified chunk pages, not cloned authoring layers.

### P0: validation has no semantic or byte budget

`validate_layers` computes `width as usize * height as usize` and checks only vector length. It does not reject zero dimensions, enforce checked multiplication across targets, cap cells/layers/bytes, validate finite/ranged opacity, require stable unique layer IDs, resolve tile IDs against a Tileset, or validate Tileset image dimensions, tile sizes and collider semantics. Tileset itself has no validator and represents collider data as an optional string.

Import admission must validate header/signature, source bytes, decoded map/layer/object counts, external dependencies, infinite-map chunks, decompression expansion and target cook budgets before allocation. Tile identity must include tileset source, coordinate/alternative and transforms, rather than a bare `u32` whose meaning changes when a sheet is repacked.

### P0: Scene persistence and Runtime execution are disconnected

`SceneTileMapAsset` can serialize a reference, but world load does not install a typed Tilemap component and world save fixes `tilemap: None`, silently dropping it. Production graphics contains only `BuiltinRenderFeature::Tilemap` as a descriptor-only advanced slot; no Tilemap asset/component consumer, render phase/pass, chunk bounds, culling, batching, collision, navigation, occlusion or streaming system exists.

This makes RenderDoc inapplicable today: there is no Tilemap draw to capture. The first runtime algorithm must scale with camera-intersecting resident chunks, visible occupied cells and dirty chunks, not total map area every frame.

### P0: the paint helper is bounded locally but still scans the entire map per stroke

The shared in-progress helper contains useful behavior that should survive migration:

- layer names are validated and resolved before mutation;
- a stroke is capped at 4,096 unique `(layer,x,y)` cells;
- all addresses are preflighted, so invalid strokes leave the asset unchanged;
- mutations update occupied/empty counts incrementally after the initial count.

However, every call first runs global validation, rebuilds a `BTreeMap` for all layers, builds a `BTreeSet` for requests, and calls `tilemap_editor_stats`, which scans every cell in every layer. Complexity is `O(total_cells + layers log layers + k log k)` per pointer stroke. A 1024 x 1024 map with eight layers scans about **8.39 million cells** even when one cell changes. The helper is test-only and its receipt is not a document transaction, undo delta, dirty-chunk frontier or save/reload result.

The final editor path must maintain document-owned counts and stable layer indexes at mutation time, coalesce pointer samples, write a bounded reversible cell delta, and dirty only affected render/physics/navigation/occlusion chunks. Global validation belongs at import/migration/explicit audit boundaries, not every pointer event.

### P1: the ignored benchmark proves only removal of an intentionally pathological baseline

The ignored test compares one full-map scan against 128 full-map scans on a 64 x 64 x 4 fixture and requires 80% improvement. Its measured interval excludes the full asset clone performed before timing and exercises no controller, transaction, chunk build, GPU upload, render, collision or save. It correctly demonstrates that per-cell global scans are bad, but it does not establish that one global scan per stroke is scalable.

Replace release evidence with cell-to-transaction latency, affected chunks, CPU/GPU upload bytes, undo bytes and stationary-frame work over sparse and dense representative maps.

## 3. Reference-engine evidence and adopted boundaries

Unreal Paper2D is the primary responsibility reference:

- `PaperTileMap.cpp:111-116` keeps every layer dimension synchronized with the map; lines 279-295 build collision from layer-owned data.
- `PaperTileMapComponent.cpp:58-66` creates a render scene proxy under an explicit rebuild cycle counter. Lines 282-296 count occupied cells first and reserve the resulting vertex capacity.
- `PaperTileMapComponent.cpp:305-336` separates visible layers and validates dimensions; lines 395-402 form render sections by source texture/material rather than one draw per cell.
- `PaperTileLayer.cpp:178-182` owns cell mutation; lines 200-249 build collision from tile metadata; lines 332 onward count occupied cells.

These sources establish separate asset/layer/component/render-proxy/collision responsibilities and observable rebuild cost. They also expose a limitation Zircon should not copy: `RebuildRenderData` still walks the complete Paper2D grid and emits six vertices per occupied tile. That is reasonable as a compatibility reference, not the target for large/infinite maps.

Local secondary references supply the scale boundary:

- Bevy `tilemap_chunk/mod.rs:50-67` makes each rectangular chunk one render component, lines 168-183 reuse a mesh by chunk size and encode tile data in an image, and lines 195-239 update it only when tile data changes.
- Fyrox `tilemap/data.rs:31-55` uses 16 x 16 chunks, lines 145-172 iterate chunk storage, lines 245-260 provide bounded iteration, and mutation paths can address one chunk.

Zircon's target combines Unreal's ownership/lifecycle separation with chunk-bounded storage, change-driven publication and shared geometry. It must not inherit Paper2D's whole-map proxy rebuild or create a second Tiled-specific world.

## 4. Required optimization sequence

| Milestone | Owner result | Acceptance gate |
|---|---|---|
| M0 Product/import closure | Select one Runtime/Editor provider; resolve resources/handlers; remove broad diagnostic JSON matcher; retain explicit Partial fail-close until executable. | Cold bootstrap has no matcher conflict; generic JSON selects Data; signature-qualified Tiled import returns a typed receipt. |
| M1 Source/schema hard cut | Versioned Tileset/Tilemap source, stable tile/layer identities, semantic validation and dependency graph. | Zero/overflow/oversized maps, bad opacity/tile IDs/images/colliders and corrupt dependencies fail before large allocation. |
| M2 Chunk artifact | Compile dense/finite and sparse/infinite input into deterministic chunk pages, bounds, material groups and target-qualified keys. | Two instances share immutable pages; load does not deep-clone all cells; empty chunks consume no full-grid storage. |
| M3 Scene/runtime | Typed Scene/ECS component, lossless load/save, world-owned chunk service, camera-bounded traversal, culling and residency. | Save/reopen preserves identity; stationary frames perform zero rebuild/upload; traversal scales with intersecting chunks. |
| M4 Rendering/derived systems | Shared quad/chunk geometry or GPU tile pages, material batches and generation-qualified collision/navigation/occlusion adapters. | RenderDoc proves Tilemap pixels, batches and bounded uploads; derived receipts match or explicitly lag the edit generation. |
| M5 Editor transaction | Factory-backed paint mode, coalesced pointer stroke, reversible delta, cached stats/indexes and dirty-reason frontier. | One-cell stroke touches one bounded chunk; P50/P95/P99 latency and undo/derived bytes scale with changed cells/chunks, not total map. |
| M6 Dynamic acceptance | Representative finite/sparse/infinite corpus, WPR/ETW CPU/IO/power capture and RenderDoc GPU capture on a current-source executable. | No unexplained main-thread import/chunk cook; no per-frame full-map scans; frame/edit/import budgets pass on named hardware/profile. |

## 5. Instrumentation contract

Record source/build/profile hashes, map/layer/chunk counts, non-empty cells, resident/requested/visible/dirty chunks, dirty reasons, visited cells, render sections/batches/draws, CPU artifact/staging/undo bytes, GPU resident/upload/retire bytes, import/chunk job queue states, cache hit/miss causes and render/physics/navigation/occlusion generations.

WPR/ETW owns CPU scheduling, input-to-commit latency, IO and power evidence. RenderDoc owns pass/draw/resource/upload/pixel evidence after the Tilemap pass exists. Old binaries, Sprite-only pixels, descriptor counts and the ignored microbenchmark cannot satisfy current-source acceptance.

## 6. This review's implementation decision

No production source was changed. The relevant package helpers are in shared in-progress files, are not wired to the product, and optimize a dense DTO that canonical Runtime99e/Editor34 already require to replace. A local cached counter or lower stroke cap would create policy in the wrong owner and leave JSON conflict, scene data loss and runtime absence intact.

Static review is complete for `zircon_plugins/tilemap_2d`; dynamic acceptance remains pending and is not a milestone-completion claim.
