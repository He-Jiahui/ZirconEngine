---
title: Plugin Terrain Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/terrain
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_runtime/src/graphics
  - zircon_runtime/src/scene
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeRender.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeEdit.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Public/LandscapeEdit.h
---

# Plugin Terrain Current Source Performance Review

## 1. Coverage and execution truth

The package review covers **11/11 Rust files**, **958 physical / 872 non-empty lines**, **34,875 bytes**, **10 test markers** and **1 ignored performance test**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `023b1a533a79a6490d94f62c3cee3ceebe97a013fbc9cb58fdcc41584cd26641`.

| Module/folder | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| `runtime` | 4 | 263 | Registers a component descriptor and a diagnostic-only RAW/R16/PNG importer; no terrain service, renderer, collision or query backend. |
| `editor` | 6 | 597 | Registers metadata and validates dimensions/extensions; no byte decoder, document transaction, brush, dirty-region job or operation handler. |
| `dist` | 1 | 98 | Publishes registration/lifecycle metadata; no commands or native terrain bridge. |

Five package/Editor files already contain shared uncommitted changes and were preserved. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **8/11** files; `editor/src/lib.rs`, `editor/src/tests.rs` and `runtime/src/plugin.rs` retain formatting differences. Rust tests, WPR/ETW and RenderDoc were not run: the managed Windows validator is unavailable and the current source does not produce a launchable Terrain product. RenderDoc cannot validate a zero-pass descriptor slot.

## 2. Structural performance findings

### P0: the selected product has no Terrain closure

Neither linked first-party Runtime nor Editor catalog selects Terrain. The Editor registration references `plugins://terrain/editor/authoring.zui`, `plugins://terrain/editor/terrain_component.zui` and `plugins://terrain/templates/default_heightfield.toml`; **0/3** resources exist. Import, weightmap, create, open and sculpt commands have no handlers outside descriptor/tests. The package therefore cannot execute through the normal product path even though it is marked Beta/Partial and can publish manifests.

Readiness must require one selected Runtime provider, one selected Editor provider where applicable, resolved resources, callable operations and typed receipts. Capability names or a loaded native registration table are not execution evidence.

### P0: source document, cooked artifact and resident runtime data are one object

`TerrainAsset` stores `width`, `height` and an inline `Vec<Real>` containing every height sample, plus authoring layers. The same type is used as imported asset and bincode cache payload. Generic `load_typed` then returns `asset.as_ref().clone()`, deep-cloning all height samples and layer strings/references for every typed load.

This collapses mutually different lifetimes:

1. editable source and import settings;
2. validated canonical height/layer data;
3. target-specific tiled/mipped build artifacts;
4. shared immutable CPU residency;
5. GPU height/normal/weight/hole pages;
6. render, collision and navigation generations.

No local `Vec` tweak can repair this ownership model. The hard target is `TerrainSourceAsset -> TerrainBuildArtifact -> TerrainRuntimeInstance`, with stable tile IDs, generations, shared leases and independently invalidated render/physics/navigation products.

### P0: admission checks count elements but do not budget bytes or work

The Editor plan computes `width * height` through `u64` and only checks conversion to `usize`. On 64-bit Windows that accepts the full `u32 x u32` domain. Core `TerrainAsset::validate_dimensions` uses `width as usize * height as usize`, exempts an empty sample vector, and has no dimension, byte, finite-value, dependency or build-expansion budget. RAW lacks byte order, signedness and stride; R16 lacks encoding semantics; PNG dimensions/channels are trusted from the request instead of the header.

Lower bounds before temporary decode buffers, serde/TOML text, copies, normals, weight maps, collision, GPU staging or residency duplication are:

| Heightfield | Samples | R16 base / full mip chain | current `Real=f32` base / full mip chain |
|---|---:|---:|---:|
| 4096 x 4096 | 16,777,216 | 32.00 / 42.67 MiB | 64.00 / 85.33 MiB |
| 16384 x 16384 | 268,435,456 | 512.00 / 682.67 MiB | 1024.00 / 1365.33 MiB |
| 65536 x 65536 | 4,294,967,296 | 8192.00 / 10922.67 MiB | 16384.00 / 21845.33 MiB |

Admission must be target-aware and checked before allocation: source bytes, decoded bytes, canonical tiles, mip/derived expansion, peak CPU staging, GPU bytes, concurrent jobs and cancellation. A dimension-only `usize` test is not an import budget.

### P0: there is no runtime Terrain consumer or scalable algorithm

`SceneTerrainAsset` is only an asset reference. Production searches find serialization/import/cache/load touchpoints but no Terrain consumer in graphics or scene execution. The built-in Terrain render feature remains a descriptor-only advanced slot with no phase/pass. There is no patch topology, component/tile partition, quadtree/clipmap/CDLOD selection, neighbor crack policy, frustum/HZB culling, streaming residency, GPU upload, material layer packing, height query, collision or navigation adapter.

Consequently there is no current frame algorithm to optimize or profile. A naive future full-grid mesh would be the wrong baseline: work must scale with visible/resident patches and dirty regions, not total world samples per frame.

### P0: import ownership is ambiguous and one matcher is shadowed

Runtime core owns a callable typed `.terrain.toml` importer. Terrain registers a different diagnostic-only heightfield importer for RAW/R16/PNG. Normal PNG is already owned by the built-in image/texture importer at priority 10, while Terrain retains default priority 0, so generic PNG selection resolves Texture rather than Terrain. Editor publishes separate heightfield and layer-stack descriptions over the same extensions, but its heightfield planner correctly rejects layer stacks because channel/format semantics are absent.

Keep that fail-closed rejection. Replace extension competition with an explicit Terrain import context and one canonical build provider. Decoder identity, recipe, source dependencies and output target must participate in the artifact key; ordinary PNG texture import must remain deterministic.

### P0: authoring commands do not implement region-based editing

`sculpt`, create, import, open and weightmap operations are metadata only. There is no document owner, brush sample coalescing, bounded dirty rectangle, reversible delta, tile lock, undo memory budget, asynchronous normal/mip/collision/navigation rebuild, stale-generation rejection or preview publication. Wiring a whole-asset mutation now would serialize/clone/rebuild the complete heightfield and block the UI thread.

Terrain editing must commit bounded tile deltas into a transaction, publish an immediate preview generation, coalesce dirty regions, and schedule derived work under explicit CPU/GPU budgets. Collision/navigation may lag visually but must expose their generation and never silently claim currentness.

### P1: the ignored benchmark measures the wrong work

The ignored benchmark submits 16,384 planning requests around extension normalization and requires fewer string allocations. It reads no source bytes, allocates no terrain, creates no tiles/mips, performs no brush edit, schedules no job and renders no pixels. Even a stable improvement would not establish production throughput or latency. Keep microbenchmarks only after end-to-end counters identify this helper as material.

## 3. Unreal source evidence and adopted boundaries

Unreal's local Landscape source supports the architectural direction without requiring Zircon to copy its object model:

- `LandscapeEdit.cpp:140-157` initializes section base, component quad size, subsection count and subsection size, enforcing `NumSubsections * SubsectionSizeQuads == ComponentSizeQuads`.
- `LandscapeRender.cpp:1047-1077` computes and caches section LOD values per view; lines 1237-1272 publish per-view LOD data. Lines 182-186 explicitly allow asynchronous per-component LOD bias computation.
- `LandscapeRender.cpp:1392-1403` binds component position, maximum LOD, subsections and subsection vertex dimensions; lines 1714-1811 reuse shared buffers and register Landscape culling.
- `LandscapeEdit.h:50-76` tracks update regions per mip; lines 145-146 express bounded `SetHeightData(X1,Y1,X2,Y2,...)` with optional bounds, collision and mip updates; lines 201 and 230-233 preserve dirty layer/data regions.
- `LandscapeEdit.cpp:2493` and `2746` enqueue bounded mip update regions rather than treating every edit as an unconditional full-texture rebuild.

The transferable rules are: partition first; select LOD per view; reuse immutable patch topology; update bounded regions; separate render/collision/navigation derived generations; and expose asynchronous work/budgets. Zircon should not emulate Unreal class count, but it must preserve these complexity boundaries.

## 4. Required optimization sequence

| Milestone | Owner result | Acceptance gate |
|---|---|---|
| M0 Product closure | Select exactly one Terrain Runtime/Editor provider; add real resources/handlers or remove unreachable declarations; eliminate diagnostic importer competition. | Cold bootstrap proves selected provider, every declared resource resolves, every command returns a typed receipt, and ordinary PNG still selects Texture. |
| M1 Source/artifact hard cut | Version `TerrainSourceAsset`, import recipe, canonical tile schema and target-specific build artifact; stop caching/loading the inline authoring object as runtime payload. | Loading two instances shares immutable artifact pages; typed load does not deep-clone height samples; content/build/profile hashes reproduce identical artifacts. |
| M2 Budgeted import/build | Header-first validation, checked byte arithmetic, bounded decode/cook queues, cancellation and spill policy outside C:. | Oversized/corrupt RAW/R16/PNG fails before large allocation; measured peak bytes stay within declared envelope; editor remains responsive under concurrent imports. |
| M3 Runtime tiles/LOD | World-owned generation service, stable component/tile IDs, patch topology, per-view screen-error LOD, neighbor continuity, culling and residency. | Work scales with visible/resident patches; camera-still frame has zero terrain rebuild/upload; traversal and draw counters explain every visible patch. |
| M4 Render/physics/nav adapters | Height/normal/weight/hole GPU products and generation-qualified collision/navigation products. | RenderDoc proves Terrain draw/pixels and bounded uploads; collision/nav receipts identify matching or intentionally lagged generations. |
| M5 Region authoring | Transactional brush deltas, dirty-tile coalescing, undo budget and asynchronous derived updates. | Brush latency P50/P95/P99, dirty samples, rebuilt tiles, CPU/GPU bytes and undo bytes are recorded; cost scales with dirty area, not world area. |
| M6 Dynamic acceptance | Current-source executable, representative 4K/16K corpus, WPR/ETW CPU/IO/power capture and RenderDoc GPU capture. | No unexplained main-thread decode/cook/LOD work; no repeated stationary uploads; frame/brush/import budgets pass on named hardware and build/profile. |

## 5. Instrumentation contract

Each Terrain generation must expose source/build/profile hashes, resident/requested/visible tile counts, LOD histogram, cull reasons, queued/running/cancelled build jobs, dirty-region area, CPU artifact/staging bytes, GPU resident/upload/retire bytes, render/collision/navigation generations and cache hit/miss causes. Dynamic comparisons must freeze scene, camera path, resolution, quality, hardware, driver, build, provider and warm/cold state.

RenderDoc is reserved for pass/draw/resource/upload/pixel evidence. WPR/ETW owns CPU scheduling, main-thread stalls, IO and power evidence. Neither tool may be run against an old binary and reported as current-source Terrain evidence.

## 6. This review's implementation decision

No production source was changed. The apparent small fixes (a larger dimension constant, one checked multiplication, or one fewer extension allocation) would preserve the fatal source/artifact/runtime collapse and invent policy before a canonical owner exists. The first safe implementation is M0/M1 contract closure with tests; local micro-optimization before that cut would create another temporary path.

Static review is complete for `zircon_plugins/terrain`; dynamic acceptance remains pending and is not a milestone-completion claim.
