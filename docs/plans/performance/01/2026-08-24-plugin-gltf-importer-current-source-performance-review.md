---
title: Plugin glTF Importer Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/gltf_importer
status: static_complete_shared_source_preserved_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Public/Gltf/InterchangeGltfTranslator.h
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Gltf/InterchangeGltfTranslator.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Gltf/InterchangeGltfMesh.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Gltf/InterchangeGltfAnimation.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Mesh/InterchangeStaticMeshFactory.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Mesh/InterchangeSkeletalMeshFactory.cpp
  - dev/godot/modules/gltf/editor/editor_scene_importer_gltf.cpp
  - dev/godot/modules/gltf/gltf_document.cpp
  - dev/godot/modules/gltf/gltf_state.h
---

# Plugin glTF Importer Current Source Performance Review

## 1. Coverage and evidence state

The review covers **9/9 Rust files**, **3,931 physical / 3,671 non-empty lines**, **131,110 bytes**, **25 tests** and **4 ignored performance tests**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `880759e11aa6c7bf9162fb804e58090e038646d72e4698d734520321255d7a20`.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 254 | Exports ABI-v3 registration and save/restore/unload callbacks, but `invoke_command` is absent and no import bridge executes native work. |
| Runtime capability/registration | 2 | 287 | Advertises a Stable source/library/native provider and registers a priority-120 schema-v1 glTF importer. The first-party Runtime catalog selects this provider for `RuntimePluginId::GltfImporter`. |
| Runtime import/subassets | 2 | 1,297 | Synchronously parses source, loads sidecars/images, constructs meshes/materials/scenes, cooks derived geometry and emits subassets in one importer call. |
| Runtime tests/fixtures | 4 | 2,093 | Covers small functional fixtures and fragment microbenchmarks; no end-to-end import-scale, memory, scheduling or energy baseline. |

At review end, five tracked package files remained modified and one hot-path test file remained untracked from shared Plugins07 work: `dist/src/lib.rs`, `runtime/src/lib.rs`, `runtime/src/plugin.rs`, `runtime/src/subassets.rs`, `runtime/src/tests.rs` and `runtime/src/tests/hotpaths.rs`. Their current implementation was read but not edited or formatted. Per-file `rustfmt --check --edition 2021`, with `skip_children=true` on crate roots, passes **8/9** files; Dist retains a pre-existing import-order diff.

Managed Rust tests, WPR/ETW and RenderDoc were not run. This session has no executable managed Windows validator identity and no launchable current-source engine/editor binary. RenderDoc would only qualify post-import pixels, draws and GPU products; it cannot prove importer CPU ownership. The findings below are source-proven algorithm and architecture findings, not absolute latency or power claims.

## 2. Structural performance findings

### P0: product selection replaces the stronger Runtime v2 importer with an older split v1 authority

Three glTF contracts coexist:

1. `zircon_runtime` owns a functional schema-v2 importer at priority 10 with Mesh, Scene, Material, Texture, Data, AnimationSkeleton and AnimationClip outputs.
2. `zircon_plugins/asset_importers/model` owns a priority-100 diagnostic entry saying the split glTF package owns the format.
3. `zircon_plugins/gltf_importer` owns a functional priority-120 schema-v1 importer with no AnimationSkeleton or AnimationClip output.

The first-party Runtime catalog selects the split provider behind `base-runtime-plugins`, so normal product selection prefers the older implementation. Static contract probes confirm that the split path omits tangents, colors and the virtual-geometry request while the Runtime path handles all three, parses the transaction bytes once and emits real animation products. Maintaining two full implementations has already created behavior and performance regressions.

The target should follow Unreal Interchange ownership: one format translator plugin emits a typed translated graph and keyed payload interfaces; shared Runtime factories own canonical assets and derived cook. Zircon should move the current Runtime-v2 behavior behind that single plugin-owned translator/shared-factory boundary, then delete the duplicate Runtime format body and diagnostic shadow path in one hard cutover. Raising a version or priority before parity would only conceal the authority defect.

### P0: one import parses the glTF twice and reopens the main source

`AssetImportContext` already owns the transaction snapshot. The split importer first calls `gltf::Gltf::from_slice(&context.source_bytes)` to validate external buffers, then calls `gltf::import(&context.source_path)`, which reopens/reparses the glTF and loads buffers/images. This doubles main-document parse work, retains overlapping parser data and creates a TOCTOU boundary between the hashed snapshot and live path.

The Runtime-v2 decoder already demonstrates the correct local direction: `from_slice_without_validation(&context.source_bytes)`, required-extension admission, then one controlled load of buffers/images. The canonical source resolver still needs to own every external `.bin` and image read, publish URI/content hashes and enforce path/byte limits. Neither implementation currently proves sidecar currentness through the import outcome.

Unreal's translator reads source into one retained `GltfAsset`, builds translated nodes with payload keys, and defers texture/mesh/animation payload extraction through interfaces on that retained state. Zircon needs the same one-snapshot/one-translation authority even if the concrete parser differs.

### P0: the split importer forces virtual-geometry cook for every primitive

The split path calls `cook_virtual_geometry_from_mesh` with default settings for every primitive and never reads `context.virtual_geometry_cook_request()`. Mesh SDF is request-gated, but virtual geometry is not. The Runtime-v2 importer respects both recipe requests.

This makes merely selecting the plugin add O(index count plus cluster/hierarchy work) to every import, including MVP assets that did not request virtual geometry. It also couples format parsing to one derived product version. Virtual geometry, SDF, tangents, LOD, collision and platform compression must be versioned downstream recipes with explicit disabled states, cache keys, budgets and cancellation. Disabled means zero task time and zero retained payload.

### P0: one primitive is retained in at least three logical geometry products

During split import, parser arrays coexist with a constructed `Vec<MeshVertex>`. The importer clones each primitive into the root `ModelAsset`. Subasset emission then clones complete primitives into a `MeshN` model and calls `MeshAsset::from_model_primitive`, which walks the AoS vertices into separate position/normal/UV/tangent/color/joint/weight arrays and clones indices, SDF and virtual geometry.

After publication, the outcome can retain:

- root Model inline vertices/indices/virtual geometry;
- MeshN Model inline vertices/indices/SDF/virtual geometry;
- MeshN/Primitive Mesh attribute/index/SDF/virtual-geometry payloads.

That is three persistent logical copies before artifact serialization and cache staging, plus parser/source scratch during import. The Runtime-v2 path is materially closer to the correct product: it moves SDF/VG into a unique MeshAsset and leaves root/mesh model primitives as references with empty geometry. The hard cutover must preserve that single-payload behavior and add byte/accounting tests so future schemas cannot reintroduce inline copies.

### P0: scene dependency construction has quadratic traversal and cubic comparison risk

The split subasset builder emits a one-entity SceneAsset for every glTF node, but recursively walks that node's entire descendant subtree to collect dependencies. It recursively builds each scene tree and separately traverses it again for dependencies. Dependency uniqueness uses `Vec::contains`.

For a chain of `N` nodes, per-node subtree traversal performs `N + (N-1) + ... + 1 = O(N^2)` visits. Linear membership checks over each growing subtree dependency list raise worst-case comparison work toward O(N^3), while recursive depth adds stack-risk on adversarial files. Runtime-v2 upgrades deduplication to `HashSet`, but still repeats descendant traversals for every node, so the canonical path also requires redesign rather than copying as-is.

Translate nodes once into a dense node table plus child adjacency, validate hierarchy iteratively once, and derive dependency edges once in O(N+E). Scene artifacts reference root node indices/IDs. A node asset must not rematerialize or rescan its entire subtree.

### P0: skin binding is assigned to mesh identity instead of scene instance identity

The split importer builds `mesh_skin_assets_by_mesh` by scanning nodes and storing the first node skin found for each mesh, then clones that skin into every primitive MeshAsset. glTF binds skin on a node instance; one mesh can be instantiated with different skins. The Runtime-v2 implementation explicitly retains the same limitation.

Unreal encodes both mesh and skin in skeletal payload identity, duplicates translated mesh nodes when skins differ, and connects them to explicit skeleton dependencies. Its source even records that geometry and skeleton could become separate payload keys, which is the stronger Zircon target: canonical geometry remains unique, while scene instance/skeletal factory products pair geometry with skin/skeleton identity. First-node-wins maps are both semantically wrong and hostile to cache reuse.

### P0: animation and skin payloads are placeholders or duplicated text

The split importer emits animation `DataAsset` text saying channel import is not implemented. Skin and inverse-bind data are emitted as JSON `DataAsset` values that retain both pretty-printed text and `canonical_json`; inverse-bind matrices are also cloned into mesh-skin payloads. The Runtime-v2 path already emits typed AnimationSkeleton and AnimationClip products with channels/interpolation.

Unreal separates animation payload query types, returns typed curve/morph/baked products and runs large query batches through background `ParallelFor`. Zircon should hard-cut to typed skeleton/clip/channel schemas, stable target IDs and dedicated payload tasks. JSON text may remain a diagnostic/debug projection but cannot be a second authoritative Runtime payload.

### P0: import, decode, conversion and cook run as one synchronous work item

The split `FunctionAssetImporter` call performs document parse, external I/O, image decode, primitive conversion, normal generation, virtual geometry, optional SDF, texture/material/subasset construction and animation placeholders on one call stack. There is no translator-specific task graph, priority, progress, cancellation, stale-generation rejection or bounded memory admission.

Unreal's Static and Skeletal Mesh factories deduplicate payload keys and create async payload tasks; animation payload batches use background parallel work. Game-thread/finalization work is separated from heavy payload acquisition. Zircon needs an equivalent Runtime job graph: one admitted source translation, then independent buffer/image/mesh/animation payload tasks and downstream derived recipes. Main/editor thread work should stop after request admission and generation-safe publication.

### P1: textures and sidecars lack canonical identity and economical ownership

`gltf::import` eagerly decodes all images. The split subasset path clones RGBA bytes for texture publication, including repeated use of one image. Sampler distinctions are not part of the texture product identity, and external buffers/images are not published as hashed source dependencies. The Runtime-v2 path counts image uses and moves the final use while cloning only shared uses, and it admits `EXT_texture_webp`, but dependency receipts still need product-level proof.

Unreal assigns texture payload keys, carries normal-map usage in payload identity and delegates external files to an appropriate texture translator. Godot exposes explicit embedded-image handling in `GLTFState` and separates file append from scene generation. Zircon needs image-content identity plus sampler/view identity, lazy decode, shared immutable pixel leases, source dependency hashes and target/profile compression recipes.

### P1: advertised native behavior is registration-only state theater

Dist advertises `native_dynamic`, Stable maturity and importer capabilities, but has `invoke_command: None` and no bridge method. Its save/restore state consists of a process-global epoch unrelated to an import transaction. Native registration therefore cannot execute the advertised behavior, while lifecycle callbacks make the package appear more stateful than the importer is.

Native acceptance must require a callable import bridge or the same shared translator/factory service loaded in-process, plus source/native canonical artifact hash parity. Registration metadata and an epoch round trip are not importer functionality.

### P1: current benchmarks measure fragments rather than import scale

The four ignored tests cover cold registration allocation, borrowed input arrays/material movement and index-admitted normal generation. They do not include document parse, source reopen count, sidecar I/O, image decode, mesh conversion, scene graph traversal, geometry copy bytes, VG/SDF cook, animation, artifact serialization, cache publication, queue delay or main-thread residency.

Required counters include source/dependency reads and bytes, parse/decode/payload/cook/publish time, nodes/edges/meshes/primitives/vertices/indices/images/animations, logical and physical payload bytes, peak RSS, allocation count, queue wait, worker/main-thread CPU, cancellation latency, stale result rejection, cache outcomes and energy/import.

## 3. Reference-engine constraints

Unreal is the primary structural constraint:

- `InterchangeGltfTranslator.h/.cpp` implements mesh, texture, animation, variant and light-profile payload interfaces. Translation creates lightweight typed nodes and stable payload keys rather than eagerly publishing every final asset payload.
- Texture, static mesh, skeletal mesh and morph targets have distinct payload keys. Mesh payload acquisition validates/fixes product data after keyed extraction.
- A mesh instantiated with different skins receives different skeletal payload identity and explicit skeleton dependency instead of a first-node-wins mesh map.
- `InterchangeStaticMeshFactory.cpp` and `InterchangeSkeletalMeshFactory.cpp` reserve/deduplicate payload maps and create async task-lambda payload requests with CPU profiler scopes. Heavy mesh products are moved/combined in the async factory phase.
- Animation payload queries are typed and batches above ten entries are processed with background `ParallelFor`.

Godot is a secondary behavior reference, not the scheduling target. `EditorSceneFormatImporterGLTF` uses a `GLTFState`, applies explicit image/animation import settings, calls `append_from_file`, then separately calls `generate_scene`. `GLTFState` owns typed buffers, images, skins and animations, and the document routes extensions through explicit extension objects. Zircon should preserve this explicit recipe/state behavior while adopting Unreal's translated graph, payload and factory scheduling boundaries.

## 4. Dependency-ordered optimization plan

### M0: hard-cut to one executable schema-v2 provider

Define the split glTF plugin as the sole format translator and shared Runtime import/factory services as the sole artifact/cook owner. Move the current Runtime-v2 capabilities behind that boundary, prove source/library/native execution and artifact parity, then remove the duplicate Runtime format implementation and aggregate diagnostic shadow in the same migration. Do not select or advertise the split provider until tangents, colors, extensions, typed animation and recipe policy reach parity.

### M1: establish immutable source and sidecar authority

Parse `AssetImportContext::source_bytes` exactly once. Resolve external buffers/images through one sandboxed dependency resolver with normalized URI/path admission, byte/item/depth limits, content hashes, diagnostics and immutable leases. Publish every sidecar dependency and source generation in the import receipt; never reopen the main path from the translator.

### M2: translate to a compact graph and lazy payload keys

Produce one dense translated graph for scenes, nodes, meshes, primitives, materials, textures, samplers, skins, skeletons and animations. Validate indices/extensions/hierarchy once and use stable typed IDs. Root/model/scene products retain references and metadata only. Full mesh, texture and animation data are fetched by payload key when a factory/recipe requests them.

### M3: schedule bounded payload and recipe jobs

Create independent buffer/image/mesh/skin/animation payload jobs on the Runtime scheduler, with byte budgets, priorities, progress, cancellation and generation receipts. Schedule VG/SDF/tangent/LOD/collision/compression as downstream DDC recipes. Keep main/editor thread work to admission and final generation-current publication; reject stale jobs without replacing the last-good artifact.

### M4: publish one canonical product per identity

Store geometry once per canonical primitive/mesh payload and move completed buffers into the artifact. Pair geometry with skin/skeleton at scene instance or skeletal factory identity, not mesh source identity. Share immutable decoded image content while keeping sampler/view identity separate. Use typed AnimationSkeleton/AnimationClip/channel products; debug JSON/text is non-authoritative and optional.

### M5: make scene and dependency work linear

Build the node adjacency/dependency graph once, iteratively. Derive scene root closure and node direct edges without per-node subtree recursion. Use indexed/hash membership only where uniqueness is required. Add adversarial chain, star, DAG-like reference, duplicate-name, multi-scene, multi-skin shared-mesh and shared-image fixtures with operation counters that prove O(N+E) translation/dependency work.

### M6: version/cache/instrument every phase

DDC keys include source and sidecar content, translator/schema/factory/recipe versions, import settings, target platform, profile and backend. Store products only under configured non-C project/cache roots. Emit phase timing, bytes, allocation/RSS, queue, cache, cancellation and generation receipts for cold/warm/reimport paths.

### M7: qualify product behavior, rendering and power

Run fixed tiny, 10K, 1M and 10M triangle fixtures; 1/4/16 concurrent imports; deep/wide scene graphs; shared textures; animations/skins; and edit-burst reimports. Report p50/p95/p99 wall/CPU, throughput, peak RSS, bytes read/copied/written, worker/main-thread time, cache outcomes and energy/import. WPR/ETW must prove no source decode/cook on the editor/frame thread and exactly one main-file read. RenderDoc then verifies imported pixels, mesh/material/skin products, draw counts and optional VG output against the fixed recipe; it does not replace CPU profiling.

## 5. Quantified acceptance gates

1. Exactly one executable schema-v2 glTF provider exists per source/library/native profile. Source and native paths emit equal canonical artifact hashes for the same target recipe.
2. The main `.gltf`/`.glb` is parsed once from the transaction snapshot and reopened **zero** times. Every external buffer/image has one admitted content identity and dependency receipt.
3. Translation and scene dependency construction are O(source bytes + N + E + descriptor count), use no recursive per-node subtree rescan and remain stack-safe at the declared maximum depth.
4. Root/mesh/scene descriptors retain **zero** duplicate inline geometry after assetization. One canonical mesh payload owns vertex/index data; copy/retained-byte counters enforce the boundary.
5. Disabled VG/SDF/tangent/LOD/collision/compression recipes execute zero work and retain zero payload. Enabled recipes are independently cancellable, cacheable and versioned.
6. One mesh referenced by multiple skins produces correct instance/skeletal identities without mutating or cloning the canonical geometry payload. Typed skeleton/clip products replace animation placeholder text.
7. Heavy parse/decode/payload/cook work consumes zero editor/frame-thread CPU after admission. Latest-generation publication, bounded cancellation and last-good retention are proven under edit bursts.
8. Dynamic comparisons fix hardware, storage, build, source corpus, settings, target and sample window. Unreal/Godot establish architecture and workload semantics; no latency, power or parity claim is accepted from source inspection alone.

## 6. Current disposition

- Static review is complete for **9/9** package Rust files and the selected Runtime/catalog/reference-engine product paths.
- No production edit was made: the package is under active shared Plugins07 changes, and the required fix is a cross-owner hard cutover rather than a safe local micro-optimization.
- Rust compilation/tests, current-source import/reimport, WPR/ETW, RenderDoc, memory, power and soak evidence remain pending.
- The package is not eligible for protected review-ledger acceptance, milestone commit or WeCom completion notification.
