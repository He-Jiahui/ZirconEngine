---
title: Runtime95 Adaptive Shadow Slot Request Capacity
category: zircon_runtime
report_id: Runtime95-adaptive-shadow-slot-request-capacity-2026-08-27
date: 2026-08-27
session_id: root-runtime95-adaptive-shadow-slot-request-capacity-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime95 Adaptive Shadow Slot Request Capacity

## Scope

`shadow_slot_requests_for_additional_lights` previously appended six requests per shadow-casting
point light and one per shadow-casting spot light to an empty `Vec`. A dense 32,768-light frame grew
the request allocation 16 times and repeatedly copied the already-built request prefix.

Blindly counting every enabled light first regressed sparse frames because it doubled the complete
light scan to save at most one small allocation. The production path therefore uses an adaptive
density gate: it samples at most 64 point and 64 spot lights and only performs an exact full count
when that prefix represents at least 32 slot requests. Dense sets allocate the exact request count
once. Sparse sets retain the original single full scan and growing-vector behavior. Both paths use
the same `shadow_enabled` predicate and preserve point-before-spot order, point face order, request
keys, tiers, minimum tiers, priorities, and empty-output behavior.

The 64-light sample and 32-request threshold are performance policy constants backed by the dense
and sparse measurements below. They do not affect which lights cast shadows or which requests the
atlas receives.

This is a bounded current-path improvement. It does not close the parent plan's shadow
policy/allocation authority split, visibility duplication, atlas cache, persistent scratch storage,
GPU upload, depth submission, authoring, or product-scale qualification work.

## Performance Evidence

The isolated release model mirrors 16,384 point and 16,384 spot lights and a 24-byte
`ShadowSlotRequest`-like payload. It runs 31 alternating sample pairs and 16 rounds per sample. The
model was compiled with `rustc -O` on Windows.

The dense workload enables every light and emits 114,688 requests:

| Metric | Growing `Vec` | Adaptive exact capacity | Change |
|---|---:|---:|---:|
| Allocator calls per build | 16 | 1 | -93.750% |
| Cumulative requested bytes per build | 6,291,360 | 2,752,512 | -56.249% |
| P50 for 16 rounds | 40,165,100 ns | 20,212,700 ns | -49.676% |
| P95 for 16 rounds | 62,925,200 ns | 40,611,800 ns | -35.460% |

The sparse boundary enables only the final point light and emits six requests. The density gate
retains the original two allocator calls and 288 requested bytes. P50 changes from 486,400 ns to
527,700 ns (+8.491%) and P95 from 579,700 ns to 578,000 ns (-0.293%). The P50 change remains below
the 10% sparse-path guard and avoids the roughly 30% regression observed when every sparse frame
performed a second complete light scan.

Model source:
`.codex/state/session-coordinator/runtime95-preallocated-shadow-slot-request-capacity-model.rs`.

## Contracts And Validation

- `tools/tests/test_runtime95_adaptive_shadow_slot_request_capacity_performance_contract.py`
  locks the named density constants, shared shadow predicate, optional exact capacity, and sparse
  fallback allocation path.
- Rust behavior tests require a dense mixed point/spot set to produce exact `capacity == len` and
  require a sparse late-enabled point light to bypass exact preallocation while preserving six face
  requests.
- Local source-contract result: 3 tests passed.
- Local `rustfmt --edition 2021 --check` passed for production and Rust test files.
- Dense and sparse release-model modes passed their allocation, requested-byte, P50, and P95 gates.
- Cargo compilation and focused Rust behavior tests remain pending in a later managed asynchronous
  coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime95 still owns authoring roundtrip, physical photometry, real GPU clustered assignment,
unified shadow arbitration and planned views, atlas/cache correctness, stable uploads, batched depth
submission, cookies/IES, diagnostics, failure handling, and same-hardware product qualification.
This slice only removes repeated CPU request-vector growth for dense existing shadow workloads.
