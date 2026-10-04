---
title: Runtime624 Preallocated Volume Parameter Validation
category: zircon_runtime
report_id: Runtime624-preallocated-volume-param-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime624 Preallocated Volume Parameter Validation

Post-process volume descriptor validation now uses a borrowed `HashSet<&str>` reserved from the
known parameter count. The previous `BTreeSet` paid logarithmic insertion cost while validation
only needs first-seen membership.

Descriptor order, empty component and parameter diagnostics, and the first duplicate parameter
error remain unchanged. Focused source coverage locks preallocation and borrowed insertion, while
existing registry tests continue to cover the public validation behavior.

The ignored Windows Release benchmark emits `RUNTIME624_PREALLOCATED_VOLUME_PARAM_BENCH_V1` over
17 alternating sample pairs with 32,768 unique long parameter names. The gate requires the
preallocated hash-membership P95 to be at most 40% of the ordered-tree P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime624/Editor624 Windows
Release compile, focused regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both performance gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime624 is prepared with Editor624 under request
`runtime624-editor624-volume-material-membership-performance-20260901in-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
