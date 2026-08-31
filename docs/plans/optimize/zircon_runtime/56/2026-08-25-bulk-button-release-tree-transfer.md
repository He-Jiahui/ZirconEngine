---
title: Runtime56 Bulk Button Release Tree Transfer
category: zircon_runtime
report_id: Runtime56-bulk-button-release-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Bulk Button Release Tree Transfer

## Scope

This slice optimizes `ButtonInputState::release_all`, used by the input reset path when all held
controls must become frame-local release transitions. It does not close Runtime56's physical-input
ownership, stable binding, product action-consumer, or lifecycle P0 work.

## Implementation

The previous implementation cloned every pressed key into the return vector, then performed one
`BTreeSet::take` and one transition-set insertion for every key. The new implementation moves the
whole pressed tree out with `mem::take`, preserves the sorted public return value with the one
required clone pass, and transfers the owned tree into `just_released`:

- an empty transition tree receives the released tree by direct move with zero key comparisons;
- a non-empty transition tree uses `BTreeSet::append` to merge owned nodes;
- `pressed`, `just_pressed`, existing `just_released`, and sorted return-value semantics stay
  covered by the focused regression;
- the return-value clone completes before mutation, so a panicking generic `Clone` leaves held and
  transition state unchanged as it did before the optimization.

## Performance Contract

| Evidence | Baseline | Optimized gate |
| --- | ---: | ---: |
| Empty transition tree, 4,096 held buttons | 4,096 per-key searches plus per-key insertion | 0 key comparisons during tree transfer |
| Release benchmark | 32,768 held buttons, 11 alternating samples | optimized P95 <= 60% of baseline P95 |
| Absolute release budget | not bounded | optimized P95 <= 50 ms on Windows-native release validation |

The release benchmark emits `RUNTIME56_BULK_BUTTON_RELEASE_BENCH_V1` with baseline and optimized
P95 nanoseconds plus percentage reduction. Actual timings are accepted only from the coordinator's
terminal Windows-native release evidence and remain pending in this record.

## Validation

The managed batch covers three correctness/work regressions and one ignored release benchmark in one
Cargo invocation. Exact `rustfmt --check` and scoped `git diff --check` passed before submission
(only the repository's existing CRLF notice was emitted). Test execution, performance gate results,
integration SHA, and the automatic WeCom performance notification remain pending until terminal
coordinator evidence exists.
