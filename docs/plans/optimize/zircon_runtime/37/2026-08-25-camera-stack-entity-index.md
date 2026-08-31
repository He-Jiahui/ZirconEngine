---
title: Runtime37 Camera Stack Entity Index
category: zircon_runtime
report_id: Runtime37-camera-stack-entity-index-2026-08-25
date: 2026-08-25
session_id: root-runtime37-two-task-camera-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime37 Camera Stack Entity Index

## Scope

This slice reduces overlay-reference resolution cost in Runtime37's camera stack resolver, aligned
with CAM-P1-020 and the plan's multi-view scale direction. It does not claim the parent plan's
camera source, director, rig, cinematic, history-retirement, network, or product-fixture work is
complete.

## Implementation

After active cameras receive their existing stable sort, the resolver builds one capacity-sized
entity index. Every base stack reference now performs an indexed lookup instead of rescanning the
full active camera sequence.

The index uses `entry(...).or_insert(...)`, so duplicate entity IDs retain the first descriptor in
the sorted sequence, matching the previous iterator `find` behavior. Hash-map iteration never
drives output; base and overlay output ordering, target and viewport inheritance, and all four
existing violation dispositions remain unchanged.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K active cameras, 10K references to the last camera | 100,000,000 entity comparisons | 10K index inserts + 10K indexed lookups | 99.98% lookup-work reduction |
| Stack resolution complexity | O(N x M) after sort | expected O(N + M) after sort | repeated full scan removed |
| Entity index allocation | none | one capacity-sized map per resolution | bounded by active camera count |
| Focused release wall-clock target | unbounded | <= 500 ms | pending terminal evidence |

The ignored Windows-native release evidence prints `RUNTIME37_CAMERA_STACK_INDEX_BENCH_V1` with
camera/reference counts, legacy entity comparisons, indexed operations, reduction percentage,
elapsed milliseconds, and the target. Exact wall-clock evidence is accepted only from the
coordinator's terminal result.

## Validation

- RED proved the first-match behavior test referenced a missing index while production performed
  a linear scan for every stack reference.
- First-match preservation, duplicate stack-reference output, existing violation behavior, and the
  ignored 10K-by-10K release gate are prepared for one Runtime37 camera batch.
- The managed `runtime37_batch_` release gate covers both camera optimizations in one Cargo
  invocation: 2 source contracts, 5 Rust tests, and 2 performance rows.
- Scoped `rustfmt --check`, `git diff --check`, and both source contracts pass locally. Dynamic
  marker values, integration commit, and WeCom delivery remain coordinator-owned and pending.

## Documentation Decision

The public camera documentation does not promise the internal overlay-reference lookup algorithm.
Camera stack order and violation semantics are unchanged, so this scoped optimization record is
the only documentation change.

## Remaining Parent-plan Work

Versioned endpoint source, projection validation, director ownership, rig evaluation, input
arbitration, collision/blend/shake, explicit cuts, bounded history, multi-view/XR, persistence,
and full product-scale qualification remain open under Runtime37.
