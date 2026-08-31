---
title: Runtime07 In-place Animation Bool Parameter
category: zircon_runtime
report_id: Runtime07-in-place-animation-bool-parameter-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 In-place Animation Bool Parameter

## Scope

This slice removes a temporary parameter-name allocation from repeated guest
`set_animation_bool` calls. The World/player read and write path remains unchanged; only mutation of
an already-present `AnimationParameterMap` key is updated in place.

## Change

- Look up an existing animation parameter with borrowed `&str` and replace only its value.
- Allocate and insert an owned key only when the parameter is missing.
- Record guest string-copy metrics only on that actual first insertion.
- Preserve the Boolean value, missing-player result, World writeback, and first-insert behavior.
- Add a Rust existing/missing-key regression plus a Python source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model isolates the repeated `BTreeMap<String, Value>` mutation layer
with 64 existing parameters and a representative 56-byte guest parameter name. Each sample performs
131,072 updates, alternates legacy and optimized order across 31 samples, counts allocator calls and
requested bytes for one update, and verifies identical final values and rolling checksums. It
deliberately excludes the common player clone and World writeback costs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Existing-key update allocation calls | 1 | 0 | 100% |
| Existing-key update requested bytes | 56 | 0 | 100% |
| Existing-key update P50 | 23.9251 ms | 13.9361 ms | 41.751% |
| Existing-key update P95 | 52.5048 ms | 23.1322 ms | 55.943% |

Evidence marker: `RUNTIME07_IN_PLACE_ANIMATION_BOOL_PARAMETER_MODEL_V1`.

A second complete run remained favorable: P50 improved 39.248% and P95 improved 76.874%.
Both paths produced checksum `830875648711917568`.

## Validation

- `python tools/tests/test_runtime07_in_place_animation_bool_parameter_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust 1.94.1 model compiled and passed two complete 31-sample runs with identical
  values and checksums.
- The Rust guard verifies allocation intent for both an existing `moving` parameter and a missing
  `grounded` parameter while preserving their Boolean values.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
