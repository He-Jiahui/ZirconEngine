---
title: Runtime56 Skip Frame Axis Self Swaps
category: zircon_runtime
report_id: Runtime56-skip-frame-axis-self-swaps-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Skip Frame Axis Self Swaps

## Scope

This slice removes redundant same-index swaps from frame-axis value and transition compaction. It
preserves sorting, duplicate-axis last-source-wins semantics, binary-search lookup, retained storage,
and all public input contracts. It does not change input ingress, action ownership, device identity,
recording, replay, or Runtime56's remaining product-integration gaps.

## Implementation

`FrameAxisIndex` sorts axis values and transitions by `(input, source_index)` before compacting each
sorted vector in place. The compaction cursor equals the source cursor until the first duplicate. On
the common unique-axis path, the retired implementation therefore called `Vec::swap(index, index)`
for every item even though no data needed to move.

Both compaction loops now call `swap` only when `retained != index`. After a duplicate, later unique
items still move into the compacted prefix; duplicate runs still overwrite the retained entry in
source order, so the latest source value and transition remain authoritative.

The regression compares retired and optimized value and transition signatures across duplicate and
unique inputs. A source contract requires both production compaction loops to guard their swap.

## Performance Contract

| Evidence for 4,096 unique sorted axis values | Retired path | Optimized gate |
| --- | ---: | ---: |
| Same-index `Vec::swap` calls per compaction | 4,096 | 0 |
| Additional scans | 0 | 0 |
| Alternating release benchmark | 11 samples x 256 compactions | optimized P95 <= 80% of retired P95 |

The benchmark emits `RUNTIME56_SKIP_FRAME_AXIS_SELF_SWAPS_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/axis counts, and retired/optimized self-swap counts. The benchmark is
isolated to the in-place compaction stage because frame sorting is unchanged by this slice.

## Validation

The TDD source gate first observed `0/2` required production guards and passed with `2/2` after the
implementation. Rust 1.94.1 formatting, scoped diff checks, behavioral equivalence, and the ignored
release benchmark are submitted as one managed Runtime batch. Dynamic P95 evidence, integration SHA,
automatic commit, and automatic WeCom performance delivery remain coordinator-owned and pending.
