---
title: Runtime07 Preallocated Feature Capability Wait Buckets
category: zircon_runtime
report_id: Runtime07-preallocated-feature-capability-wait-buckets-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Feature Capability Wait Buckets

## Scope

This slice removes geometric growth from the capability wait index used by feature dependency
resolution. The resolver already knows the number of pending feature rows before it consumes the
input, so the wait-bucket map can reserve that lower-bound number of entries up front.

## Change

- Initialize `waiting_by_capability` with `HashMap::with_capacity(pending.len())`.
- Preserve capability keys, waiting-row order, readiness propagation, and unresolved-cycle
  behavior.
- Keep the existing preallocated state and ready queues unchanged.
- Add a Rust regression that locks the one-bucket-per-pending-row capacity contract.
- Add a Python source contract for the bounded initialization path.

## Deterministic Performance Evidence

The standalone optimized Rust model builds 4,096 wait indexes per sample across 31 samples,
with 128 distinct pending features per index. It uses integer keys to isolate map bucket growth
from production string construction. Both runs produced checksum `31`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 4,096 builds | 552,960 | 528,384 | 4.444% |
| Requested allocation bytes | 85,901,312 | 51,445,760 | 40.111% |
| Run 1 build P50 | 83.2094 ms | 60.0079 ms | 27.883% |
| Run 1 build P95 | 133.2566 ms | 91.5648 ms | 31.287% |
| Run 2 build P50 | 85.7581 ms | 60.9342 ms | 28.946% |
| Run 2 build P95 | 156.8863 ms | 103.9594 ms | 33.736% |

Evidence marker: `RUNTIME07_PREALLOCATED_FEATURE_CAPABILITY_WAIT_BUCKETS_MODEL_V1`.

## Validation

- The pre-change Python contract failed its two new wait-index checks; the post-change run passed
  all 3 tests.
- `python -m py_compile` passed for the source contract.
- `rustfmt --edition 2021 --check` passed for the production source and standalone model.
- `git diff --check` passed for the scoped production and contract paths.
- The standalone model retained equivalent map contents and checksum across two runs, with positive
  P50/P95 reductions in both runs.
- Managed Runtime07 Cargo compilation and tests are pending in the next asynchronous batch; this
  slice is not a commit or WeCom milestone until that ticket completes successfully.

Managed batch request: `runtime07-plugin-five-task-batch-20260830-v1`.

Validation attempt: ticket `27e27a159794475b9bd8636cf2859288` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## File Fingerprints

- `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs`
  SHA-256 `F969F0DEC58F26C47AFEF5F413F95169A98DBA54C4D9A69CCD0B86644EF4FE48`
- `tools/tests/test_runtime07_preallocated_feature_capability_wait_buckets_performance_contract.py`
  SHA-256 `3840C7B3B129AB5E6B1C8FA1DA3C468059146D500A924F1175314271DD3EA685`
- `.codex/state/session-coordinator/runtime07-preallocated-feature-capability-wait-buckets-model.rs`
  SHA-256 `456C54AFF08FAF63B74568BBA89AC46E0BCBE212AA66E65097550C8401DB6E13`

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
