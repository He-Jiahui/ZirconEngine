---
title: Runtime80 Single-pass Handle Rejection Projection
category: zircon_runtime
report_id: Runtime80-single-pass-handle-rejection-projection-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime80 Single-pass Handle Rejection Projection

## Scope

This slice combines ordered resolution projection and rejected-pair counting into one pass. It
preserves per-input output order, stale-handle normalization, missing result behavior, face and
instance rejection semantics, metrics, and public APIs.

## Implementation

The retired path first allocated the complete result vector, then zipped the original input with
that result in a separate traversal to count rejected face or instance handles. The optimized helper
normalizes and resolves each pair, updates the rejection counter, and pushes the result within the
same ordered projection.

A regression covers current, stale, and empty pairs. The ignored release benchmark uses 16,384
current pairs and an indexed resolution map.

## Performance Contract

| Evidence for 16,384 projected pairs | Retired path | Optimized gate |
| --- | ---: | ---: |
| Result/rejection projection passes | 2 | 1 |
| Output entries | 16,384 | 16,384 |
| Rejection semantics | exact | exact |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 90% of retired P95 |

The benchmark emits `RUNTIME80_SINGLE_PASS_HANDLE_REJECTION_PROJECTION_BENCH_V1` with pair/pass
counts, P95 timings, and raw samples for coordinator-owned WeCom reporting.

## Validation

Rust 1.94.1 formatting, scoped diff and source-structure checks precede one managed release Cargo
invocation filtered by `runtime80_handle_`, shared with allocation-free normalization. Dynamic P95
evidence, integration SHA, and automatic WeCom delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime80 still requires session-owned font collections, generational leases, segmented snapshot
publication, font artifacts, pressure recovery, and full product qualification. This projection
optimization does not claim those milestones complete.
