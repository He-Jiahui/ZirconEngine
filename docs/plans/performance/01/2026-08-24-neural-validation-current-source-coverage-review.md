---
title: Neural Validation Current-Source Coverage Review
date: 2026-08-24
scope:
  - zircon_plugins/neural/runtime/src/tests
  - zircon_plugins/neural/editor/src/tests.rs
status: static_complete_execution_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
---

# Neural Validation Current-Source Coverage Review

## 1. Coverage

The validation scope is **7/7 Rust files**, **1,579 physical / 1,456 non-empty lines**, **55,028 bytes** and **39 test attributes**. Ordered fingerprint: `cab772ec7b2e66f7cf81ff22c9e166d52d17ea0b2201aaf183c718d8cadb507d`.

Together with production, Neural is **36/36 Rust files**, **7,154 physical / 6,656 non-empty lines**, **245,520 bytes** and **43 test attributes**. Composite fingerprint: `ac2b8c8f59f859ecb7a4d3c078f629b8cd8a527d09867f3b8d92922e8c4af6a7`.

| Scope | Files | Test attributes | Static result |
|---|---:|---:|---|
| Production | 29 | 4 inline | Runtime core, Editor, feature, plugin and Dist fully reviewed. |
| Validation | 7 | 39 | Model, CPU, GPU, resource, manifest and Editor behavior reviewed. |
| Total | 36 | 43 | Every current Neural `.rs` file accounted for at the composite fingerprint. |

One inline release benchmark is ignored by default. It compares tensor-ID decode allocation inside the model-format file; it is not a full loader, model-instance, inference, Editor import or GPU benchmark.

## 2. Coverage gaps that block performance acceptance

1. No test proves a parsed `.znn` satisfies one shared topology/producer/arity/shape/alias/backend executable contract before CPU or GPU execution.
2. No differential suite runs all supported operators and edge cases on CPU and GPU with a declared tolerance/NaN policy.
3. GPU tests inspect descriptors/WGSL markers only; there is no real shader compilation, adapter/device execution, Render Graph submission, readback, timestamp or pixel oracle.
4. No test proves warm inference avoids model validation, weight decode, shader generation, pipeline creation and whole-workspace allocation.
5. Current `.znn` malformed tests cover the newly bounded count/size cases but not aggregate resource policy, serialization symmetry, mapped/shared weights, cancellation, reload or property/fuzz corpora.
6. ONNX tests lack file/field/string/node/tensor/dimension/raw/external-data budgets, invalid UTF-8 identity, negative/overflow dimensions, unknown opset/domain and cumulative peak-memory cases.
7. Editor tests exercise happy import/undo/path authority but not background-job affinity, cancellation, progress, same-volume staging, atomic replace or fault/crash recovery.
8. No product test imports/cooks/loads/instantiates/executes the exact generation through PIE/game; the runtime and post-process registration paths remain empty.
9. No static/native Dist parity, device-loss, hot reload, stale generation or in-flight retirement scenario exists.
10. No scale suite reports cold/warm p50/p95/p99, allocations, peak RSS/VRAM, scheduler wait, UI blocked time, GPU timestamps, wakeups or power.

## 3. Required validation matrix

Add counters before elapsed gates: source/artifact/weight bytes, nodes/tensors/operators, validated graph cache hits, prepared model/instance counts, decoded/uploaded bytes, live/workspace/resident bytes, shader/pipeline cache misses, dispatch/pass/barrier counts, job queue/wait/run/publish time, owner-thread blocked time, cancellation latency and stale-generation rejections.

Run small/medium/large and malformed source matrices, cold/warm loads, one/many instances, stable/dynamic shapes, full/three-quarter/half post-process scale, reload/device-loss and debug off/on. Report raw samples and p50/p95/p99 with CPU, RSS/VRAM, wakeups and power. RenderDoc is an acceptance tool only after a current-source executable submits the neural graph.

## 4. Execution status

- Static validation-file review: **7/7 complete**.
- Static all-Rust-file review: **36/36 complete** for the captured fingerprint.
- Test execution: pending; the managed Windows Cargo validation session is unavailable and raw Cargo was not used.
- Current-source WPR/ETW/RenderDoc/power: pending; no launchable current-source executable exists.
- No performance bottleneck is declared removed and no Unreal time/power parity is claimed from static evidence.
- This record does not promote protected `review.md`/`pending.md`, create a milestone commit or send WeCom completion.
