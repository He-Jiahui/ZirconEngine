---
title: Runtime857 production drain API regression
category: zircon_runtime
report_id: Runtime857-production-drain-api-regression-2026-09-28
date: 2026-09-28
implementation_status: candidate_static_review_complete
validation_status: managed_validation_pending
performance_status: ignored_release_profile_added_unrun
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

## Gap and correction

The earlier lower-level tests exercised `model_collect`, a test-local copy of
the vector reservation pattern. They did not call
`LevelSystem::drain_animation_clip_events`, so they could pass if the production
drain stopped reserving its output vector. This successor retains those model
cases and adds behavior checks through the real `LevelSystem` API.

The production change in `animation_runtime.rs` is pre-existing and was not
edited in this successor. Its current SHA-256 is
`45088df65f39e5aec4f4721e2e4d4b8b5380e0f9aa22dcfc2c030510f42a7948`; the next
grouped source manifest must capture these exact bytes so the tests validate
that implementation. The lower test file's earlier snapshot hash was
`3bab65a6a73ce115cf56b9d3eb507ca600b6c56152f3c916fb3c8759707f8ba1`; the new
candidate test file hash is
`5d7cf0694e26af1e7ba1a0231d33b47082a47b255aa47230d84790f16442baeb`.

## Production behavior covered

- A non-empty drain emits ordered events through the injected sampler and
  returns capacity at least equal to the production event limit.
- An empty queue returns an empty vector with zero capacity.
- Filling the event budget requeues the remaining range; the next drain resumes
  it without losing the queued entity/order.
- The ignored Release profile runs the production drain for 31 raw samples with
  32 prepared ranges and 64 events per sample. Range construction and queue
  setup happen before timing. The `OrderedSampler` constructs its event vectors
  during each timed drain, so raw latency includes sampler output allocation
  and production drain work; it is not a collector-only measurement. It prints
  raw nanoseconds and nearest-rank p50/p95/p99.

The profile has no allocation counter and defines no latency threshold. It has
not been run, so it supplies no measured pass or product-performance claim.

## Validation boundary

Pinned Rustfmt `1.94.1`, edition 2021, passes for the candidate test file. Cargo
and managed Rust tests were not run; grouped managed compilation, behavior
execution, and the ignored Windows Release profile remain pending. The source
and test paths are outside the frozen 480-path v2 manifest. Coordinator status
could not be read because the bundled PowerShell runtime could not load its
`Microsoft.PowerShell.Management` module; a separate read-only lease check
reported no lease rows for the source and test paths or the two new record
paths.
