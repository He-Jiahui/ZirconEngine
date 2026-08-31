---
title: Runtime07 Streaming Cargo Manifest
category: zircon_runtime
report_id: Runtime07-streaming-cargo-manifest-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Streaming Cargo Manifest

## Scope

This slice removes temporary formatting strings from generated Cargo dependency rows and avoids a
full temporary package-name input. It preserves package-name normalization, dependency order,
target features, platform-specific library sections, paths, and final newlines.

## Change

- Build the `zircon_export_` package name in one exactly sized `String`, then normalize the profile
  output name directly into it.
- Stream each linked runtime crate row into the shared manifest output with `writeln!` instead of
  allocating a formatted row and copying it with `push_str`.
- Keep the existing initial manifest formatting and platform suffixes unchanged.
- Add a Rust byte-for-byte regression covering output-name normalization and a linked runtime crate,
  plus a Python source contract for the allocation-sensitive implementation shape.

## Deterministic Performance Evidence

The standalone optimized Rust model generates one Cargo manifest with 8,192 linked runtime crates
for 17 alternating samples. Inputs are allocated before measurement, and both implementations first
compare the complete output byte-for-byte. Both produced checksum `704794`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 16,401 | 13 | 99.921% |
| Requested allocation bytes | 3,890,853 | 2,022,965 | 48.007% |
| Manifest generation P50 | 3.7258 ms | 1.9510 ms | 47.635% |
| Manifest generation P95 | 5.1370 ms | 3.3431 ms | 34.921% |

Evidence marker: `RUNTIME07_STREAMING_CARGO_MANIFEST_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_streaming_cargo_manifest_performance_contract.py`: 3 passed
  after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts byte-for-byte equality for the complete 8,192-dependency output.
- A Rust regression asserts the complete one-dependency Cargo manifest contract.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending. This candidate will be submitted in one
  asynchronous Runtime07 batch with the frozen streaming native load-manifest candidate rather than
  validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
