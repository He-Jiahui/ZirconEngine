---
title: Plugin Texture Current Source Review
date: 2026-08-24
scope:
  - zircon_plugins/texture
status: static_complete_product_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TextureDerivedDataTask.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/AsyncTextureStreaming.cpp
  - dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Private/TextureEditorToolkit.cpp
---

# Plugin Texture Current Source Review

## 1. Coverage

The current Rust surface is **12/12 files**, **529 physical / 470 non-empty lines**, **18,401 bytes**, and **7 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `a9c41d59011ed8aac92195b78b1e05129696d2fee2b24a2cf5b618aec96313c9`. The package directory is clean.

Per-file coverage is complete by module:

- Runtime: `capability.rs`, `lib.rs`, `manager.rs`, `module.rs`, `plugin.rs`, `tests.rs`.
- Editor: `capability.rs`, `extension_ids.rs`, `lib.rs`, `plugin.rs`, `tests.rs`.
- Dist: `lib.rs`.

The generated manifest, all three Cargo manifests, physical resource inventory, first-party Runtime and Editor catalogs, and the existing cross-layer Texture owner reports were also checked.

## 2. Primary finding

This package is not a Texture subsystem. Its only texture operation is `DefaultTextureManager::summarize_texture(width, height, mip_count)`, which clamps the mip count and returns the base-level `width * height`. It has no source bytes, decoded image, color space, typed format, layer/depth/cube identity, mip offsets, compression, build key, artifact, GPU upload, residency, streaming, eviction or diagnostics contract. The manager is lazy and the calculation is constant-time, so there is no credible hot-path optimization inside this package.

The returned `texel_count` is not total texture storage or work. A full square 2D mip chain approaches `4/3` of the base texel count before layers, faces, depth, block compression and row alignment. Correct accounting requires the actual subresource layout and format block geometry. Extending this summary function with another guessed formula would create a third, weaker Texture authority beside the real import/upload/streaming code; it is therefore intentionally not patched.

Despite that boundary, `plugin.toml` labels `runtime.plugin.texture` as `stable` and `complete`. The first-party Runtime catalog links the source package and publishes its registration. The Editor side registers one view/drawer/template against `plugins://texture/editor/authoring.zui`, but the package contains **zero physical assets**, so that resource edge is unresolved. The first-party Editor catalog does not link the package. The native dist exports metadata and an empty command/event manifest, with no command, state, unload, bridge or host-ready behavior; it cannot reconstruct the source Editor registration.

This is a product-truth and ownership defect rather than a local arithmetic bottleneck: the runtime can advertise completion for a shell, the Editor contribution is unreachable and references a missing resource, and source/dist behavior is not equivalent. No production code edit is justified before the Texture owner cutover.

## 3. Unreal source constraints

Unreal's Texture path is divided by durable data and execution boundaries rather than a generic manager summary:

1. `TextureDerivedDataTask.cpp` constructs build definitions and policies, resolves parent build metadata/data asynchronously, hashes source inputs, pipes dependent build outputs into child builds, uses cache-query/build policies, launches Texture builds through the texture compiler thread pool, and exposes priority, cancellation and wait controls. It also separates virtual-texture build output and validates it before DDC publication.
2. `AsyncTextureStreaming.cpp` computes wanted mips on asynchronous views, culls levels that cannot affect resolution, sorts candidates to enable early exit, stops walking once maximum resolution is reached, applies memory budgets and checks abort state while reducing resolution. This is demand, priority and budget logic, not a full-image synchronous reload loop.
3. `TextureEditorToolkit.cpp` owns channel masks, exposure, explicit mip/layer/slice/face state, volume/cubemap modes, persisted sampling and zoom behavior, and final-quality preview policy. A Texture editor is an artifact-aware toolkit, not one generic missing template.

The transferable structure is `versioned source/recipe -> deterministic build graph -> cacheable platform artifacts -> budgeted asynchronous residency -> artifact-backed editor toolkit`. Zircon should not copy Unreal APIs, but the package may not claim the Texture capability until it participates in that same class of end-to-end contract.

## 4. Dependency-ordered plan

### M0: fail closed and establish one owner

Downgrade or withdraw `runtime.plugin.texture` completion until product qualification exists. Make Plugins18 the package/product owner and Editor35 plus Runtime92 the shared Texture architecture owners. The current manager may remain only as an explicitly diagnostic metadata helper, or be removed during hard cutover; no consumer may interpret it as import, build, memory or residency truth.

Add static product gates for capability-to-provider closure, source/dist behavior parity, first-party catalog reachability and package-resource resolution. Missing `authoring.zui`, an empty dist behavior table, or a catalog-unreachable Editor provider must fail before readiness publication.

### M1: join the canonical Texture build graph

Replace the summary-only service contract with a narrow provider over the canonical Texture source, versioned import recipe and immutable artifact pipeline owned by Runtime85/Runtime92/Editor35. Identity must include source/dependency hashes, recipe and algorithm versions, target platform/device capabilities, format/encoder policy, mip/layer/face layout and packaging variant. The plugin must not invent parallel asset or cache types.

Build/decode/mip/compress work runs through bounded jobs with priority, cancellation, duplicate coalescing and generation-qualified publication. Cache hit, miss, rejected artifact and fallback decisions emit structured diagnostics. Ordinary Texture, cube/array/volume, render target, virtual texture and sampler identities remain typed and cannot be collapsed into width/height metadata.

### M2: source/dist and catalog closure

Generate one package contribution bundle for source and dist. Runtime registration must expose the same provider identity and qualified behavior in both forms. Editor registration must be linked by the selected product profile and reconstructable from dist, or the package must stop declaring Editor support. Mount/unmount revokes new work, cancels queued jobs, waits for active leases and releases artifacts/providers by generation.

### M3: real Texture authoring

Replace the unresolved template with the single transactional Texture toolkit owned by Editor35. It consumes the same recipe, compiler artifact and runtime install receipt. It must expose source/artifact comparison, format/color-space/compression truth, mip/layer/face/slice selection, channel/exposure/zoom controls, memory estimates, reimport, save/undo/redo and structured build diagnostics without creating a second document owner.

Stable UI frames perform no decode, mip generation, compression, full preview rebuild or manifest rematerialization. Source/recipe/artifact/device generations invalidate only affected projections, and background work publishes last-good preview state.

### M4: performance qualification

Use representative PNG/JPEG/HDR/EXR, DDS/KTX2/ASTC, normal/data/mask, cube/array/volume and virtual-texture corpora. Measure `1/1,000/100,000` catalog records and 4K/8K/16K sources across cold/warm build, cache hit/miss, edit/reimport, runtime install, stream-in/out and Editor preview. Record p50/p95/p99 wall and queue time, decoded/built/uploaded bytes, cache hit ratio, duplicate jobs, cancellations, main/render/worker CPU, allocations, RSS, VRAM and energy.

WPR/ETW must prove bounded main/render-thread publication and worker utilization. RenderDoc is used only after a current-source executable exists, to verify uploaded format/mips, resource lifetime, draw/copy counts and pixels. Compare against Unreal only with matched source, output format, quality, mip policy, cache state, viewport, hardware and power settings.

## 5. Acceptance

1. Capability readiness is impossible without a concrete provider, resolved resources, source/dist parity and catalog reachability; `complete` is backed by executable product evidence.
2. One canonical Texture source/recipe/build/artifact/install identity is shared by runtime, Editor and plugin packages. No summary-only or importer-specific parallel authority survives.
3. Artifact keys cover every input that can change bytes or layout. Duplicate requests coalesce, cancellation is cooperative, stale generations cannot publish and cache corruption fails closed.
4. Runtime starts from an explicit mip tail and obeys I/O, upload and residency budgets. Main/render threads do bounded handoff only; no synchronous decode/build/full-artifact reconstruction occurs there.
5. Editor source and dist forms open the same transactional Texture toolkit, backed by the same artifact. A stable frame performs zero decode/build/preview rebuild work.
6. Scale tests publish cold/warm p50/p95/p99, throughput, queue depth, cache ratio, allocations, RSS, VRAM and energy, with regression budgets checked in CI.
7. WPR and RenderDoc captures come from a launchable executable built from the reviewed source fingerprint. Reference architecture evidence is never reported as measured parity.

## 6. Validation status

- Static per-Rust-file review: **12/12 complete**.
- Package resources: **0 present / 1 referenced**, product gate failed statically.
- First-party catalog closure: Runtime source **linked**; Editor source **not linked**; dist behavior parity **failed statically**.
- `rustfmt --check`: **pass** for all 12 Rust files.
- Cargo/test execution: **pending** because the managed Windows validation session is not executable; no raw Cargo lane was substituted.
- Current-source executable, WPR/ETW, RenderDoc, workload and energy qualification: **pending**.
- No production source was changed: the defect is architectural and the proposed arithmetic-only edit would entrench the wrong owner.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
