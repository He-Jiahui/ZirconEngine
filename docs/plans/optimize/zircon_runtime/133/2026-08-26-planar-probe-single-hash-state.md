# Runtime133 Planar Probe Single Hash State

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826cp_`

## Problem

`PlanarReflectionUpdateState` tracked captured and dirty probe ids in two `BTreeSet` values. An
on-demand probe already captured in a previous frame required two tree lookups, while transitions
could update both trees for one logical state.

## Optimization

- Replace the two trees with one private `HashMap<u64, PlanarProbeCaptureState>`.
- Represent `Captured` and `Dirty` explicitly; absence retains the never-captured/forgotten state.
- Resolve on-demand capture with one map lookup while preserving the unconditional EveryFrame
  path and all public methods.

## Test And Performance Contract

- The behavior regression covers two independent probes, first capture, captured steady state,
  dirty recapture, successful capture, forget, and EveryFrame.
- The source regression requires the single hash owner and rejects both old trees and their lookup
  paths.
- Ignored release evidence prints `RUNTIME133_PLANAR_PROBE_SINGLE_HASH_STATE_BENCH_V1` for 21
  alternating sample pairs over 65,536 probes with one dirty probe per four entries.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo results,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

