---
title: Runtime46 Shared Service Dependencies
category: zircon_runtime
report_id: Runtime46-shared-service-dependencies-2026-08-27
date: 2026-08-27
session_id: root-runtime46-shared-service-dependencies-20260827
implementation_status: implementation_complete
validation_status: local_contract_passed_managed_validation_pending
---

# Runtime46 Shared Service Dependencies

## Scope

This slice addresses `MOD-P2-06`. Driver, manager, and plugin descriptors now freeze their owned
dependency vectors into immutable `Arc<[DependencySpec]>` slices at construction. Engine service
contracts share that compiled dependency slice instead of deep-cloning every dependency and its
registry-name storage each time a contract wrapper is created.

## Change

- Keep the three descriptor constructors source-compatible for callers that provide a `Vec`.
- Convert the vector once when the descriptor becomes immutable runtime metadata.
- Store the dependency slice in `ServiceContract` as the same shared immutable type.
- Use `Arc::clone` in all three contract constructors.
- Add an in-module Rust contract asserting pointer identity for driver, manager, and plugin slices.
- Add a source-bound Python contract guarding all three descriptor and contract paths.

## Deterministic Performance Evidence

The standalone optimized Rust model freezes 10,000 dependencies once, then alternates legacy deep
clones and shared-slice contract construction for 21 samples with 8 constructions per sample. Each
dependency contains an owned registry-name string. The model asserts identical results, zero
optimized-path allocations, and at least 90% P50/P95 reduction.

| Contract construction with 10,000 dependencies | Before | After | Reduction |
|---|---:|---:|---:|
| Allocations per construction | 10,001 | 0 | 100.000% |
| Allocated bytes per construction | 560,000 | 0 | 100.000% |
| Deep-cloned dependencies | 10,000 | 0 | 100.000% |
| P50 for 8 constructions | 45,198,000 ns | 49,500 ns | 99.890% |
| P95 for 8 constructions | 70,910,200 ns | 395,400 ns | 99.442% |

Evidence checksum: `107,520,000`.

## Validation

- RED: the new Python contract failed all 3 assertions against the former `Vec`-clone path.
- GREEN: `python -m unittest
  tools.tests.test_runtime46_shared_service_dependencies_performance_contract -v`: 3 passed.
- Exact-file `rustfmt --edition 2021 --check` and scoped `git diff --check` pass.
- The standalone Rust model compiles with `rustc --edition=2021 -O` and passes equivalence,
  allocation, byte, and latency gates.
- Cargo compilation and focused Rust tests remain pending in the asynchronous coordinator batch.

## Remaining Parent-plan Work

Runtime46 still owns the module/service authority, lifecycle, composition, factory-binding, and
qualification gaps in the canonical review. This slice only removes repeated dependency-vector
materialization after descriptor construction; it does not claim closure of the broader compiled
module graph.
