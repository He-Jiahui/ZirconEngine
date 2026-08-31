---
title: Plugins02 Stack-Bounded ONNX Shape Admission
category: zircon_plugins
report_id: Plugins02-stack-bounded-shape-admission-2026-08-26
date: 2026-08-31
session_id: root-plugins02-stack-shape-release-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins02 stack-bounded ONNX shape admission

## Scope

- Parent scope: Plugins02 neural Editor ONNX conversion, specifically V1 CPU/GPU executable shape admission.
- Baseline: `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, epoch `575`.
- Owned paths: `onnx/executable_contract.rs`, its focused source contract, and this record.
- This slice reduces one allocation per admitted node and contributes to `P1-30`. It preserves all existing tensor-count, operator arity, shape, output, and diagnostic checks. It does not claim to close parser budgets, compiled IR, runtime model instances, GPU execution, post-process integration, or product qualification.

## Change

V1 executable operators require at most five inputs, with BatchNorm defining the maximum. Shape admission now resolves those borrowed tensor-shape references into a five-slot stack array and passes the populated slice to the existing operator checks. The previous `collect::<Option<Vec<_>>>()` allocated a heap vector for every valid node before any shape comparison.

Nodes above the V1 input bound fail through the existing executable-contract diagnostic. Missing tensor metadata still uses the existing missing-tensor diagnostic, and valid operators consume the same ordered shape slices. The direct Rust contract confirms a valid ReLU graph remains admitted.

## TDD and local evidence

- Historical RED: the first source contract failed 4/4 because the old source had no input bound or stack array, retained the heap collector, and had no direct Rust contract.
- Current RED: extending the contract for real benchmark evidence produced 3 expected failures while the original 4 contracts remained green.
- Current GREEN: `python -m unittest tools.tests.test_plugins02_stack_bounded_shape_admission_performance_contract -v` passes 7/7.
- `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` passes for `executable_contract.rs`.
- Scoped `git diff --check` passes.
- The crate-local ignored Rust benchmark now exercises the legacy heap collector and the stack-bounded collector against the same `OnnxNode`/`OnnxGraph` tensor lookups. It uses 4 warmup pairs and 21 alternating sample pairs over 65,536 admissions with one to five inputs.
- Allocation counting is gated to the benchmark operation. Raw legacy/stack nanosecond arrays, checksum parity, P50/P95, and allocation counts are emitted in one machine-readable `PERF_RESULT` line.

The following numbers are the historical 2026-08-26 standalone preflight model, not the final managed crate result. The final record and WeCom message must replace them with output from the crate-local Rust benchmark in the immutable validation copy.

| Metric | Heap shape-reference vector | Stack-bounded references | Change |
|---|---:|---:|---:|
| P50 | 6.2604 ms | 1.1861 ms | -81.0539% |
| P95 | 9.5990 ms | 2.0683 ms | -78.4530% |
| allocations / admission sweep | 65,536 | 0 | -100.000000% |

The other three runs produced P50 reductions of 82.1814%, 80.4971%, and 82.6063%, and P95 reductions of 80.0061%, 48.1522%, and 90.0427%, with identical allocation counts and checksum. These values cover CPU ONNX V1 shape-reference admission only and do not claim model import, inference, GPU, or frame-time improvement.

## Async validation

Validation request ID: `089f0038287642e197041152c0ffb2cc`.

The seven focused source contracts pass before the immutable snapshot. The coordinator runs the crate-local Rust behavior and ignored performance tests together under one release-mode filter. Acceptance requires 7/7 source contracts, both filtered Rust tests passing, checksum parity in all 21 sample pairs, P50 reduction of at least 70%, P95 reduction of at least 40%, exactly 65,536 legacy allocations per sample, and zero stack-bounded allocations.

The ticket joins the outstanding optimization validation batch and this business Session does not wait for it before subsequent work. Cargo remains coordinator-owned.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and allocation reductions and label them as CPU ONNX V1 shape-admission evidence for the 65,536-node workload.
