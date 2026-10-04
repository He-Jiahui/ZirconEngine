---
title: Runtime animation missing-track diagnostic capacity
category: zircon_runtime
report_id: Runtime839-animation-missing-track-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime839 · animation missing-track diagnostic capacity

## Scope

`compile_sequence_for_world` already knows the complete compiled source track
count. Its report-only `missing_tracks` vector nevertheless grew from zero
capacity whenever a target or writer was missing. Successful sequence compiles
should remain allocation-free for this diagnostic buffer, while missing-track
compiles can reserve the known upper bound once.

## Implementation

- Keep `missing_tracks` zero-capacity at function entry.
- On the first missing entity or missing property writer, reserve the source
  track-count upper bound before extending/pushing the existing diagnostic
  path values.
- Preserve missing-path order, exact path payloads, successful writer
  compilation, binding/track indices, and apply-time behavior.
- Add the lower lazy-success/order/source regression and ignored
  `RUNTIME839_ANIMATION_MISSING_TRACK_CAPACITY_BENCH_V1` Release marker.

This slice does not change animation target resolution, writer compilation,
sampling, source ownership, or diagnostic semantics. Tooling production remains
out of scope.

## Deterministic work model

For 4,096 missing tracks, the old zero-capacity diagnostic collector models 11
geometric growth events; the first-miss source-count reservation changes that
to `11→0`. A fully successful compile retains zero capacity, so the common
success path does not pay a diagnostic allocation. This is allocation-shape
evidence only; it is not a claim about allocator, CPU, RSS, or product
p50/p95/p99 performance.

## TDD and local evidence

- The Python source/model contract was intentionally RED against the original
  zero-capacity missing-track collector, then GREEN after both missing branches
  gained the lazy reservation and lower-test wiring (`2/2`).
- The lower Rust regression checks that success leaves zero capacity, the first
  missing path preserves order and reserves the source bound, and the ignored
  marker reports p50/p95/p99 fields for legacy versus reserved collection.
- Exact-file Rustfmt and Python compilation pass. The focused thirteen-contract
  Runtime/Editor batch passes `49/49` tests in `0.043s`; the refreshed broad
  one-process non-tooling loader covers `861` modules and passes `3489/3489`
  tests in `33.065s`, with zero failures, errors, load errors, or skips.
- Managed Cargo/Windows Release, allocator, and animation product percentile
  evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/sequence/compiled.rs` | `2B8871DE1AB02CA0E8CA7DAC8A5302180A32BB0A74A17C881EBF095C78EBA1E1` |
| `zircon_runtime/src/animation/sequence/compiled_missing_track_capacity_tests.rs` | `2CE6D764C8F7052805F39D3A077955D58FF52E58D25A7615D1064BF0A8434DFD` |
| `tools/tests/test_runtime_animation_missing_track_capacity_performance_contract.py` | `3521526F5E2CAD15953BEFCE16FFB141950925FACFBFAF0EC302DA23BCCA2A00` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, successful and
missing-track diagnostic parity, allocation behavior, and the declared Runtime
animation product p50/p95/p99 gates.
