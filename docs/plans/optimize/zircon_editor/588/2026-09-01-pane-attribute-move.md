---
title: Editor588 Pane Attribute Move
category: zircon_editor
report_id: Editor588-pane-attribute-move-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor588 Pane Attribute Move

Pane projection now moves the materialized payload attribute map directly into the retained root
for payload kinds without a hybrid slot anchor. The prior M2.8 path correctly materialized payload
attributes once, but cloned the complete map for every pane before discovering that most payload
kinds do not publish a second anchor node.

Hybrid hierarchy, animation, module/plugin, build/export, and performance-timeline panes retain the
existing root clone plus anchor-owned map. Root updates still precede component patch application,
and anchor insertion still follows patch application. Non-anchor panes preserve the same root
attributes while avoiding the temporary duplicate map.

The ignored Windows Release benchmark emits `EDITOR588_PANE_ATTRIBUTE_MOVE_BENCH_V1` over 21
alternating sample pairs, 64 retained root attributes, and 8,192 payload attributes per sample. The
legacy model clones one complete attribute map; the optimized model moves it. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor588 is prepared with Runtime588 under request
`runtime588-editor588-dependency-pane-performance-20260901hl-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
