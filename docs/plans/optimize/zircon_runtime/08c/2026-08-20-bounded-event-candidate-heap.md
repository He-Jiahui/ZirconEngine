# Runtime08C Bounded Event Candidate Heap Optimization Record

- Date: 2026-08-20
- Owner: `optimize-runtime08c-event-heap-r1-01a00797-20260820`
- Source plan: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-15
- Status: implementation and 21-pair release-gate definition complete; combined managed validation pending

## Problem

`ProjectAnimationClipEventSampler` collected one candidate per event track and
then rescanned the entire remaining vector with `min_by` before every emitted
event. Sampling E one-shot events from E tracks therefore performed E * (E +
1) / 2 candidate visits and approached O(E^2). Removing the selected vector
entry also shifted later candidates.

## Change

- Initial candidates are collected into a reversed `BinaryHeap` ordered by
  playback time, event name, and track index.
- Sampling peeks the earliest candidate before the byte-budget decision, pops
  only accepted events, and reinserts a looping candidate at its next playback
  time.
- Budget exhaustion therefore retains the unconsumed candidate, while existing
  cursor, oversized-first-event, same-time ordering, and looping semantics stay
  unchanged.
- A debug regression limits 1,024-event selection to 32 comparisons per event.
  A duplicate same-time event regression locks the track-index tie breaker.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Select 2,048 one-shot events | 2,098,176 full-vector candidate visits | 2,048 heap pops plus heap construction | 99.90% of full-vector visits removed |
| Select E events from T tracks | O(E * T) selection, approaching O(E^2) | O(T + E log T) | one complexity class |
| Remove selected candidate | vector shift | O(log T) heap repair | linear shift removed |

The ignored release gate runs 21 alternating legacy/heap sample pairs, emits
all raw microsecond samples, and computes nearest-rank P50/P95. Acceptance
requires heap P95 to be no more than 25% of legacy P95. Exact Windows timing
values remain pending the combined coordinator batch.

## Acceptance

- Existing bounded, resumable, byte-budget, event-count, and looping tests keep
  the public sampling behavior fixed.
- `same_time_duplicate_events_resume_by_track_index` locks the final stable
  ordering tie breaker.
- `event_candidate_selection_scales_subquadratically` requires no more than
  32,768 comparisons for 1,024 events in the debug validation build.
- `event_candidate_heap_release_benchmark_evidence` emits
  `ANIMATION_EVENT_CANDIDATE_HEAP_BENCH_V1` with `sample_pairs=21`, alternating
  order, nearest-rank P50/P95, raw samples, and the 25% P95 threshold.
- The current managed validator combines five Runtime08C slices with
  Runtime45, Runtime48, and Runtime49, covering eight logical tasks in twelve
  Cargo groups. It independently recomputes all raw-sample percentiles and
  requires seven performance gates. Validator SHA-256:
  `A2C1864BDCA19026FD02493EC066031AF95CE6A050E59A608859C64FBC9E0943`.
- Exact-file Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Cargo regressions and release timing: pending a multi-task coordinator batch;
  no direct or competing Cargo process was started.

## Remaining Plan Work

This slice does not close Runtime08C P1-15. Cooked time-sorted dense event
tables, loop-segment indexing, direct binary/cursor advancement, canonical
gameplay event ABI ownership, generation and stale-delivery policy, downstream
count/byte/time budgets, and a real gameplay consumer remain open.
