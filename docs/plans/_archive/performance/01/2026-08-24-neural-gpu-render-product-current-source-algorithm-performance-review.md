---
title: Neural GPU Render Product Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/neural/runtime/src/gpu
  - zircon_plugins/neural/runtime/src/plugin.rs
  - zircon_plugins/neural/features/post_process/runtime
canonical_owners:
  - docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
status: static_complete_dynamic_pending
references:
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntimeRDG.h
  - dev/UnrealEngine/Engine/Plugins/NNE/NNEDenoiser/Source/NNEDenoiser/Private/NNEDenoiserModelInstanceRDG.cpp
  - dev/UnrealEngine/Engine/Plugins/NNE/NNERuntimeORT/Source/NNERuntimeORT/Private/NNERuntimeORTModel.cpp
---

# Neural GPU Render Product Current-Source Algorithm Performance Review

## 1. Current product truth

GPU sources were reviewed inside the 14-file runtime-core fingerprint `64f13b0045687fa76589cbd9ebeb7a82ba38dfc345dc0cd976c35e064de6bc75`; runtime/plugin, feature and distribution assembly are part of the separate **7-file / 431-line / 15,483-byte** production group with fingerprint `101fb2c137f612da6b4907c323caeda23577ad5676362cc0fcca489c2de5facb`.

`NnGraphExecutor` currently produces descriptors. There is no package-external production consumer for `NnGraphExecutor`, `NnModelAsset` or `NnPostProcessSettings`; `NeuralRuntimePlugin::register` is empty, and the post-process registration test explicitly requires all render-feature, pass-executor, scene-hook and provider collections to remain empty. This is not executable GPU inference and cannot be profiled with RenderDoc.

## 2. Structural performance findings

### P0: each plan build reconstructs a string-heavy graph instead of reusing a compiled instance

`gpu/graph_executor.rs:94-184` validates the model again, creates tree maps of string resource aliases, formats resource/pass names, allocates binding and parameter vectors and attaches owned inline WGSL to every pass. `shader_templates.rs:286-328` clones static shader bodies; unary, binary and pooling operators perform one or more full-string replacements per operator.

If a consumer builds this per frame, preparation is **O(operators * shader bytes + graph metadata)** plus allocations before any GPU work. If built once, the missing artifact/pipeline/device generation still prevents safe cache reuse. The solution is a provider-compiled plan keyed by validated graph, shape profile, shader artifact, device and pipeline generation, not isolated string micro-optimization.

### P0: descriptor planning stops before every GPU ownership boundary

The output contains pass names, inline source, bindings and dispatch dimensions, but no shader compilation/reflection, pipeline/PSO cache, persistent weight upload, transient liveness/alias plan, Render Graph registration, queue admission, submission, completion, device-loss handling or output binding. `gpu/weight_upload.rs` validates again, builds a tree map and clones all weight bytes.

One pass per operator is emitted without fusion, constant folding beyond Reshape alias, layout propagation or liveness-based reuse. The advertised async-compute/post-process stage is not backed by queue capability or a dependency/cost decision.

### P0: the optional post-process is settings-only while copying models by value

`NnPostProcessSettings` stores `Option<NnModelAsset>` by value, so settings clones can duplicate graph and weight storage. Yet the feature registers no render owner, inputs, outputs or execution path. Before enabling it, the setting must hold a stable resource handle and revision; a render feature must acquire a prepared RDG instance and bind scene textures/history through the actual graph.

### P1: there is no truthful performance baseline

The current tests inspect descriptor fields and WGSL markers. They do not compile WGSL, create a device, upload weights, execute a graph, read back numeric output, compare CPU/GPU values or render a post-process image. RenderDoc would have no launchable neural frame to capture, and CPU plan-building timings would not answer GPU bottlenecks.

## 3. Unreal source constraints

- `NNERuntimeRDG.h:22-105` makes input shape preparation explicit and binds caller-owned input/output buffers when enqueueing work into an existing `FRDGBuilder`; `:113-169` separates reusable RDG models and instances from model data.
- `NNEDenoiserModelInstanceRDG.cpp:23-54` resolves a named runtime, checks capability, creates a model and then creates one reusable model instance. `:88-95` delegates shape preparation and frame enqueue to that instance.
- `NNERuntimeORTModel.cpp:1020-1160` documents render/RHI/submission-thread ownership, validates bindings, allocates RDG pass parameters, adds an instrumented graph pass and submits through the graphics queue. This is concrete execution, synchronization and telemetry, not a list of descriptors.

Zircon should match the ownership model: provider-selected compiled model -> prepared device instance -> caller-bound graph resources -> scheduled pass -> completion receipt. It should not copy DirectML-specific implementation details.

## 4. Dependency-ordered optimization plan

### M0: make capability truthful

Keep neural runtime/post-process unavailable or explicitly descriptor-only until an asset loader, backend provider, render feature/pass owner and completion path exist. Remove tests that treat empty extensions as product success.

### M1: compile a provider/device-qualified plan once

Lower `ValidatedNnGraph` to stable numeric resources, typed parameter layouts, compiled shader artifacts/reflection, pipeline keys, fusion groups and liveness intervals. Cache by model revision, shape profile, backend, precision and device generation.

### M2: own residency and instances

Upload immutable weights once per resident generation. Give each model instance reusable workspace bindings and shape preparation. Integrate budgets, eviction, reload and device-loss retirement with Runtime09a/09d.

### M3: integrate with the real Render Graph

Register input/output scene resources, dependency and queue requirements through the runtime render graph; let the graph choose legal queue placement and transient aliases. Return typed enqueue/completion/stale/cancel receipts.

### M4: build the post-process product path

Use resource handles rather than model values; define color space, extent, scale, history validity, fallback and disable behavior. Add Editor controls only after the exact runtime generation can execute in PIE/game.

### M5: qualify GPU behavior

Capture compiled/current-source frames at full, three-quarter and half scale. Report per-stage GPU timestamps, dispatches, barriers, queue overlap, transient/resident bytes, pipeline misses, upload bytes, frame p50/p95/p99, CPU submission, RSS/VRAM and power. Compare output pixels and numeric tolerances before comparing cost.

## 5. Acceptance gates

1. Warm frames do not regenerate WGSL, names, parameter layouts, pipelines or weight uploads.
2. Intermediate memory is liveness planned and bounded; queue choice is dependency/capability driven.
3. Post-process registration owns a real render feature/pass path or remains unavailable.
4. CPU/GPU numeric parity and post-process pixel oracles pass before performance claims.
5. Device loss, reload, cancellation and stale generations cannot reuse invalid resources.
6. RenderDoc and GPU timestamps come from a launchable current-source product executable.

## 6. Validation status

- Static GPU/product review: complete for the captured source fingerprints.
- Package-external production consumers found: **0** for the reviewed neural runtime types.
- Direct source optimization: deferred because the safe unit of change is the provider/model-instance/render-graph boundary.
- Cargo, current-source executable, RenderDoc, GPU timestamp and power evidence: pending.
- No GPU bottleneck is declared removed, and no Unreal performance parity is claimed.
