---
title: Plugin OBJ Importer Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/obj_importer
status: static_complete_shared_source_preserved_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Mesh/InterchangeOBJTranslator.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Public/Mesh/InterchangeOBJTranslator.h
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Mesh/InterchangeStaticMeshFactory.cpp
  - dev/godot/editor/import/3d/resource_importer_obj.cpp
---

# Plugin OBJ Importer Current Source Performance Review

## 1. Coverage and evidence state

The current package surface is **4/4 Rust files**, **1,068 physical / 970 non-empty lines**, **38,735 bytes**, **13 tests** and **2 ignored microbenchmarks**. Its package-relative `path + NUL + LF-normalized raw bytes + NUL` SHA-256 is `eaef4c8acffa9978a7b697313af542bf7c37ff8da261064f569cb9d07982657e`.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 108 | Exports ABI and registration metadata, but has no import command or bridge method. Its own diagnostic says importers remain hosted by the Rust Runtime module. |
| Runtime capability | 1 | 34 | Declares package/capability/native metadata only. |
| Runtime importer | 1 | 689 | Parses OBJ synchronously, builds model and mesh payloads, always cooks virtual geometry, optionally cooks SDF, and contains package/import tests plus an ignored normal-clone microbenchmark. |
| Runtime plugin | 1 | 237 | Registers one source importer and contains an ignored cold registration-plan microbenchmark. |

The two Runtime implementation files already had shared Plugins07 changes at review start: `runtime/src/lib.rs` and `runtime/src/plugin.rs` (`+366/-99` combined). They were read as current source and were not edited or formatted. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passed **4/4** files.

Managed Rust tests, WPR/ETW and RenderDoc were not run. This session has no executable managed Windows validator identity and no launchable current-source engine/editor binary. Existing tests also create files through `std::env::temp_dir()`, which would place artifacts on C on this machine, so they are not an acceptable substitute for the required non-C managed validation fixture.

## 2. Structural performance findings

### P0: three OBJ authorities disagree, and the advertised plugin is not linked into product selection

The engine has three OBJ contracts:

1. `zircon_runtime` registers `zircon.builtin.model.obj` and owns a functional importer.
2. `zircon_plugins/asset_importers/model` publishes a priority-100 diagnostic importer saying OBJ is provided by the split package.
3. `zircon_plugins/obj_importer` publishes a priority-120 functional importer with a second implementation.

The first-party Runtime catalog recognizes `RuntimePluginId::ObjImporter` in shared IDs and generated manifest tests, but has no `zircon_plugin_obj_importer_runtime` dependency, feature, or provider branch. Selecting the manifest therefore produces no linked registration. The native Dist cannot fill the gap because it has `invoke_command: None`, no bridge methods and no importer implementation.

This is not merely duplicate source. If the split importer were wired today, its higher priority would replace the more complete Runtime builtin with a path that rereads the file, ignores materials and forces virtual-geometry cook. If the aggregate model diagnostic is registered without the split provider, its priority can shadow the builtin with an unavailable result. Performance work must begin with one product authority, not tune all three.

The target architecture should follow Unreal Interchange's ownership split: an OBJ translator plugin owns format parsing and source-node/payload keys; shared Runtime asset factories own canonical mesh/material artifacts and derived cook. After the split provider is executable in source and native profiles, remove the format-specific Runtime builtin and diagnostic shadow path in one hard cutover.

### P0: the split plugin discards the transaction snapshot and reads the OBJ twice

`ProjectManager::prepare_full_generation` reads every source into `AssetImportContext::source_bytes`, records its byte count and hashes it. The split plugin ignores those bytes and calls `tobj::load_obj(&context.source_path)`, reopening and reparsing the live path. A normal import therefore performs at least two complete main-file reads and keeps the transaction snapshot while `tobj` allocates its own arrays. The parsed bytes can also differ from the bytes that supplied the source digest if the file changes between the two reads.

The Runtime builtin already demonstrates the correct local primitive: `tobj::load_obj_buf(Cursor::new(context.source_bytes.as_slice()), ...)`, with companion MTL reads taken from `source_file_snapshot` when available. Maintaining the second plugin implementation has reintroduced an I/O and consistency defect that was already fixed elsewhere.

The importer contract should expose one immutable source snapshot plus dependency-resolver reads. The translator must not reopen the main source path. Companion reads need URI/path admission, byte limits, hashing and dependency publication so reimport currentness includes MTL and texture inputs.

### P0: import, conversion and derived cook are one serial call stack

`AssetImporterHandler::import` is synchronous. Full project generation loops `for source in sources`, then calls `self.importer.import_context(&import_context)` inline. OBJ parsing, normal generation, vertex conversion, virtual-geometry BVH construction, optional mesh-SDF voxel cook, subasset conversion and artifact preparation therefore execute serially in the import phase. There is no OBJ-specific task graph, progress, cancellation, stale-generation rejection or worker budget.

Unreal's StaticMesh factory creates per-payload tasks on an async thread, performs the heavy factory phase in `ImportAsset_Async`, and limits game-thread work to object setup/finalization. It emits CPU profiler scopes around payload/factory phases and starts asynchronous mesh build. Zircon's existing environment-IBL parallel executor does not schedule OBJ work.

Parsing/translation, per-mesh payload construction and each derived-data recipe need explicit dependency tasks. Main/editor thread work should be limited to admission and generation-safe publication. Cancellation must stop work between bounded parse/payload/cook units; a newer reimport must not install stale artifacts.

### P0: one import publishes two complete geometry representations

Each parsed primitive first owns `Vec<MeshVertex>` plus `Vec<u32>` in the root `ModelAsset`. `MeshAsset::from_model_primitive` then walks the vertices repeatedly to allocate position, normal, UV0, optional UV1, tangent, color, joint-index and joint-weight arrays, and clones the index buffer. `MeshVertex` carries **96 bytes/vertex**, so the root and mesh subasset retain approximately `192 * V + 8 * I` raw bytes before allocator/map overhead and derived data.

For `V=1,000,000` and `I=3,000,000`, those two persistent raw representations alone are about **216,000,000 bytes / 206 MiB**. A complete `tobj` position/normal/UV/index representation adds about **44,000,000 bytes / 42 MiB** while conversion is active, excluding the source snapshot, names, maps and scratch. OBJ single-index expansion can raise `V` further at position/normal/UV seams.

Virtual geometry is cloned into the mesh subasset while remaining in the root primitive, so its hierarchy/page vectors are also retained twice. Mesh SDF is cloned by `from_model_primitive`, immediately dropped by assignment, and then moved from the primitive; a dense voxel payload can therefore be copied transiently for no product value.

The canonical artifact must store geometry once. A model primitive should publish a stable reference plus material/section metadata; the authoritative `MeshAsset` should own vertex/index/derived payloads. Publication should move completed buffers rather than deinterleave a second retained copy, and root serialization must not preserve inline geometry after assetization.

### P0: the split plugin forces virtual-geometry cook and ignores the canonical recipe

The split plugin calls `cook_virtual_geometry_from_mesh` for every valid primitive with default settings. It never reads `context.virtual_geometry_cook_request()`. The canonical Runtime importer does read that request; its default is disabled and it cooks only when settings admit the recipe. The plugin also does not call `assign_virtual_geometry_vertex_ordinals` on the root primitive, while `MeshAsset::from_model_primitive` synthesizes ordinals only for the duplicated mesh attributes.

Thus selecting the split provider changes both performance and data semantics: every OBJ pays an O(indices plus cluster hierarchy) derived cook, obtains duplicated virtual-geometry data, and can expose different root/subasset vertex channels. This cook is a simple contiguous 64-triangle clustering plus a four-way hierarchy, not evidence that the asset is ready for the complete virtual-geometry Runtime product.

Virtual geometry, SDF, tangents, LODs, collision and platform compression must be explicit versioned recipes. They should execute in the derived-data worker/cache layer, keyed by canonical mesh content, recipe/compiler version, target, profile and relevant dependencies. A disabled recipe must do zero cook work and retain zero payload.

### P0: materials, sections and dependencies are parsed by the library and then discarded

The plugin binds `let (models, _) = tobj::load_obj(...)`, dropping all returned materials. It does not publish MTL or texture dependencies, material assets, material slots, section boundaries, smoothing groups, source transforms or diagnostics for missing companion files. Subasset IDs are ordinal strings such as `#Mesh0/Primitive0`; inserting or reordering an object changes downstream identities even when named OBJ groups are unchanged.

Unreal's translator parses `mtllib` and material/texture properties, builds mesh/material/texture nodes, connects material-slot dependency UIDs, uses normalized texture payload keys, and keys mesh payloads by group name. Godot likewise parses MTL, reports missing texture dependencies, splits surfaces on material changes and exposes explicit import settings.

Zircon needs stable normalized group/object identity with deterministic collision disambiguation, material-slot/section mapping and dependency hashes. Unsupported OBJ/MTL features must produce bounded diagnostics rather than disappear silently. The source node graph should be cheap metadata; full mesh payload construction should remain lazy/task-owned.

### P1: current microbenchmarks optimize cold fragments, not import scale

The shared change that borrows complete normal arrays removes one `normals.to_vec()` allocation and is directionally correct. Its ignored benchmark repeatedly clones a synthetic 524,288-float slice but excludes parse, vertex normalization, cook, subasset publication, serialization and peak RSS. The plugin registration benchmark repeatedly constructs 16,384 one-importer plans to measure removal of a one-element `Vec`; registration is a cold path and this does not address product import cost.

There are no tests or counters for source opens/bytes, dependency reads, parse/payload/cook phase time, vertices/indices/materials, single-index expansion, peak/retained bytes, geometry copies, worker queue delay, cancellation, cache outcomes, stale publications, main-thread time or energy. Existing functional fixtures cover only tiny triangles/multiple objects and use host temp storage.

## 3. Reference-engine constraints

Unreal is the primary architectural constraint:

- `InterchangeOBJTranslator.h/.cpp` separates translation into a node container from keyed mesh/texture payload retrieval. Group names become mesh UIDs/payload keys; materials and textures are explicit dependency nodes rather than discarded parser output.
- `InterchangeOBJTranslator.cpp` uses a line visitor and keyword dispatch, validates source existence/data and creates cheap bounds/vertex/polygon metadata before the factory requests full mesh payloads. Zircon does not need to copy this parser, but it needs the same metadata-versus-payload boundary.
- `InterchangeStaticMeshFactory.cpp` creates payload tasks on async or game threads according to policy, deduplicates payload keys, marks profiler scopes, performs the heavy factory phase in `ImportAsset_Async`, and isolates game-thread finalization/build ownership.
- The Runtime build path uses a content hash as identity and avoids a slow mesh-description commit when only Runtime render data is required. Zircon should similarly avoid retaining authoring and product copies without a consumer.

Godot is a secondary behavior reference. Its OBJ importer streams source lines, preserves materials/surfaces, reports missing dependencies, exposes tangents/LOD/shadow/lightmap/compression choices and writes generated files. Its synchronous editor importer and repeated scans are not the target for Zircon scheduling, but its explicit recipe options are preferable to unconditional hidden cook.

## 4. Dependency-ordered optimization plan

### M0: hard-cut to one executable OBJ provider

Choose the split OBJ plugin as the sole format translator and shared Runtime asset services as the sole artifact/cook owner. Link `RuntimePluginId::ObjImporter` through the first-party catalog/profile, prove source and native provider behavior, then remove the Runtime format-specific builtin and aggregate diagnostic shadow path in the same migration. Until closure, report the split capability as unavailable rather than silently omitting it or advertising Stable.

### M1: define immutable source and dependency translation

Parse only `AssetImportContext::source_bytes`; add resolver-owned bounded reads for MTL and texture companions. Publish every companion URI/hash and missing/denied diagnostic. Build a deterministic translated node graph for objects/groups, mesh payload keys, sections/material slots, materials and textures. Derive stable IDs from normalized source identity plus named group/object/material identity with deterministic duplicate disambiguation, not array order alone.

### M2: separate metadata, payload and derived-data tasks

Return cheap translated metadata first. Schedule independent mesh payload tasks by key on the Runtime job system with byte/item/time budgets, priority, progress, cancellation and generation receipts. Schedule virtual geometry, SDF, tangents, LOD, collision and compression as downstream recipe tasks. Only generation-current results may publish; failed/cancelled recipes preserve the last-good artifact.

### M3: publish one canonical mesh payload

Make each authoritative mesh subasset own geometry once. Root models retain stable mesh/material references and lightweight primitive/section metadata, not duplicate vertex/index/VG/SDF payloads. Move parser/payload buffers into the artifact representation where layouts permit; otherwise account for one bounded conversion scratch and drop it before publication. Remove the transient SDF clone and persistent virtual-geometry clone.

### M4: make cook policy explicit and cacheable

Version the OBJ translator, canonical mesh schema and every derived recipe. Keys include source/dependency content, import settings, translator/schema/cooker versions, target platform, feature/profile and backend. Disabled recipes perform no work. Store artifacts and receipts only under configured non-C project/cache roots; native packaging consumes the same artifacts rather than registration-only metadata.

### M5: add product instrumentation and adversarial fixtures

Use deterministic fixtures for tiny, many-object, many-material, seam-heavy, missing-normal, malformed-index, malformed/large-line, missing/escaping MTL/texture, duplicate-name and large meshes. Add counters for source/dependency reads and bytes, parse/payload/cook time, V/I/material/section counts, expansion ratio, allocations/peak RSS, retained/copy bytes, queue wait, cancellation latency, cache hit/miss, stale results and main-thread time.

### M6: qualify import, reimport, render and power

Run `1/4/16` concurrent imports and edit-burst reimports on fixed `10K/1M/10M` triangle fixtures. Report p50/p95/p99 wall and CPU time, throughput, peak RSS, bytes read/copied/written, worker/main-thread time, cache outcomes and energy/import. WPR/ETW must show no heavy parse/cook on the editor/frame thread and no second main-file read. RenderDoc is used only after import to verify sections/material bindings, virtual-geometry recipe output, draw counts and pixels; it is not a CPU importer profiler.

## 5. Quantified acceptance gates

1. Exactly one executable OBJ provider is selected for each source/native profile; an enabled but unavailable provider returns a typed failure instead of silent omission or diagnostic shadowing.
2. The main OBJ file is read once into the transaction snapshot and reopened **zero** times by the translator. Every MTL/texture read is admitted, hashed and published as a dependency.
3. Import complexity is O(source bytes + emitted vertices + indices + enabled recipe work). Root publication retains **zero** duplicate inline geometry after mesh assetization; persistent raw geometry is one canonical payload, not the current `192 * V + 8 * I` duplicate floor.
4. Disabled VG/SDF/LOD/collision/tangent recipes consume zero task time and publish zero payload. Enabled recipes have versioned cache keys, bounded scratch and measurable hit/miss receipts.
5. Main/editor-thread parse, conversion and cook time is zero after admission. Cancellation and stale-generation rejection complete at bounded task checkpoints, and every accepted task reaches one terminal state.
6. Material/texture/section identity and dependencies remain stable under unrelated object reorder; duplicate names are deterministically disambiguated. Source and native paths produce matching canonical artifact hashes for the same target recipe.
7. Dynamic comparisons fix hardware, storage, build, source, settings, target and sampling window. Unreal/Godot establish architecture and workload-scale constraints; no latency, power or parity claim is accepted from source inspection alone.

## 6. Current disposition

- Static review is complete for **4/4** Rust files and all package manifests/tests.
- No production edit was made: the touched implementation files are shared with active Plugins07 work, and a local plugin/catalog patch before canonical-owner cutover would activate a slower and less complete authority.
- Rust compilation/tests, current-source product import/reimport, WPR/ETW, RenderDoc, power and soak evidence remain pending.
- The package is not eligible for protected review-ledger acceptance, milestone commit or WeCom completion notification.
