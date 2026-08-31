---
title: Runtime Event Diagnostics Power Of Two Sampling 565
category: zircon_runtime
report_id: Runtime565-event-diagnostics-power-of-two-sampling-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Event Diagnostics Power Of Two Sampling 565

Event diagnostics sampled routine timing with a runtime `% interval` operation for every
publish/delivery counter. Power-of-two intervals now use the equivalent `sample_index &
(interval - 1)` predicate; zero remains disabled and non-power-of-two intervals retain modulo
semantics. The change preserves the exact sample indices and does not alter counters or timing
payloads.

A Rust 1.94.1 `opt-level=3` standalone benchmark with a runtime-variable interval of 64 used
13 interleaved pairs and 50,000,000 predicate calls per sample. P95 changed from
`120,581,900 ns` to `5,735,500 ns`, a `95.24%` reduction. The benchmark also compared all
sample indices through 4,096 and found identical decisions.

## Static evidence

- TDD RED: the power-of-two sampling helper was absent before implementation.
- TDD GREEN: focused tests compare power-of-two and non-power-of-two intervals, including zero,
  against the legacy modulo predicate.
- Benchmark marker: `RUNTIME565_EVENT_SAMPLING_MASK_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831ez_runtime565_`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `0f7896abc15a163c6e99c0ff218699feadc141730b8e4ae808eeceb784b37040`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime event tests pass.
2. Sampling decisions remain identical for zero, power-of-two, and arbitrary non-power-of-two intervals.
3. Diagnostics counters and sampled timestamps retain existing index semantics.
4. Coordinator records the clean-copy validation result before commit/push and WeCom publication.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
