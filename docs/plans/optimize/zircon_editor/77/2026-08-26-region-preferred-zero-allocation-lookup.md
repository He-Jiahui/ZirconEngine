# Editor77 Region Preferred Zero-Allocation Lookup

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826cn_`

## Problem

Every workbench shell geometry solve projected both transient drag extents and token-default extents
from physical to logical units by collecting two temporary `BTreeMap` values. The solver then read
only the fixed Left, Document, Right, and Bottom keys, making those allocations and full-map copies
unnecessary on a frame-sensitive Editor path.

## Optimization

- Introduce a copyable borrowed extent view that retains the physical map and root
  `ResolutionContext`.
- Convert an extent only when the region lookup succeeds; no temporary map or owned key/value copy
  is created.
- Preserve the existing priority order: transient drag extent, persisted drawer extent, token
  default, then zero. Document width continues to accept only its transient override.

## Test And Performance Contract

- The behavior regression covers 2x root scaling, fixed-region hits, missing keys, and no-map input.
- The source regression requires borrowed lookup construction and rejects the former
  `collect::<BTreeMap>` projections.
- Ignored release evidence prints
  `EDITOR77_REGION_PREFERRED_ZERO_ALLOCATION_LOOKUP_BENCH_V1` for 21 alternating sample pairs over
  16,384 four-region lookup batches.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo results,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

