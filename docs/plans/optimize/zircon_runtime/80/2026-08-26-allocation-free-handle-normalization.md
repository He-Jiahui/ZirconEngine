---
title: Runtime80 Allocation-free Handle Normalization
category: zircon_runtime
report_id: Runtime80-allocation-free-handle-normalization-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime80 Allocation-free Handle Normalization

## Scope

This slice removes the input-sized normalized handle-pair vector from font-handle batch resolution.
It preserves atomic mixed-generation rejection, first-seen unique-pair order, empty-pair handling,
generation metrics, snapshot acquisition, and all public runtime contracts.

## Implementation

`resolve_font_handle_batch` previously normalized every pair into a temporary `Vec`, then scanned
that vector again to build the unique-pair set. The optimized path normalizes each copied pair while
building the unique set, retaining only valid first-seen pairs and allocating no input-sized
normalization buffer.

The ignored release benchmark compares both paths with 16,384 input pairs and 8,192 unique pairs.

## Performance Contract

| Evidence for 16,384 handle pairs | Retired path | Optimized gate |
| --- | ---: | ---: |
| Temporary normalized entries | 16,384 | 0 |
| Normalization/dedup input passes | 2 | 1 |
| Unique result pairs | 8,192 | 8,192 |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME80_ALLOCATION_FREE_HANDLE_NORMALIZATION_BENCH_V1` with pair counts,
structural reductions, P95 timings, and raw samples for coordinator-owned WeCom reporting.

## Validation

Rust 1.94.1 formatting, scoped diff and source-structure checks precede one managed release Cargo
invocation filtered by `runtime80_handle_`. Dynamic P95 evidence, integration SHA, and automatic
WeCom delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime80 still requires session-owned font collections, generational leases, segmented snapshot
publication, font artifacts, pressure recovery, and full product qualification. This batch
resolution optimization does not claim those milestones complete.
