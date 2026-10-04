---
title: Editor75 Timeline ruler upper-bound capacity
category: zircon_editor
report_id: Editor761-timeline-ruler-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor75 Timeline ruler upper-bound capacity

## Scope

`build_timeline_ruler_ticks` already derives a finite desired interval count and
enforces `MAX_RULER_TICKS`, but its non-zero-duration path still started the
output vector empty. The interval bound is authoritative enough to admit the
generated ticks plus the optional start/end endpoint slots before the loop. This
is a capacity-only follow-up to the Editor75 timeline review and PERF-MVP-215;
the larger tick-cache, LOD, and shared-generation work remains separate.

## Implementation

- Added `ruler_tick_capacity`, which clamps finite interval estimates to the
  4,096 hard cap, treats positive overflow as the hard cap, and adds two endpoint
  slots with saturating arithmetic.
- `build_timeline_ruler_ticks` now uses that bound before generating labels. The
  zero-duration fast path, nice-step calculation, floating-point guard, hard
  tick cap, endpoint insertion, ordering, and label formatting are unchanged.
- The helper remains crate-internal to the ruler parent; no new public cache or
  timeline authority is introduced.

## Deterministic pressure model

At the 4,096 interval ceiling, the admitted output capacity is 4,098 (generated
ticks plus the possible start/end endpoints). The legacy empty vector incurs
geometric growth for a non-empty batch, while the reserved model incurs zero
growth events. The ignored marker
`EDITOR761_TIMELINE_RULER_CAPACITY_BENCH_V1` records this lower allocation-shape
comparison. It is not a CPU, RSS, or product p50/p95/p99 measurement.

## TDD and local evidence

- The new source contract was intentionally RED before `ruler_tick_capacity`
  existed, then GREEN at `4/4` after the helper and reservation were wired.
- The timeline test module covers finite estimates, the positive-overflow hard
  cap, and the lower ordering/endpoint behavior; the ignored Release marker is
  ready for the next owner-attributed batch.
- Scoped Rustfmt, Python compilation, and `git diff --check` are run together
  with the adjacent Editor760 slice. The focused five-contract capacity batch
  passes `21/21` in `0.127s`; the merged Runtime/Editor performance-contract
  batch loads `515` modules and passes `1844/1844` in `47.661s` in one process.
  These are local source/model receipts, not managed Cargo or product latency
  evidence.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline/ruler.rs` | `84BE8F51B569A57334A64C4B7219DE9E76A686C31E4E440E9A68407AF1D78B2E` |
| `zircon_editor/src/ui/timeline/tests.rs` | `29CE143FE85D1F2E63D76D37590CED8D10C6217726380C674978B832C0CDD66A` |
| `tools/tests/test_editor_timeline_ruler_capacity_performance_contract.py` | `EE0F65576D90CC1527F9E0ED5A77BDC509A232DF2C509DA29B31E3FB74672172` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, tick parity,
allocation behavior, and timeline paint p50/p95/p99. No standalone Cargo
command or coordinator status query is used here; tooling production work stays
deferred by request.
