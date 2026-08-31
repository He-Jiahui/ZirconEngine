---
title: Runtime07 Streaming Materialized Path Normalization
category: zircon_runtime
report_id: Runtime07-streaming-materialized-path-normalization-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Streaming Materialized Path Normalization

## Scope

This slice removes the temporary component vector from generated-file and archive-path
normalization. It affects the shared export materialization path but does not change accepted path
syntax, traversal rejection, root joining, ZIP entry ordering, or filesystem behavior.

## Change

- Reserve one output `String` using the input byte length, which is an upper bound after separator
  normalization.
- Stream each UTF-8 `Component::Normal` directly into the output and insert `/` only between
  accepted components.
- Remove the temporary `Vec<&str>` and final `join` allocation.
- Cover unchanged portable output, repeated-separator normalization, empty/current/parent path,
  backslash, and trailing-separator rejection with Rust regressions and a Python source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model normalizes 8,192 valid eight-component materialized paths for
17 alternating samples. Both implementations use `Path::components` and produced checksum
`679936`. The table records the more conservative complete run.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 24,576 | 8,192 | 66.667% |
| Requested allocation bytes | 2,252,800 | 679,936 | 69.818% |
| Normalization P50 | 6.6781 ms | 4.7810 ms | 28.408% |
| Normalization P95 | 8.3390 ms | 6.4620 ms | 22.509% |

Evidence marker: `RUNTIME07_STREAMING_MATERIALIZED_PATH_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_streaming_materialized_path_normalization_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- Direct standard-library `rustc --test` wrapper: 2 passed, covering output and rejection behavior.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  with the archive projection candidate.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
