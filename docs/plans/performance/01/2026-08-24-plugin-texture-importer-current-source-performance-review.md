---
title: Plugin Texture Importer Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/texture_importer
  - zircon_plugins/first_party_runtime_catalog
  - zircon_runtime/src/asset
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Texture/InterchangeImageWrapperTranslator.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Texture/InterchangeTextureFactory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TextureDerivedDataTask.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TextureCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Developer/TextureCompressor/Private/TextureCompressorModule.cpp
  - dev/UnrealEngine/Engine/Source/Developer/TextureFormatOodle/Source/Private/TextureFormatOodle.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/AsyncTextureStreaming.cpp
---

# Plugin Texture Importer Current Source Performance Review

## 1. Coverage and evidence state

The primary package review covers **41/41 Rust files**, **13,256 physical / 12,092 non-empty lines**, **463,380 bytes**, **224 test markers** and **16 ignored performance tests**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `b2db24997c195e736b1e2ac0814f5867dbdcbdab59926ba5006cd5e41cc2fc7c`.

| Module/folder | Rust files | Physical lines | Reviewed responsibility |
|---|---:|---:|---|
| `dist` | 1 | 108 | Registration and lifecycle ABI metadata; no callable native import bridge. |
| Runtime core | 6 | 1,052 | Capability, provider registration, image/PSD/container dispatch, manifest-source loading and normal convention. |
| Container parsers | 9 | 4,408 | ASTC, DDS, KTX1/KTX2, DFD and supercompression validation/rewriting. |
| Derived products | 6 | 2,697 | Array/cubemap assembly, offline mip generation and BC5 encoding. |
| Tests | 19 | 4,991 | Parser, descriptor, manifest, cubemap, mip, BC5 and registration coverage. |

The dependent first-party catalog, Runtime built-in importer, `TextureAsset`/payload schema, artifact cache conversion, typed loading, GPU resource construction and residency transition path were followed to their terminal CPU/GPU ownership boundaries. The current package contains extensive shared uncommitted changes in 21 tracked files plus two untracked files; that snapshot was reviewed but not edited or formatted.

Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **40/41** files. The only failure is export ordering in `runtime/src/lib.rs`, an otherwise unmodified file. Rust tests were not executed because this session has no executable managed Windows validator identity. WPR/ETW and RenderDoc were not run because no launchable current-source engine/editor executable exists. The analysis below is therefore source-proven structure and lower-bound data-volume modeling, not measured latency, energy or engine parity.

## 2. Structural performance findings

### P0: the product declares TextureImporter but never selects this provider

The package registers standard images at priority 120, PSD at 100, DDS/KTX/KTX2/ASTC at 90 and cube/array manifests at 130. The first-party Runtime catalog exposes a `TextureImporter` identity in generated metadata, but its provider function and default/base feature closure only link `RuntimePluginId::Texture` to `zircon_plugin_texture_runtime`; no branch links `zircon_plugin_texture_importer_runtime`. Meanwhile the Runtime built-in `zircon.builtin.texture.image` importer remains executable for standard images.

This produces three conflicting truths: a catalog record, an unreachable 13k-line importer, and a lower-priority built-in implementation. The first optimization is an authority hard cutover to exactly one selected provider per format. Capability must fail closed until source/library/native execution and product-profile selection agree.

### P0: import, derived build and publication are one synchronous, unbounded call stack

The main path performs decode, settings mutation, normal conversion, runtime mip preparation, offline mip generation, optional BC5 transcode and outcome publication in one call. Manifest imports additionally perform direct synchronous filesystem reads and complete child-image decodes. There is no shared job graph, byte admission, queue priority, cancellation/deadline, progress, duplicate-request coalescing or source-generation fence.

Unreal's `InterchangeTextureFactory` separates game-thread begin/publication from asynchronous import and uses `ParallelFor` for tiled payload work. `TextureDerivedDataTask` constructs cacheable build definitions and policies, estimates required memory, launches through the texture compiler pool and carries owner priority/cancellation. `TextureCompiler` adjusts concurrency against memory constraints. Zircon must establish the same execution classes before optimizing filter loops.

### P0: source, recipe, physical platform product and resident asset are collapsed

The importer can decode bytes and then change descriptor dimension, mip count, layer count or format from settings without converting the physical payload to that layout. Tests permit a 2D RGBA source to describe a multi-layer 3D `rgba16float` product. Container settings can similarly change the logical descriptor while the preserved DDS/KTX/ASTC bytes retain their original block format.

This defers deterministic incompatibility until cache load or GPU upload after expensive work has completed. Sampler/logical authoring metadata, source interpretation, physical subresource layout and target cook format require separate versioned schemas. Publication must validate exact byte ranges, row/block geometry, layers/faces/depth/mips and device format support.

### P0: full texture payloads are repeatedly cloned across cache, load and residency

Container import falls back to `context.source_bytes.clone()` and stores the complete container in `TextureAsset`. Artifact cache conversion clones both `rgba` and container payload bytes. `ProjectAssetManager::load_texture_asset` then uses generic typed loading and `asset.as_ref().clone()`, deep-copying the asset again. Each texture residency rebuild synchronously reloads that complete CPU asset even when only a subset of mips is wanted.

Only uncompressed RGBA currently advertises physical mip streaming; compressed containers remain whole-payload resources. A mip transition reconstructs a GPU texture and copies common mips rather than reading and uploading independently keyed mip chunks. The required boundary is immutable payload leases plus platform artifacts chunked by independently loadable subresources, with no full-payload clone on cache hit, load, preview or residency transition.

### P0: image decode and KTX2 expansion have no aggregate memory admission

Standard images and PSD are fully decoded to resident RGBA8 with no explicit width, height, pixel, decoded-byte or expansion-ratio ceiling. HDR/EXR inputs also pass through `to_rgba8`, discarding radiance instead of selecting a float source product. KTX2 zstd/zlib validates each declared `uncompressedByteLength` and reserves that exact level size, but has no aggregate source/project/job memory budget while source and rewritten container coexist.

Admission must validate headers before allocation, reserve source/decoded/scratch/cooked bytes against shared budgets, and cancel safely when declared or produced bytes cross policy. HDR/EXR must retain an appropriate float representation and color metadata through target cook.

### P0: cubemap and array construction amplify memory before derived work begins

Manifest sources bypass a canonical source broker and use direct `std::fs::read`; multiple complete images are decoded before validation and assembly. Array slices allocate one vector per layer and then interleave them into a second full vector. Cubemap cross/equirect paths allocate six face vectors and then interleave them into another full cube payload. Equirectangular conversion is serial over six faces and performs four filtered fetches plus floating-point direction mapping for every output texel.

For a 16K x 8K RGBA source, the decoded source alone is **512 MiB**. The default 4096 cube face size produces six base faces totaling **384 MiB**, and the interleaved cube adds another **384 MiB**. Source + face temporaries + base cube therefore reaches a **1.25 GiB lower bound** before the complete mip chain, cache clones, allocator overhead or GPU staging. This is a byte-lifetime model, not measured RSS.

The target streams source tiles through a bounded broker, validates all dependency receipts first, partitions face/tile work through admitted jobs, and writes directly into an owned or chunked destination without six long-lived intermediate owners.

### P0: offline mip generation is serial, copy-heavy and not yet a production texture compiler

Decoded images default to offline Kaiser mip generation. Work is serial by layer and level. Each level allocates output, layer results collect into `Vec<Vec<u8>>`, and data is copied again into a packed chain. Large 2D RGBA mip chains approach **4/3 of base bytes** even before those temporaries. The Kaiser path precomputes weights, but sRGB encoding still evaluates nonlinear conversion per output channel and no shared compiler scheduling/admission exists.

Box/normal paths halve odd extents with floor division; a 3-to-1 reduction can ignore the final source row/column, unlike a coverage-correct kernel. Alpha coverage, premultiplication, address mode, normal renormalization and composite maps are not represented as one versioned production policy. Unreal's `TextureCompressorModule` is the applicable source reference: it centralizes filter policy, alpha coverage/composite/normal behavior and parallel image work. Zircon should retain scalar kernels as deterministic references, not make them the platform compiler boundary.

### P0: handwritten BC5 is an unsuitable production compression authority

The BC5 path is a serial min/max endpoint encoder that tests eight palette entries for 16 texels in each channel of every block. It has no platform quality tier, effort, perceptual/RDO policy, target tiling, mature encoder backend or shared cancellation. During transcode the complete RGBA mip chain and complete BC5 payload coexist, then payload bytes are copied into a container before RGBA is cleared.

Unreal's `TextureFormatOodle` provides the relevant architectural evidence: BC1-BC7/BC5 selection, quality/effort/RDO and tiling participate in the derived-data key; expensive work is cache-backed and accepts cancellation. Zircon should use a proven encoder backend for production target cook. The current implementation may remain only as a deterministic test/reference fallback until output quality and throughput are qualified against a corpus.

### P1: parser hardening is stronger than the product pipeline, but still lacks project budgets

DDS/KTX/ASTC validation is generally fail-closed and the KTX2 overlap check is quadratic only in a mip count bounded by texture extent, so it is not the primary bottleneck. The remaining structural risk is aggregate expansion and duplicated rewritten buffers, not small metadata loops or string allocations. BasisLZ may be preserved even when the selected device cannot upload it, leaving target transcode and fallback policy unresolved.

### P1: registration overclaims native readiness and tests measure fragments

Dist exports registration/lifecycle metadata with `invoke_command: None` and no native import bridge, while capability reports Stable source/library/native support. The 16 ignored tests benchmark local parsing, loops or descriptors, but there is no end-to-end corpus for decode, mip, compression, artifact cache, upload, residency, peak RSS, queue/main-thread time or power. Several manifest/cubemap tests use `std::env::temp_dir()`; any later run must set `TEMP` and `TMP` to an approved D/E/F path so validation leaves no C-drive artifacts.

## 3. Reference-engine constraints

Unreal is the primary reference and establishes the following non-negotiable boundaries:

1. `InterchangeImageWrapperTranslator` produces typed image payloads and preserves source format/HDR metadata rather than quantizing every source to RGBA8.
2. `InterchangeTextureFactory` separates game-thread object work from asynchronous payload/build work and partitions tiled imports.
3. `TextureDerivedDataTask` and `TextureCompiler` own deterministic build definitions, cache policy, required-memory admission, priority, cancellation and memory-adjusted worker concurrency.
4. `TextureCompressorModule` owns coherent mip/filter/alpha/normal behavior and parallel image processing.
5. `TextureFormatOodle` makes compression format, effort, quality, RDO and tiling part of target cook and cache identity.
6. `AsyncTextureStreaming` computes demand asynchronously and applies memory/residency budgets; it does not reload and clone the complete source asset for each mip transition.

The transferable architecture is `immutable source/dependencies -> versioned recipe -> admitted asynchronous build graph -> immutable platform artifacts/chunks -> budgeted GPU residency`. Zircon should not copy Unreal APIs, but must converge on those ownership and scheduling classes instead of tuning a temporary universal `TextureAsset`.

## 4. Dependency-ordered optimization plan

### M0: hard-cut provider authority and capability truth

Select exactly one TextureImporter provider per format in the first-party product. Remove or absorb the duplicate built-in image importer in the same migration. Generate source/library/native registration from one contribution bundle; until a native bridge executes equivalent work, capability must report it unavailable.

### M1: define source, recipe and physical platform artifact schemas

Separate immutable source/dependency receipts, authoring settings, logical texture identity, physical target layout and install metadata. Keys include source/dependency hashes, algorithm/version, color/alpha/normal policy, target/device format, compressor quality/effort/RDO, layer/face/depth/mip layout and packaging variant. Descriptor/payload disagreement must fail before publication.

### M2: introduce the source broker and aggregate byte admission

Resolve manifest dependencies through the canonical VFS/source authority with canonical path, hash, revision and sandbox receipts. Validate headers before allocation. Reserve source, decoded, scratch, derived, artifact and upload bytes under Runtime/Editor budgets; enforce dimensions, pixels, expansion ratio, layer/face count and aggregate project/job ceilings.

### M3: build a cancellation-aware texture job graph

Split probe/decode, orientation/color/normal transform, cube/array assembly, mip generation, compression, artifact encode and publication into dependency jobs. Carry recipe/source generation, target profile, priority, byte reservation, progress and cancellation. Coalesce identical keys and only publish the current generation while preserving last-good artifacts on failure.

### M4: replace temporary algorithms with a qualified texture compiler

Adopt a mature image resize/filter path with defined odd-extent, alpha coverage, premultiplication, normal and color-space semantics. Partition tiles/layers/faces on the shared pool. Integrate proven target encoders for BCn/ASTC/ETC and platform formats; keep handwritten mip/BC5 code as reference tests until corpus quality, determinism and throughput justify any production use.

### M5: eliminate full-payload ownership amplification

Build directly into immutable, content-addressed artifact chunks. Artifact serialization borrows/moves data or streams output. Typed loading returns leases/handles rather than deep clones. Array/cube assembly writes directly into final tiles/chunks, and editor preview/runtime install share artifact identities instead of rematerializing equal-size CPU payloads.

### M6: make upload and residency genuinely subresource-based

Cook independently loadable mip/layer/face chunks for the selected device format. Residency transitions read/upload only requested chunks, preserve shared GPU resources when possible and never synchronously load the complete CPU asset on main/render threads. Apply I/O, staging, VRAM and eviction budgets with generation-qualified requests.

### M7: instrument and dynamically qualify the current-source product

Emit phase wall/queue/CPU time, source/decoded/scratch/cooked/cache/upload/resident bytes, allocations/peak RSS, cache/coalescing result, cancellations, main/render/worker utilization and energy. Run PNG/JPEG/HDR/EXR, PSD, DDS/KTX2/ASTC, normal/data, cube/array and adversarial expansion fixtures at 4K/8K/16K. WPR/ETW owns CPU, I/O, scheduling, memory and power evidence. RenderDoc owns pixels, physical format/mips, resource lifetime, copy/upload counts and VRAM evidence only.

## 5. Acceptance gates

| Gate | Required evidence |
|---|---|
| A1 | Exactly one product-selected provider executes each advertised format; source/library/native capability matches executable behavior. |
| A2 | Source, recipe, physical artifact and install identities are separate and deterministic; descriptor/payload mismatch is rejected before cache publication. |
| A3 | HDR/EXR retain qualified float/color metadata; corrupt and expansion-bomb inputs terminate within declared dimension/byte/time budgets. |
| A4 | Decode/mip/compress/artifact jobs are admitted, cancellable and generation-bound; stable main/editor frames perform zero heavy texture work. |
| A5 | A built texture has no full-payload deep clone during cache hit, typed load, preview, upload or residency transition. |
| A6 | Cube/array memory is bounded by admitted tiles/chunks rather than source + per-face/layer owners + another complete assembled payload. |
| A7 | Odd-extent, alpha coverage, normal, color-space and composite-map fixtures match the versioned compiler policy; production compression uses a qualified backend. |
| A8 | Compressed and uncompressed residency read/upload only requested subresources and obey I/O, staging and VRAM budgets. |
| A9 | Fixed cold/warm/reimport/upload/stream fixtures report p50/p95/p99, throughput, queue depth, allocations, peak RSS, cache ratio, main/render/worker CPU and energy. |
| A10 | WPR and RenderDoc captures come from a launchable binary built from the reviewed current source; comparisons use matched input, output quality, cache state, hardware and power settings. |

## 6. Validation record

- Static primary-package coverage: complete, 41/41 Rust files; dependent catalog/cache/load/upload/residency chain reviewed to terminal ownership.
- Source snapshot: shared dirty work preserved; no production file was edited because the required fixes cross provider, asset schema, scheduler, artifact and residency owners.
- Formatting: 40/41 per-file checks pass; `runtime/src/lib.rs` has export-order formatting debt.
- Rust tests: not executed because the managed Windows validator identity is unavailable; no raw Cargo fallback was used.
- WPR/ETW/RenderDoc/power: pending until a launchable current-source product exists. RenderDoc will not be used to infer CPU importer performance.
- Direct optimization: none. A local loop or clone patch would entrench the wrong universal-asset boundary before provider/schema cutover.
- Protected ledgers, milestone commit and WeCom completion notice remain pending until dynamic acceptance evidence exists.
