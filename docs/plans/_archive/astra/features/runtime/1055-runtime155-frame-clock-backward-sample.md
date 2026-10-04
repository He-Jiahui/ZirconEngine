---
title: Runtime155 frame clock backward-sample baseline guard
category: zircon_runtime
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/211/2026-09-29-frame-clock-backward-sample.md
implementation_status: completion_candidate_validation_pending
validation_status: static_checks_passed_managed_runtime_validation_pending
---

# Runtime155 frame clock backward-sample baseline guard

`FrameClock` now keeps the last accepted monotonic sample when a public
`ClockSource` returns an earlier `Instant`. The regression tests prove that a
later sample does not double-count elapsed time, including when the earlier
sample is consumed by `rebase`.

Focused checks passed: `rustfmt --edition 2021 --check` and scoped
`git diff --check`. The two new Runtime tests have not run because managed
Cargo validation is pending. Runtime155-P1-007 remains open for typed sample
rejection and diagnosis; this change only prevents a regressed sample from
lowering the accounting baseline.
