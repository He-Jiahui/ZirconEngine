---
title: Editor633 Preallocated Widget Dependency Closure
category: zircon_editor
report_id: Editor633-preallocated-widget-dependency-closure-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor633 Preallocated Widget Dependency Closure

External-widget promotion now reserves its visited component set and pending breadth-first queue
from the local component-map size. The previous closure traversal grew both collections from zero.

The root is still processed first, duplicate dependencies remain suppressed by the same hash set,
missing component references still return `None`, and the later ordered projection of component
definitions remains unchanged.

The ignored Windows Release harness emits
`EDITOR633_PREALLOCATED_WIDGET_DEPENDENCY_CLOSURE_BENCH_V2` over 31 alternating sample pairs with
65,536 component visits. It reports nearest-rank p50/p95/p99. This is a helper microbenchmark of a
synthetic queue and set; it does not invoke `promote_selected_component_to_external_widget` or
construct a real UI asset document.

Managed Windows Release evidence through the real caller (`promote_selected_component_to_external_widget`) remains pending. That workload
must cover component lookup, dependency traversal, external-widget construction, and source-document
mutation. The helper microbenchmark is not product acceptance.

No direct Cargo validation was run. The coordinator owns combined Runtime633/Editor633 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor633 is prepared with Runtime633 under request
`runtime633-editor633-meta-widget-capacity-performance-20260901iw-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
