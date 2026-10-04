---
title: Neural Model Format CPU Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/neural/runtime/src/model
  - zircon_plugins/neural/runtime/src/ops
  - zircon_plugins/neural/runtime/src/cpu
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Private/NNEModelData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntime.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntimeCPU.h
---

# Neural Model Format CPU Current-Source Algorithm Performance Review

## 1. Coverage and execution truth

The runtime model, operator, CPU and GPU-support core scope is **14/14 production Rust files**, **3,077 physical / 2,877 non-empty lines**, **105,901 bytes** and **1 inline test**. At repository revision `f811b3bf474d70347199772a175422333dfb36f6`, its ordered `workspace-relative path + NUL + raw bytes + NUL` SHA-256 is `64f13b0045687fa76589cbd9ebeb7a82ba38dfc345dc0cd976c35e064de6bc75`.

`runtime/src/model/format.rs` and `runtime/src/tests/model_asset.rs` contain concurrent edits that were treated as current source and preserved. The current loader now has explicit artifact/weight/op-table/count limits, minimum op-record proof and fallible reservation. This is a real improvement over the earlier canonical review, but it is external current-worktree state, not a change made by this audit. No Cargo execution or dynamic timing is claimed.

## 2. Structural performance findings

### P0: `run_cpu` rebuilds execution state and decodes every weight on every inference

`cpu/interpreter.rs:43-102` validates the complete model, creates a fresh `BTreeMap`, copies every caller input and calls `load_weights` on every invocation. `:131-175` scans all tensors and decodes every F32 weight blob into a new `Vec<f32>`. Every operator then allocates another output vector; intermediate tensors remain in the map until the complete run ends, and Reshape clones the full source tensor (`:787-799`).

For model bytes `W`, live intermediate bytes `L`, inputs `I` and runs `R`, repeated inference performs at least **O(R * (W + I + operator work))** preparation and can retain **O(W + I + all intermediates)** rather than a liveness-bounded workspace. The CPU implementation is useful as a correctness oracle, but it is not a reusable product runtime.

### P0: admission does not create a validated, backend-executable graph

`model/validate.rs` proves local references and byte ranges but not topology, unique producers, tensor-kind legality, operator arity, shape/broadcast rules, alias legality or a selected backend's executable contract. CPU and GPU therefore repeat different late checks. A public/programmatic or directly loaded `.znn` can bypass the stricter ONNX converter contract.

One concrete failure is GPU Reshape: `gpu/graph_executor.rs:187-204` aliases output to input without proving equal element counts. CPU and GPU Gemm/Conv also accept different attribute and bias subsets. This is a correctness blocker for performance work because a cheaper invalid execution path cannot be used as a benchmark baseline.

### P1: current `.znn` limits are necessary but are not resource policy

`model/format.rs:17-21` hard-codes up to 512 MiB artifact and weight blobs and over one million tensors/operators. `from_znn_bytes:118-206` now validates those limits, proves the op count against table bytes and uses `try_reserve_exact`, but still copies the complete weight blob. `to_znn_bytes:64-115` uses infallible capacity allocation and does not apply the same product limits.

These constants prevent the previous immediate count-driven allocation defect; they do not establish project/platform/provider budgets, mapped/shared immutable storage, artifact identity, cache admission, cancellation or load generation. A valid near-limit file can still cause several simultaneous whole-model copies across import, serialization, loading, CPU decode and undo history.

### P1: scalar kernels and tree maps are reference behavior, not the optimization target

The interpreter uses string-free numeric tensor IDs, but `BTreeMap<u16, Vec<f32>>` lookup, scalar nested Conv/Gemm loops and per-op allocations dominate before SIMD micro-tuning matters. Parallelizing those loops immediately would multiply memory pressure and compete with the engine scheduler without cancellation, affinity or a frame budget.

Keep this implementation deterministic as the numeric oracle. Product CPU inference should use a mature provider or a separately prepared backend with immutable shared weights, packed kernels, caller-owned bindings, reusable workspaces and scheduler receipts.

## 3. Unreal source constraints

- `NNERuntime.h:62-86` requires a runtime to prove it can create runtime-specific model data, create that data and provide a cache identifier including source/version/target identity.
- `NNEModelData.cpp:588-623` returns cached runtime-specific shared model data and only creates/inserts it on a miss. `:633-669` routes creation through the selected runtime's capability check.
- `NNERuntimeCPU.h:17-45` separates a model from caller-owned reusable CPU model instances. Weight sharing belongs to the model/provider lifetime; per-run bindings/workspace belong to the instance.

Zircon should adopt these lifetime properties, not Unreal object layout: source artifact -> validated/provider-specific shared model -> reusable instance -> bounded run bindings.

## 4. Dependency-ordered optimization plan

### M0: compile one `ValidatedNnGraph`

Make parse/load produce an immutable graph only after topology, producer/consumer, kind, dtype, arity, shape, broadcast, alias, backend capability and resource-budget checks. Both CPU and GPU must consume this graph; remove converter-only executable truth.

### M1: establish model and instance lifetimes

Add a provider-qualified shared model generation containing decoded/packed immutable weights and compiled operator metadata. Add caller-owned instances with prepared input shapes, liveness-derived workspace plans, reusable buffers, cancellation and generation checks.

### M2: keep CPU as an oracle and add a product provider deliberately

Retain deterministic scalar kernels for differential tests. Select a mature CPU backend or implement prepared kernels behind a provider boundary; do not silently transform the oracle into an unbounded worker pool. Declare supported operators, precision and dynamic-shape profiles per provider.

### M3: integrate resource authority

Load `.znn` asynchronously through Runtime64 with artifact digest, provider/target/build identity, byte and resident budgets, cancellation, last-good generation, reload swap and lease-based retirement. Avoid decoding/copying weights at every inference.

### M4: qualify memory and scale

Measure cold load, warm instance creation and steady inference separately across model bytes, operator count, live tensor bytes, batch and concurrent instances. Record validate/decode/prepare/run time, allocations/bytes, peak RSS, cache hit, scheduler wait, CPU, wakeups and power.

## 5. Acceptance gates

1. Steady inference performs zero model validation, weight decoding and whole-input ownership copies unless the API explicitly requests a copy.
2. Peak workspace follows liveness intervals rather than the sum of all intermediates.
3. CPU and GPU accept the same validated graph or return a provider-specific typed admission failure before execution.
4. Resource limits are project/provider policies and every allocation is fallible or accounted.
5. Reload/device/provider replacement rejects stale instances and retires old storage after in-flight completion.
6. Dynamic evidence distinguishes cold preparation from warm execution and includes power/RSS, not only elapsed microbenchmarks.

## 6. Validation status

- Static production review for the captured runtime core fingerprint: complete.
- Direct source optimization: intentionally deferred; the current bounded-loader edits are concurrently owned, while the next safe change is the validated-graph/model-instance boundary.
- Cargo/tests: pending because the managed Windows validation session is not executable; raw Cargo was not substituted.
- WPR/ETW/power: pending because no launchable current-source executable exists.
- Protected ledgers, milestone commit and WeCom completion remain pending.
