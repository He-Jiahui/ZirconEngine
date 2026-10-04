---
related_code:
  - zircon_runtime/src/scene/ecs/change_detection/mod.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick_window.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/change_detection/stats.rs
  - zircon_runtime/src/scene/ecs/change_detection/wrappers.rs
  - zircon_runtime/src/scene/ecs/query/query_filter.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
implementation_files:
  - zircon_runtime/src/scene/ecs/change_detection/mod.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick_window.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/change_detection/stats.rs
  - zircon_runtime/src/scene/ecs/change_detection/wrappers.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
tests:
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/change_detection/mod.rs zircon_runtime/src/scene/ecs/change_detection/stats.rs zircon_runtime/src/scene/ecs/mod.rs zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - cargo test -p zircon_runtime --lib change_detection_scan_stats_record_mark_checks_and_diagnostics --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 07 scan telemetry: passed, 1 passed; 0 failed)
  - cargo test -p zircon_runtime --lib change_detection_scan_skips_unmarked_archetypes --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 07 M1.2 named assertion: pending after render-owned HZB compile blocker clears; source/rustfmt static checks passed)
doc_type: module-detail
---

# ECS Change Detection

## Purpose

The change detection module owns tick comparison for ECS components and resources. It lets query filters and wrapper types decide whether a component was added or changed within the current system run window without forcing world-level authoring state into runtime ECS.

Runtime 07 adds a small scan-statistics surface beside the existing tick model. The goal is to expose how many change marks a hot path inspected and how many `Added` or `Changed` matches were found, while leaving automatic frame-level collection to a later runtime diagnostics integration slice.

## Related Files

- `change_tick.rs` owns wrapping tick arithmetic, stale tick clamping thresholds, and `is_newer_than(...)`.
- `change_tick_window.rs` owns the `last_run` / `this_run` window passed into system params and query filters.
- `component_ticks.rs` owns the per-component `added` and `changed` tick pair plus the `is_added(...)` and `is_changed(...)` checks.
- `wrappers.rs` owns `Ref<T>` and `Mut<T>` wrappers that expose tick checks beside borrowed component values.
- `stats.rs` owns `ChangeDetectionScanStats` and the `ecs.change_detection.*` diagnostic path constants.

## Behavior Model

`ChangeTick` uses wrapping subtraction so tick comparisons remain valid across `u64` rollover. `ChangeTickWindow::new(...)` clamps a stale `last_run` tick to the maximum representable change age before the window is used. `ComponentTicks::is_added(...)` and `ComponentTicks::is_changed(...)` then compare the component's stored tick against that window.

`ChangeDetectionScanStats` is an explicit counter object. Calling `scan_added(...)` or `scan_changed(...)` increments `scanned_marks`, performs the same `ComponentTicks` predicate as the direct API, and increments the matching counter only when the predicate returns true. `merge(...)` combines counters with saturating arithmetic so per-filter or per-system local stats can be aggregated before being recorded.

## Diagnostics

`ChangeDetectionScanStats::record_diagnostics(...)` writes the current counters into the shared `DiagnosticStore` with `ecs` and `change_detection` tags:

- `ecs.change_detection.scanned_marks`
- `ecs.change_detection.added_matches`
- `ecs.change_detection.changed_matches`

This is a local projection API, not a global frame collector. Query filters still call `ComponentTicks` directly today. Future Runtime 07 frame-level collection should reuse this type rather than inventing another counter vocabulary.

## Constraints

The module must stay folder-backed. `mod.rs` should only declare and re-export child modules. Tick arithmetic remains in `change_tick.rs`; scan counter projection remains in `stats.rs`; query filter behavior remains in `query/query_filter.rs`.

The scan-stat API must not change `Added` or `Changed` query semantics. It mirrors the same predicates and is safe to use as a measurement wrapper around existing checks.

## Test Coverage

`change_detection_scan_stats_record_mark_checks_and_diagnostics` verifies mark scan counts, added and changed match counts, counter merging, and diagnostic snapshot readback. `change_detection_scan_skips_unmarked_archetypes` is the Runtime 07 M1.2 named assertion for the current local counter layer: it scans stale/unmarked component tick marks through both added and changed predicates, asserts zero match counters, and verifies that diagnostic readback stays at zero matches while `scanned_marks` records the inspected marks. Existing tests cover newly added components in `Changed<T>` filters, wrapping tick comparison, stale tick clamping, component removal records, and direct hot-path branches in resource and world change-detection helpers.

The focused scan-stat test passed under the Runtime 07 `core-min` validation target. Broader ECS and full runtime package validation remain milestone testing-stage work because this slice only adds the local counter projection and does not yet wire automatic frame-level collection.
