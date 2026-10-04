---
title: Editor313 listener projection lower-test wiring repair
category: zircon_editor
report_id: Editor821-listener-projection-test-wiring-repair-2026-09-19
date: 2026-09-19
related_to:
  - docs/plans/optimize/zircon_editor/313/2026-08-30-listener-projection-capacity-v2.md
  - docs/plans/optimize/zircon_editor/313/2026-09-19-listener-projection-capacity-repair.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: lower_regression_wired
---

# Editor821 · listener projection lower-test wiring repair

## Finding

The Editor313 lower Rust regression file existed at
`listener/projection/capacity_tests.rs`, but `projection.rs` did not declare
the child test module. The source/model contract therefore checked a detached
file and the ignored Release marker was not compiled into the Editor listener
projection test tree.

## Repair

`projection.rs` now declares the test-only child module with the existing path.
Production descriptor and delivery projection behavior is unchanged; the
existing lower count/source regression and ignored
`EDITOR313_LISTENER_PROJECTION_CAPACITY_BENCH_V1` marker are now reachable by
the normal Rust test harness.

## TDD and local evidence

- RED: the strengthened source contract failed because the path declaration and
  `mod capacity_tests;` were absent.
- GREEN: the focused contract passes `3/3` after wiring; exact-file Rustfmt
  passes for production and lower test sources.
- The current Runtime/Editor performance-contract batch remains the batched
  validation mechanism; no standalone Cargo process was started.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editor_event/listener/projection.rs` | `86D187D466448B3FF2A053C892D8BAEDC9A696F474B6BD1BCC59C9ED05614797` |
| `zircon_editor/src/core/editor_event/listener/projection/capacity_tests.rs` | `34BF7D1AD342F642AA7A947618CDEEF4EE133B8D5D9601C58BFCFD991B6CDABB` |
| `tools/tests/test_editor_listener_projection_capacity_performance_contract.py` | `8EBC66DB0FCB096C45C05B9E097051231E87A1A21066E524FE9CF7AF3CE6D64E` |

## Acceptance boundary

Managed Windows Cargo/Release compilation, execution of the now-wired lower
regression and ignored marker, allocator observations, and Editor Event
p50/p95/p99 evidence remain pending behind the external dirty worktree gate.
Tooling production remains deferred for the later Rust migration.
