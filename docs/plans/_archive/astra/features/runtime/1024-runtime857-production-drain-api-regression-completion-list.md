---
doc_type: milestone-detail
status: candidate_static_review_complete_managed_validation_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/level_system/2026-09-28-runtime857-production-drain-api-regression.md
related_code:
  - zircon_runtime/src/scene/level_system/animation_runtime.rs
  - zircon_runtime/src/scene/level_system/animation_runtime/drain_output_capacity_tests.rs
tests:
  - runtime857_production_drain_reserves_capacity_and_preserves_sample_order
  - runtime857_production_empty_drain_keeps_zero_capacity
  - runtime857_production_event_budget_requeues_and_resumes_remaining_ranges
  - runtime857_animation_drain_output_capacity_bench
---

# Runtime857 production drain API regression

| Slice | Work | Status | Evidence |
|---|---|---|---|
| Runtime857 successor | Exercise output reservation, ordering, empty drain, budget requeue, and resume through `LevelSystem::drain_animation_clip_events`; profile the actual production API | `candidate_static_review_complete_managed_validation_pending` | Pinned Rustfmt 1.94.1 edition 2021 passes. Managed compile/tests and the ignored 31-sample Windows Release profile remain pending. |

The production source file remains an existing modified path and was not edited
by this successor. Its current SHA-256 is
`45088df65f39e5aec4f4721e2e4d4b8b5380e0f9aa22dcfc2c030510f42a7948`; include
this exact source in the next grouped manifest so the regression runs against
the reservation implementation. The candidate test file SHA-256 is
`5d7cf0694e26af1e7ba1a0231d33b47082a47b255aa47230d84790f16442baeb`.

The Release profile prints 31 raw latency samples and nearest-rank p50/p95/p99.
Range construction and queue setup are outside timing; `OrderedSampler` creates
event vectors during each measured production drain, so these samples include
sampler output allocation and are not collector-only timings. The profile has
not been run and has no threshold or allocation claim. No managed validation
pass is claimed.

The candidate test file SHA-256 is
`5d7cf0694e26af1e7ba1a0231d33b47082a47b255aa47230d84790f16442baeb`.
