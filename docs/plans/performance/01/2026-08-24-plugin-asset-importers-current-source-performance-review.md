---
title: Plugin Asset Importers Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/asset_importers
  - zircon_plugins/first_party_runtime_catalog
  - zircon_runtime/src/asset
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Mesh/InterchangeStaticMeshFactory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerThreadRunnable.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Factories/DataTableFactory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DataTable.h
---

# Plugin Asset Importers Current Source Performance Review

## 1. Coverage and current product truth

The review covers **26/26 Rust files**, **4,376 physical / 3,974 non-empty lines**, **155,544 bytes**, **48 test markers** and **6 ignored performance tests**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `8072d2b16ad6a63095b2867ce87e41a1fc2d8bf8e0f3c73e1b8ca700b702c03b`.

| Module/folder | Rust files | Physical lines | Execution truth |
|---|---:|---:|---|
| `audio` | 4 | 410 | Publishes WAV/codec/Opus descriptors but registers no callable importer. |
| `data` | 4 | 948 | Synchronously parses TOML/JSON/YAML/XML and stores raw text plus a generic JSON tree. |
| `model` | 10 | 1,702 | Synchronously imports PLY/STL/DXF, unconditionally cooks virtual geometry, and publishes glTF/OBJ/native diagnostic placeholders. |
| `shader` | 4 | 918 | Synchronously parses/validates WGSL/GLSL/SPIR-V through Naga and emits/stores WGSL. |
| `texture` | 4 | 398 | Publishes image/container/PSD/native descriptors but registers no callable importer. |

Seven production files contain shared uncommitted changes and were preserved. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **24/26** files; the two failures are shared `shader/runtime/src/lib.rs` and `plugin.rs` formatting differences. Managed Rust tests were not run because the current validation session has no executable Windows validator identity. WPR/ETW, RenderDoc and power measurements remain pending because no launchable current-source product exists.

## 2. Structural performance findings

### P0: the aggregation boundary mixes catalog metadata, error placeholders and real compilers

Audio and texture describe importers but rely on the default `RuntimePlugin::register`, so they contribute no executable handlers. Model contributes real PLY/STL/DXF handlers beside diagnostic-only glTF/OBJ/native handlers. Shader contributes real WGSL/GLSL/SPIR-V handlers beside a diagnostic-only HLSL/CG/FX handler. Data is fully executable. These five packages therefore do not share one meaningful responsibility despite being shaped as parallel families.

The Runtime built-in catalog instead gives canonical product identities to the split audio, texture, glTF, OBJ and WGSL packages. `RuntimePluginId` contains `AssetImporterData`, `AssetImporterModel` and `AssetImporterShader`, but the first-party linked catalog has no dependency, feature or provider branch for any of them. It also has no aggregate audio/texture identity or provider branch. Generated/plugin manifests can advertise rows that the product cannot execute.

This is the first performance defect: duplicate discovery and diagnostic surfaces obscure which algorithm owns a format, while the real data/model/shader code is unreachable from the normal product. Hard-cut each format to one executable provider and keep catalog metadata generated from that provider. An unavailable optional backend belongs in capability state, not as an importer that wins selection only to return a diagnostic.

### P0: overlapping registrations are not composition-safe

The split glTF, OBJ and WGSL handlers use priority 120 while aggregate placeholders use 100, so manual coexistence silently selects the split implementation. Texture is worse: aggregate PSD and split PSD both use priority 100. `AssetImporterRegistry` rejects duplicate matchers at the same priority, so enabling both packages can fail registration rather than produce a deterministic product.

Audio/texture aggregate manifests also declare format capabilities that their runtime package capability list does not provide. All four dist crates publish registration manifests with `invoke_command: None`, no bridge methods and diagnostics saying execution remains hosted by the runtime module. Source/library/native readiness is therefore not equivalent.

### P0: all real import work runs synchronously without aggregate admission

Data DOM parsing, Naga parsing/validation/emission, PLY/STL/DXF parsing, normal generation, vertex materialization, virtual-geometry cook and subasset publication all occur in the importer call stack. There is no shared work request carrying source generation, recipe key, target profile, byte reservation, priority, progress, cancellation or deadline. Source size, node/element count, nesting depth, vertex/index/face count and compiler memory have no package-level admission ceiling.

Unreal's `InterchangeStaticMeshFactory` separates game-thread object creation from asynchronous mesh payload/import work. Its shader compiler submits jobs to managed workers, tracks pending results, caches by job inputs/DDC and dynamically suspends/reschedules workers against an explicit process memory limit. Zircon needs those execution boundaries before format-specific loop tuning can be accepted.

### P0: structured data keeps multiple whole-document representations without a schema

`AssetImportContext` already owns complete source bytes. Every data importer then calls `source_text()`, allocating another complete string, and successful `DataAsset` retains that string plus a `serde_json::Value` tree. TOML temporarily owns a `toml::Value` before converting it to JSON. XML retains the source string while `roxmltree` indexes it, then recursively allocates a second neutral tree. There is no byte/node/depth/string budget, schema identity, row key, field validation, lazy projection or query-oriented artifact.

For a 100 MiB valid UTF-8 document, source bytes plus stored text are a **200 MiB lower bound** before either DOM, per-key/string allocation, parser index, artifact serialization or typed-load clones. The generic JSON conversion can exceed source size substantially for small keys and deeply nested collections. Recursive XML conversion also permits stack exhaustion from adversarial depth.

Unreal DataTable import requires a `RowStruct`, preserves import provenance, exposes row-key and extra/missing-field policy, collects import problems and marks the runtime asset lazy-on-demand. Zircon should not copy DataTable APIs, but must choose a versioned typed schema and runtime query product instead of treating arbitrary TOML/YAML/XML as one permanent raw-text-plus-JSON asset.

### P0: model import duplicates final geometry and generates an optional derived product by default

PLY holds generic property maps while collecting positions, normals, UVs and indices. STL holds parser vertices/faces while collecting another positions/indices pair. DXF first materializes a complete `Drawing`, then expands every supported triangle into new, unwelded positions. All paths create `Vec<MeshVertex>`, clone indices, and immediately call `cook_virtual_geometry_from_mesh` with `VirtualGeometryCookConfig::default()`.

This bypasses the Runtime `VirtualGeometryCookRequest`, whose default is explicitly Disabled so MVP projects do not generate payloads without a consumer. The same call then constructs a root `ModelPrimitiveAsset` and a `MeshAsset` subasset. `MeshAsset::from_model_primitive` recollects position, normal, UV, tangent, color, joint-index and joint-weight arrays, clones indices, SDF and virtual geometry, while the root primitive keeps its original AoS vertices and indices.

The declared `MeshVertex` fields total 96 bytes per vertex. Even with default UV1 omitted from the final SoA mesh, root + subasset retain at least **184 bytes per vertex**; one million vertices therefore retain **175.5 MiB** before indices. A two-million-triangle mesh adds six million indices in both assets, another **45.8 MiB**, yielding a **221.3 MiB final-geometry lower bound** before parser objects, source bytes, virtual-geometry pages, SDF, artifact staging or allocator overhead.

The target publishes one immutable source mesh identity and references it from Model. Render mesh, LOD, collision, SDF and virtual geometry are independent, optional, content-addressed build products. Parser/cook jobs consume bounded chunks or move ownership; they must not rematerialize every vertex attribute merely to express a subasset relationship.

### P0: the shader path is validation, translation and archival duplication rather than a shader compiler service

WGSL parsing builds a Naga module and then stores two complete identical strings in `ShaderAsset.source` and `wgsl_source`. GLSL stores the complete original plus emitted WGSL. SPIR-V converts every input byte to two hexadecimal characters and also stores emitted WGSL. A 100 MiB WGSL input therefore has a **300 MiB source/string lower bound** across context bytes and the two asset strings before Naga IR. A 100 MiB SPIR-V input has the same minimum before emitted WGSL because hexadecimal source alone is 200 MiB.

Validation uses `Capabilities::all()` rather than the selected device/target profile. Includes, source-file dependencies, definitions, permutations, reflection/layout, compiler version, target bytecode, diagnostics provenance and pipeline-cache identity remain empty or absent. No job cache or duplicate coalescing exists. The split WGSL importer independently owns the same extension, reinforcing the authority problem.

Unreal's ShaderCompiler provides the applicable architecture: explicit compile jobs, pending-map accounting, local/distributed workers, memory-aware worker control, input-hash job cache and DDC. Zircon's production product should be target/profile-specific compiled artifacts plus reflection and dependency receipts. Original source stays in source authority; SPIR-V is never hex-expanded as the canonical runtime asset.

### P1: existing microbenchmarks optimize fragments that are not the product bottleneck

The six ignored gates compare single-pass XML traversal, direct descriptor builders, borrowed normals, stack versus heap surface-point lists and static shader-stage names. These changes may be locally valid, but none measures full decode/parse, DOM/IR/mesh peak RSS, virtual-geometry cook, artifact publication, cache hit, duplicate work, main-thread time or energy. A benchmark can report a large percentage improvement while the unreachable provider or whole-payload duplication remains unchanged.

Required fixtures cover `1/1,000/100,000` structured rows/nodes, deep/adversarial nesting, 1M+ vertex/index meshes, malformed expansion counts, shader include/permutation sets and repeated concurrent imports. Every performance receipt must bind provider identity, source/recipe/target hash, cache state and current-source binary fingerprint.

## 3. Reference-engine constraints

1. Unreal Interchange obtains mesh payloads as tasks, creates/updates engine objects on the game thread, and performs the heavy import/build phase asynchronously. Optional Nanite/collision/LOD products do not belong to an unconditional parser return value.
2. Unreal ShaderCompiler queues target-specific jobs, tracks pending results, supports worker processes/distributed backends, bounds worker memory, caches by input hash/DDC and records compile statistics.
3. Unreal DataTable binds imported records to a row schema, records source provenance and validation problems, and exposes lazy runtime lookup rather than a universal duplicate DOM.
4. The previously reviewed Unreal audio and texture paths separately constrain streamed/cooked products, derived-data identities, cancellation, memory admission and residency. This aggregation package must route to those owners, not create fallback implementations.

The transferable system is `one selected provider -> immutable source/dependency receipt -> versioned typed recipe -> admitted build jobs -> immutable target artifacts/subassets -> bounded runtime/editor leases`. Catalogs and diagnostics describe that system; they do not substitute for it.

## 4. Dependency-ordered optimization plan

### M0: hard-cut format authority and product closure

Create one format-to-provider table for all advertised extensions. Remove aggregate audio/texture descriptors or absorb them into the selected split providers. Remove glTF/OBJ/WGSL diagnostic shadows. Select and link Data, Model and Shader providers explicitly or fail capability readiness. Make duplicate matcher, missing capability and source/library/native parity checks product gates.

### M1: converge source, recipe, subasset and artifact identity

All importers consume immutable source/dependency leases and versioned settings. Artifact keys include provider/algorithm version, typed schema, target/profile, optional derived-product policy and every transitive dependency. Model-to-mesh relations reference one payload identity; Data and Shader source text remains in source authority rather than embedded redundantly in runtime artifacts.

### M2: add bounded import/compiler jobs

Split probe/header, parse, validation, conversion, optional derived cook, artifact encode and publication into scheduler jobs. Reserve source/DOM/IR/geometry/scratch/artifact bytes; enforce size/count/depth limits. Carry priority, progress, cancellation/deadline and generation. Coalesce identical requests and preserve last-good products on failure.

### M3: define typed Data products

Add schema/key/version and field policy to data recipes. Import into typed row/record chunks with structured diagnostics and migration receipts. Preserve raw source once, expose lazy/indexed runtime access, and avoid permanent generic JSON when the consumer needs a concrete schema. XML depth/node limits are mandatory.

### M4: rebuild Model/Mesh as optional derived products

Use bounded parser backends and explicit coordinate/unit/material/topology policies. Weld/deduplicate only under a versioned recipe. Publish one immutable mesh payload; Model references it. Cook LOD, collision, SDF and virtual geometry only when requested, through independent cacheable jobs and budgets. Remove unconditional default virtual geometry and all root/subasset full-payload clones in the owner migration.

### M5: establish the Shader compiler service

Make preprocessing/include discovery, parse/validate, permutation expansion, target compilation, reflection and pipeline prewarm separate cached jobs. Keys include compiler/backend version, source graph, definitions, stage, entry point, target/device capabilities and optimization/debug policy. Run compilers in bounded workers/sandboxes, return structured diagnostics, and publish binary/reflection artifacts without duplicated source or SPIR-V hex.

### M6: instrument and dynamically qualify

Record provider selection, phase p50/p95/p99, queue/main/worker CPU, source/DOM/IR/geometry/cooked/artifact bytes, allocations/peak RSS, cache/coalescing state, cancellations and energy. Use WPR/ETW for CPU, I/O, scheduling, process memory and power. RenderDoc applies only to shader/mesh/texture runtime installation, draw/copy/resource lifetime, pixels and VRAM after a current-source binary exists.

## 5. Acceptance gates

| Gate | Required evidence |
|---|---|
| A1 | Every advertised matcher resolves to exactly one product-selected executable provider; no diagnostic shadow or equal-priority collision remains. |
| A2 | Source/library/native forms expose equivalent callable behavior or fail capability readiness explicitly. |
| A3 | Import work is admitted, cancellable and generation-bound; main/editor thread performs bounded request/publication work only. |
| A4 | Data inputs obey byte/node/depth/string budgets and publish schema-validated products without permanent raw-text-plus-generic-DOM duplication. |
| A5 | Model references one mesh payload; root/subasset publication performs zero complete vertex/index/VG/SDF clones. |
| A6 | Virtual geometry, SDF, collision and LOD cook only under explicit recipes and independent artifact/budget keys. |
| A7 | Shader artifacts include source graph, target/profile, compiler version, definitions, reflection and diagnostics; source strings/SPIR-V are not duplicated into runtime products. |
| A8 | Concurrent same-key imports execute one build and share a generation-bound terminal result; stale work cannot publish. |
| A9 | Corpus tests report cold/warm p50/p95/p99, throughput, queue depth, clone bytes, peak RSS, cache ratio, main/worker CPU and energy. |
| A10 | WPR/RenderDoc evidence comes from the reviewed current-source executable and matched output-quality/profile fixtures; static models are not reported as measurements. |

## 6. Validation record

- Static package coverage: complete, 26/26 Rust files; catalog, registry, asset schema and Model-to-Mesh conversion reviewed to terminal ownership.
- Formatting: 24/26 pass; two shared Shader files retain formatting debt and were not rewritten.
- Source snapshot: shared dirty work preserved; no production edit was made because all material fixes require provider/schema/scheduler ownership cutovers.
- Rust tests: not executed; the managed Windows validator identity is unavailable and no raw Cargo lane was substituted.
- WPR/ETW/RenderDoc/power: pending until a launchable current-source product exists.
- Protected ledgers, milestone commit and quantified WeCom completion notice remain pending until dynamic acceptance.
