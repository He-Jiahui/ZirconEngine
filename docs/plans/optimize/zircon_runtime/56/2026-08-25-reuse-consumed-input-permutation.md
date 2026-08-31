---
title: Runtime56 Reuse Consumed Input Permutation
category: zircon_runtime
report_id: Runtime56-reuse-consumed-input-permutation-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Reuse Consumed Input Permutation

## Scope

This slice reuses the sorted index permutation for caller-owned consumed button and axis slices when
that permutation remains valid for the current values. It preserves membership queries, duplicate
handling, source ownership, storage reuse, and all public input contracts. It does not cache caller
values, assume that caller slices are initially sorted, or change Runtime56's remaining ownership,
device, recording, replay, and product-ingress gaps.

## Implementation

`ConsumedInputIndex::load` previously cleared, rewrote, and sorted both reusable index vectors on
every action evaluation. The new shared loader first checks whether the retained permutation has the
current length and is still ordered by the current caller-owned values. A valid permutation returns
without writing the index vector. A length change or an ordering violation falls back to the retired
clear, identity-fill, and indirect-sort path.

The retained indices are private and can only be created by this loader, so equal lengths preserve a
complete in-bounds permutation. Caller mutation cannot make membership results stale: values are read
through the current slice during validation and every binary search.

The regression starts with unsorted button and axis slices, repeats a stable load, mutates an axis
slice at the same length, and verifies that the invalid permutation is rebuilt while membership
queries remain correct. A source contract requires both production paths to use the shared early
return before any rebuild write.

## Performance Contract

| Evidence for a stable 4,096-axis consumed-input reload | Retired path | Optimized gate |
| --- | ---: | ---: |
| Index writes per reload | 4,096 | 0 |
| Sorts per reload | 1 | 0 |
| Ordered validation comparisons | sort implementation dependent | 4,095 |
| Alternating release benchmark | 11 samples x 128 reloads | optimized P95 <= 35% of retired P95 |

The benchmark emits `RUNTIME56_REUSED_CONSUMED_INPUT_PERMUTATION_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/input counts, index writes, and sort counts. The benchmark
warms both implementations once, then measures repeated loads of the same unsorted unique input set.

## Validation

The TDD source gate first observed no production helper. After implementation it found the generic
definition and both production call sites; a mistaken definition-as-call count in the test itself was
corrected before submission. Rust 1.94.1 formatting, scoped diff checks, behavioral/source tests, and
the ignored release benchmark are submitted as one managed Runtime batch. Dynamic P95 evidence,
integration SHA, automatic commit, and automatic WeCom performance delivery remain coordinator-owned
and pending.
