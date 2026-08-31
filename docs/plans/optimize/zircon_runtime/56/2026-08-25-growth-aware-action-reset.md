---
title: Runtime56 Growth-Aware Action Reset
category: zircon_runtime
report_id: Runtime56-growth-aware-action-reset-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Growth-Aware Action Reset

## Scope

This slice removes redundant default writes when the reusable action-evaluation workspace grows. It
preserves grow, shrink, same-size, zero-size, storage-reuse, and action evaluation behavior. It does
not change action-map compilation, input consumption, output ownership, or any public input contract,
and does not claim to close Runtime56's remaining device, focus, recording, or product-ingress gaps.

## Implementation

`prepare_actions` previously called `Vec::resize` with `EvaluatedAction::default()` and then filled
the entire resulting slice with the same default. On growth, `resize` already initializes every new
slot, so the following full fill wrote those new slots a second time.

The optimized helper records the portion that existed before resize, lets `resize` initialize only
new slots, and fills only the retained old prefix. Shrinking still resets every surviving slot;
same-size preparation still resets the full workspace; fresh growth does no redundant prefix fill.

The regression compares retired and optimized state field by field across grow, shrink, reuse, and
zero-size transitions. A source contract requires the growth-aware prefix reset and rejects a whole
vector fill in the production path.

## Performance Contract

| Evidence for a fresh 65,536-action reset | Retired path | Optimized gate |
| --- | ---: | ---: |
| Logical default writes | 131,072 | 65,536 |
| Storage allocations after retained-capacity warmup | 0 | 0 |
| Alternating release benchmark | 11 samples x 128 resets | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME56_GROWTH_AWARE_ACTION_RESET_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/action counts, and retired/optimized logical default-write counts.
The dynamic gate intentionally determines whether the release optimizer already removes enough of
the retired redundant work.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and production source guards passed before
submission (apart from the repository's existing CRLF notice). One managed Runtime batch covers
retired/optimized state equivalence, the growth-aware source contract, and the ignored release
benchmark. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery remain
coordinator-owned and pending.
