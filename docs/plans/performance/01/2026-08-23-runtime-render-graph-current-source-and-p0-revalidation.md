---
title: Runtime Render Graph Current Source and P0 Revalidation
date: 2026-08-23
scope:
  - zircon_runtime/src/render_graph
status: static_complete_dynamic_pending
canonical_owner:
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
related_owners:
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphResourcePool.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphPass.cpp
---

# Runtime Render Graph Current Source and P0 Revalidation

## 1. Current-source coverage

The current `zircon_runtime/src/render_graph` surface is **18/18 Rust files**, **6,319 physical / 5,825 non-empty lines**, **222,336 bytes**, and **62 test markers**. The workspace-relative `path + NUL + raw bytes + NUL` SHA-256 is `78e1148393805632e09be25d3485126f68c22406f6243c5b05d575ff24bb1b2e`.

All production and focused test files were read directly: builder/compile, graph/types/error, dump/store lint, handle/cycle/ordering/version/culling/external alias/transient alias/compute/scale tests. Product call sites were then followed through pipeline authoring, compiled-graph cache, transient materialization/validation and stage execution. Existing formatting-only foreign edits in `render_graph/error.rs` and `render_graph/graph.rs` are preserved and are not claimed by this report.

Runtime89 remains the canonical architectural owner. Its baseline was `be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`; this revalidation uses the current shared tree at HEAD `471bb732e3683fd7c12d7b69a9e85a22048efcba`. Older performance reports that describe bare handles, an unconditional pass chain, or name-only stage execution are historical and must not override Runtime89 or this currentness record.

## 2. Accepted current reductions

The following current design elements are valid foundations and must be retained:

- builder-generation-scoped pass/resource handles reject foreign handles;
- logical resource versions and separate RAW/WAW/WAR execution dependencies avoid a manually serialized total chain;
- culling provenance is distinct from physical execution hazards, so a clear/discard write can remove obsolete producers;
- graph statistics, dump data and exact-descriptor transient interval slots are compiled once rather than reconstructed by each consumer;
- sparse reservations are excluded from dense physical allocation statistics;
- readback extends a live resource interval so a transient slot cannot be reused before the readback pass;
- CPU command recording may use topology layers for passes that explicitly permit parallel recording.

Runtime89 `RG89-P0-001` is now statically implemented. Final pass names are validated for uniqueness, every `CompiledPassStage` stores its builder-issued `RenderPassId`, generated IBL stages retain those IDs, and stage execution indexes the compiled pass by ID before defensively checking the name. This removes the prior linear name lookup and duplicate-name wrong-pass execution path. The release-scale benchmark is still ignored and no current product executable was run, so this item is **implemented but dynamically unaccepted**.

## 3. Revalidated P0 contracts

### P0-A: sparse reservation can report complete without an executable backing

The core compiler accepts `SparseReserved` textures and reports a sparse slot. Product materialization deliberately skips those lifetimes. Validation increments `sparse_texture_reservation_count` without incrementing required or missing resources, so `materialized_resources_complete()` can still return true. A later pass nevertheless resolves a texture/view by logical resource name. Both the RHI capability default and the WGPU backend state that sparse textures are unsupported.

This is not an allocation-loop micro-optimization. The graph currently has no device-qualified disposition connecting compile, materialization and execution. Until a real sparse provider exists, final compilation must either emit a typed dense fallback or fail with `UnsupportedSparseResidency`; a live access without a resolvable physical binding must make materialization incomplete.

### P0-B: plugin storage textures can be inferred as a format that cannot be used for storage

Plugin graph-resource aggregation retains name, broad kind and read/write intent but not format, extent class, dimension, mips/layers/samples or required usage. An unknown written texture is inferred as `Rgba8UnormSrgb` while the logical descriptor requests storage usage. WGPU materialization silently omits `STORAGE_BINDING` for unsupported formats; generic compute rejects the resulting format only when constructing the execution pipeline.

Thus feature compile, graph compile, cache lookup and materialization may all succeed for a contract that cannot execute. Unknown names must not select formats. A typed `RenderResourceSchema` must be resolved during final compilation against device format features, and materialization must fail rather than remove a required usage bit.

## 4. Structural performance findings

1. Access identity is whole-resource. Although logical versions exist, identical pass/resource/access rows collapse in a name-oriented access index and there is no first-class mip/layer/plane range. This forces false dependencies, prevents independent subresource lifetimes and makes correct barrier planning impossible.
2. `QueueLane` remains descriptive metadata. The product records command buffers in CPU topology order but submits through one WGPU queue; no compiler-owned cross-lane fork/join/fence schedule exists.
3. Transient slots are logical planning artifacts, not GPU heap/placed-resource allocations. There is no fence-qualified alias acquire/discard plan or allocation lifetime receipt. Sparse virtual-byte statistics therefore do not prove committed-memory savings.
4. Barrier/state transitions are not a compiled artifact consumed end to end by the backend. Product stage dispatch and hard-coded resource resolution still retain execution authority outside the graph packet.
5. `compiled_graph_cache` has a fixed capacity of 16 and includes exact view/render dimensions in its key. Resize or dynamic-resolution churn can synchronously compile misses and thrash before reuse policy is measured.
6. Store lint performs pairwise pass/access comparison when requested. It must remain an explicit diagnostic pass, not enter per-frame stats or ordinary execution.
7. The ignored release-scale test leaves compile complexity, allocation count and direct-ID execution cost unqualified at product scale.

These findings explain why optimizing individual hash probes or string allocations first would be misleading. The dominant target is one device-qualified immutable execution packet that owns access ranges, hazards, queue synchronization, physical lifetime, barriers and executor identity.

## 5. Unreal source constraints

Unreal RDG is the primary reference because it closes the same boundaries rather than only presenting a graph API:

- `RenderGraphResources.h` gives textures a subresource layout/range and tracks whole, first and last state per subresource; SRV/UAV views derive explicit subresource ranges.
- `RenderGraphBuilder.cpp` derives graphics/async-compute access, adds cross-pipeline producer dependencies, and constructs async-compute fork/join overlap regions. Resource lifetimes are extended over the full parallel region because allocation or release is unsafe while either pipe can still use the resource.
- the builder compiles prologue/epilogue barriers and cross-pipeline transitions from the same dependency/resource state rather than asking a later stage dispatcher to rediscover them.
- `RenderGraphResourcePool.cpp` qualifies reuse and deallocation with transient allocation fences, pending-deallocation state and last-used frame retention. Logical interval non-overlap alone is insufficient evidence that a physical object is reusable.

Zircon should adopt these ownership and invariant boundaries, not Unreal's C++ types, macros or global lifetime model. Unity/Godot remain secondary evidence for versioned handles and subresource barrier graphs; they do not weaken the Unreal-primary target.

## 6. Dependency-ordered optimization plan

### M0: close current compile-to-execute correctness

Retain stable pass IDs and add non-ignored duplicate-name/direct-ID gates. Fail closed for unsupported sparse residency and unknown storage schemas. Materialization completeness must mean every live compiled access has a compatible physical binding. No fallback may be inferred from a debug name.

### M1: typed resource and subresource schema

Introduce device-independent resource schemas for format class, extent policy, dimension, samples, mip/layer/plane ranges and required usages. Convert each pass access to a typed view/range and version. Validate plugin declarations before caching or graph construction.

### M2: compiler-owned hazard and culling IR

Build per-subresource producer/reader state and compile RAW/WAW/WAR edges, culling provenance and lifetime endpoints in one linear/indexed pass. Preserve deterministic output. Remove name-based access identity from correctness paths and expose counts for visited accesses, emitted edges and retained versions.

### M3: queue scheduling and overlap regions

Compile graphics/compute/copy lanes, cross-lane dependencies, fork/join regions and synchronization points from the same IR. Extend lifetimes across overlap regions. Unsupported device lanes must have one explicit fallback result; metadata-only lane declarations are not accepted.

### M4: physical allocation and sparse residency

Translate live ranges into device-qualified physical allocation requests. Dense transient resources need alias acquire/discard and GPU-completion fences. Sparse resources require a page-table/heap/bind provider lease; otherwise compile selects a declared dense fallback or rejects the graph. Report reserved, committed, peak and reused bytes separately.

### M5: barrier plan and backend consumption

Compile prologue/pass/epilogue transitions at subresource granularity, including cross-lane ownership and UAV ordering. The RHI/backend validates and consumes that artifact without silently weakening usage. Runtime09A remains owner of native barrier, queue and completion implementation.

### M6: single immutable execution packet

Unify pass ID, executor payload, stage/lane, access bindings, barriers, allocation operations and diagnostics into one generation-scoped packet. Product execution iterates this packet; hard-coded stage vectors and string resource lookup cease to be an independent authority.

### M7: cache, diagnostics and transactions

Key caches by normalized structural schema plus explicit size-class policy, record hit/miss/compile/eviction reasons, and move compilation off the frame-critical path when possible. Graph replacement is transactional: old executable generation remains active until new compile, materialization and pipeline readiness succeed.

### M8: product and dynamic qualification

Exercise runtime, editor viewport, plugin render features, resize/dynamic resolution, shader reload, device capability differences and failure injection through the same packet. Promote only with current-source receipts and visible-output parity.

## 7. Quantified acceptance matrix

1. Core compiler: `1/32/256/1024/4096` passes, `1/8/64` accesses per pass, duplicate/foreign/unknown handles, same and disjoint mip/layer ranges, clear/load/store/discard, culling roots and cycles. Record compile p50/p95/p99/max, allocations/bytes, visited accesses, emitted edges, retained passes and peak working set. For sparse/typed-storage negative cases, invalid graphs must fail before cache insertion and physical creation.
2. Scheduling: graphics-only, compute-only and mixed graphs with `0/1/8/64` overlap regions. Assert exact fork/join and fence count, no undeclared cross-lane access, deterministic packet hash, and lifetime extension over every overlap region.
3. Allocation: stable, alias-heavy, sparse/fallback and readback graphs over 10,000 frames. Record reserved/committed/peak/reused bytes, physical creates, pool hits, fence-wait age and releases. No physical reuse occurs before the completion receipt that dominates all users.
4. Product churn: fixed size, continuous resize, `50-100%` dynamic-resolution sweep, plugin add/remove and shader reload. Record cache hit rate, sync compile time on frame thread, frame p95/p99, packet generations, GPU memory and RSS.
5. WPR/ETW: on one launchable current-source Windows product, capture CPU samples, allocations, locks/waits, context switches, I/O, RSS and energy for idle viewport, graph rebuild storm and representative scene. Main-thread graph compilation and waiting must be separately attributable.
6. RenderDoc: on that same executable, verify expected pixels, pass/event order, subresource bindings, resource alias backing and barrier/queue evidence where exposed. RenderDoc proves render correctness/GPU structure only; it does not substitute for CPU or power measurements.
7. Reference comparison: compare asymptotic work and ownership invariants with the frozen Unreal RDG slices. Absolute time/power is reported only for matched hardware, driver, resolution, scene, build profile and sampling window; no unsupported claim of parity is allowed.

## 8. Current result

- Static current-source review is complete for **18/18** files in `zircon_runtime/src/render_graph`.
- Stable pass-ID execution closes the old wrong-pass P0 statically but remains dynamically unaccepted.
- Sparse residency and plugin storage schema remain executable-contract P0s; subresource, queue, barrier and physical allocation ownership remain structural work.
- No production source was changed by this pass because the next correct slice crosses compiler, product materialization and RHI contracts and is not a safe isolated edit.
- Rust tests, current-source Cargo, product launch, WPR/ETW and RenderDoc were not run in this pass; the module remains pending for dynamic acceptance, milestone commit and WeCom notification.
