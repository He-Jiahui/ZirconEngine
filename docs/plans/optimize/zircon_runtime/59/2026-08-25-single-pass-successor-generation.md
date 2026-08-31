---
title: Runtime59 Single-pass Successor Generation
category: zircon_runtime
report_id: Runtime59-single-pass-successor-generation-2026-08-25
date: 2026-08-25
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Single-pass Successor Generation

## Scope

This slice removes a redundant queued-work scan when bounded keyed IO decides whether an admission
has already been superseded. It preserves key and epoch filtering, fence exclusion, maximum
successor-generation selection, terminal notification, reservation release, and all public
task/runtime contracts. It does not alter the later coalescing partition path.

## Implementation

`coalesce_queued_generation` previously used `Iterator::any` to discover a newer queued generation,
then traversed the same queue again to compute the maximum active/queued generation for the terminal
receipt. The optimized path streams the matching active entry and queued entries through one
`latest_generation_above` reduction. `None` retains the existing admission path; `Some(max)` retains
the existing superseded terminal path.

The regression verifies that equal and older generations do not supersede the incoming entry, the
latest newer generation is selected, and every candidate is visited exactly once.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Generation visits for a 4,096-entry late-match queue | 8,192 | 4,096 |
| Alternating release benchmark | 11 samples x 256 scans | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME59_SINGLE_PASS_SUCCESSOR_GENERATION_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/entry counts, and retired/optimized generation visits.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and source-structure checks are required before
submission. One managed Runtime59 Cargo invocation filtered by `runtime59_coalescing_` covers the
behavior regression and ignored release benchmark together with the ordered-insertion optimization.
Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery remain
coordinator-owned and pending.
