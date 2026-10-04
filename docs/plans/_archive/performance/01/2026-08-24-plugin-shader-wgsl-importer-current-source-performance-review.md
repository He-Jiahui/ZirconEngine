---
title: Plugin Shader WGSL Importer Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/shader_wgsl_importer
status: static_complete_dynamic_pending
canonical_owner:
  - docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderCompilerCore.h
---

# Plugin Shader WGSL Importer Current Source Performance Review

## 1. Coverage and currentness

The current `zircon_plugins/shader_wgsl_importer` Rust surface is **4/4 files**, **811 physical / 737 non-empty lines**, **30,430 bytes**, and **13 test markers**. The workspace-relative `path + LF + decoded text + LF` SHA-256 is `cd8f75a7cbdb9d4c61dcf0cd7127612da97b9c3f75a07c90e9198be847f2d19f`. Both Cargo manifests, generated `plugin.toml`, runtime registration and the direct Runtime import call path were also read.

`runtime/src/lib.rs` and `runtime/src/plugin.rs` are modified in the shared worktree. This review adopts them as input and does not rewrite them. The meaningful current change moves Naga entry-point names instead of cloning them and replaces `Debug + lowercase` stage conversion with a static mapping. The added ignored gates attempt to quantify those two projections. They are valid local allocation reductions, but they do not change the module-level compile architecture.

## 2. Measured source facts

`import_wgsl()` borrows the input bytes as UTF-8, synchronously runs `naga::front::wgsl::parse_str`, synchronously runs a validator with all flags and all capabilities, projects entry points, then owns the source and stores two full strings through `source: source.clone()` and `wgsl_source: source`. Work is therefore at least `O(source bytes + Naga parse/validation + entry points)`, with two retained source payloads for WGSL.

The Runtime contract is synchronous: `AssetImporterHandler::import()` returns the full outcome, `FunctionAssetImporter` calls the function directly, and `AssetImporter::import_context()` calls the selected handler inline. Full project generation iterates sources and calls `import_context()` inside the loop. It has a special parallel executor for environment IBL staging, but no corresponding shader compile operation, queue, cancellation, cache lookup or result time slice. A caller on the editor/main thread can therefore pay WGSL parse and validation latency directly; multiple edits can queue repeated full frontend work.

The dist exports metadata and a registration manifest but exposes no command, bridge method, state or unload function. Plugin structure conformance cannot prove that the standalone native product can import a shader.

## 3. Structural bottlenecks

1. **Synchronous frontend execution:** source read, parse and validation are one blocking call with no operation receipt, priority, cancellation or bounded completion drain.
2. **No compiler-job identity:** the importer has no content/import/define/compiler/target key, in-flight coalescing or cache hit path. Repeated and duplicate requests repeat Naga work.
3. **Duplicate retained source:** raw WGSL is retained twice in one `ShaderAsset`; this scales with aggregate source size and every live generation.
4. **Target-blind validation:** `Capabilities::all()` is not derived from backend/device/cook target, so successful import can still defer failure to pipeline creation.
5. **Artifact gap:** the result has empty reflection, resource, pipeline layout and target artifact state yet can flow as an imported Shader asset. Parse success is being used where artifact qualification is required.
6. **Overlapping authority:** core, the legacy shader-family importer and this package can own overlapping Naga paths. Optimizing one copy before the canonical owner hard cut produces drift and duplicate cache domains.
7. **Native product mismatch:** NativeDynamic is advertised without native import behavior. Loading it cannot move frontend work off the host thread or provide a compiler worker.

The two current string/allocation changes are retained as useful leaf improvements, but they do not close any item above. No further source edit is appropriate until the unique owner and job/artifact contracts are settled.

## 4. Unreal source constraints

Unreal's `FShaderCompilingManager` computes worker concurrency from available cores and memory pressure, submits keyed jobs to a synchronized queue, runs local or distributed ShaderCompileWorkers, tracks pending/outstanding work, and drains results through a game-thread time budget. `ShaderCompilerJobCache.cpp` hashes compiler inputs, coalesces duplicate in-flight jobs, supports asynchronous DDC queries, bounds job-cache memory and exports cache attempt/hit/byte counters.

The transferable design is not Unreal's C++ type volume. Zircon needs the same separation of responsibilities: cheap submission, canonical job identity, bounded worker concurrency, duplicate coalescing, persistent derived-data lookup, time-sliced publication, target-qualified artifacts and observable queue/cache/memory state. Naga can remain the frontend implementation behind that contract.

## 5. Dependency-ordered plan

### M0: one truthful owner

Adopt Plugins05's hard cut: choose one source-frontend owner, remove parallel core/legacy/new ownership, make required capabilities participate in admission, and fail selected-but-unlinked providers explicitly. Withdraw NativeDynamic for this importer until a native bridge performs import work. Add failing product-level tests before changing parser code.

### M1: compiler operation and identity

Introduce a `ShaderCompileRequest` keyed by canonical source graph digest, imports, definitions, importer/Naga version, target backend/platform/device tier and compile flags. Submission returns generation/progress/cancel/terminal receipts. Coalesce identical queued/in-flight keys and look up shared DDC before scheduling. Parse/validate workers use Runtime task admission with concurrency and memory budgets; the editor/frame thread only submits and drains bounded completions.

### M2: validated IR, reflection and target artifact

Produce a versioned intermediate artifact containing entry-point kind, required capabilities, bindings/resources, interface/reflection and structured diagnostics. Compile or translate that artifact for explicit targets. A shipping Runtime consumes qualified artifacts, not source frontend state. Replace duplicate `source`/`wgsl_source` ownership with one immutable source blob/document handle plus artifact references.

### M3: editor, cook and hot-reload behavior

Track dependency closure and invalidate only affected keys. Editor preview keeps the last-good target artifact while a new generation compiles. Cook blocks only at explicit barriers and publishes artifacts atomically. Runtime hot reload retires old pipelines after frame/fence ownership clears; it does not synchronously parse source during a frame.

### M4: observability and comparison

Expose queue depth/wait, active workers, parse/validate/target time, input/artifact bytes, cache hit/miss/coalesced counts, cancellation latency, completion-drain time and peak memory. Compare Zircon and Unreal on matched hardware, shaders, targets, warm/cold cache state and build mode; report absolute numbers and work scale rather than claiming parity from source shape.

## 6. Acceptance gates

1. Unchanged re-import performs **zero** Naga parses; `N` identical concurrent requests produce **one** compile job and `N` terminal receipts.
2. The editor/main thread spends zero time parsing/validating and uses a configurable completion budget, default **<= 1 ms P95 per frame** under the agreed corpus.
3. WGSL source is retained once per immutable content digest; retained source memory is `O(unique live source bytes)`, not twice that value per asset generation.
4. Dependency invalidation recompiles only the transitive affected closure. Record exact requested/coalesced/cache-hit/compiled counts.
5. Run cold/warm `1/4/16` worker workloads across small/medium/large shader corpora. Capture wall/CPU p50/p95/p99, throughput, allocations, peak RSS, queue wait, cancellation and cache bytes.
6. Validate target capability rejection, malformed/large input limits, provider missing/unload/reload, stale generation suppression and last-good rollback.
7. WPR/ETW must show parsing on admitted workers and bounded main-thread finalization. RenderDoc begins only after a current-source executable can render the resulting target artifact; capture pipeline creation, shader binds, draw/GPU timing and frame output.

## 7. Validation status

- Static per-Rust-file review: **4/4 complete**.
- Plugin structure audit: **pass** for manifest/schema/registration/dist-boundary conformance; it does not validate import execution.
- `rustfmt --check`: **fail** on the two shared modified runtime files; no formatting edit was made because those changes are not owned by this review.
- Cargo tests, ignored release performance gates, current-source executable, WPR/ETW, RenderDoc, power and cross-engine measurements: **pending**.
- This module is not eligible for `review.md` acceptance, milestone commit or WeCom completion notification.
