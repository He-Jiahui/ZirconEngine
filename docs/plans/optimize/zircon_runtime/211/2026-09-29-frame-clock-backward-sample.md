---
title: Runtime155 frame clock backward-sample baseline guard
category: zircon_runtime
report_id: runtime155-frame-clock-backward-sample-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/211-runtime-clock-time-policy-world-fixed-step-timer-cadence-current-working-tree-review.md
implementation_status: source_candidate_pending_managed_validation
validation_status: rustfmt_and_scoped_diff_check_passed
performance_status: correctness_guard_no_performance_claim
---

# Runtime155: reject backward frame-clock baseline movement

`FrameClock::tick` previously converted a backward `ClockSource` sample into a
zero delta and still replaced its baseline. When the source later advanced
again, time between the last valid sample and the regressed sample was counted
twice. `rebase_for` could lower the same baseline.

The frame clock now advances its baseline only when
`Instant::checked_duration_since` accepts the sample. A backward tick returns
zero while retaining the last accepted baseline; a backward rebase retains it
as well. Focused tests cover both paths and assert that the later delta starts
at the last accepted sample.

This is a bounded safety guard for Runtime155-P1-007. It does not add a typed
regression receipt or diagnose/quarantine a bad `ClockSource`, so the full
source-admission contract remains open.

## Evidence and remaining gates

- Added `backward_clock_sample_does_not_lower_the_frame_delta_baseline` and
  `backward_rebase_sample_does_not_lower_the_frame_delta_baseline` in
  `zircon_runtime/src/core/runtime/frame_clock.rs`.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` passed.
- No Cargo command ran. Managed Runtime tests remain pending; no performance
  measurement or performance improvement is claimed.
