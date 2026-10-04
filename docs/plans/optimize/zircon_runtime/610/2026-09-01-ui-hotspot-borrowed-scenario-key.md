---
title: Runtime610 UI Hotspot Borrowed Scenario Key
category: zircon_runtime
report_id: Runtime610-ui-hotspot-borrowed-scenario-key-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime610 UI Hotspot Borrowed Scenario Key

UI hotspot aggregation now keys its canonical `BTreeMap` with scenario slices borrowed from the
immutable profile snapshot. The previous entry lookup converted the scenario to a new `String` for
every accepted counter, including repeated metrics for the same scenario. The accumulator still
owns the scenario once in the final `UiScenarioHotspot`, and ordered map iteration preserves the
existing scenario order, report schema, alert behavior, and counter semantics.

A focused source regression requires the borrowed scenario map and entry call while rejecting
`scenario.to_string()` in aggregation. Existing folder-backed aggregation and alert coverage remains
active.

The ignored Windows Release benchmark emits
`RUNTIME610_UI_HOTSPOT_BORROWED_SCENARIO_BENCH_V1` over 17 alternating sample pairs, 16,384
counters, and 128 scenarios. Scenario-key allocations fall from one per accepted counter to zero
inside the index. The gate requires borrowed-key P95 to be at most 45% of legacy owned-key P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime610 is prepared with Editor610 under request
`runtime610-editor610-borrowed-scenario-hash-index-performance-20260901ia-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
