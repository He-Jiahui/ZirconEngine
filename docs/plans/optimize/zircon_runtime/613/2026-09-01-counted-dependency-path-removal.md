---
title: Runtime613 Counted Dependency Path Removal
category: zircon_runtime
report_id: Runtime613-counted-dependency-path-removal-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime613 Counted Dependency Path Removal

Targeted asset replacement receives a list of dependency paths to remove from an owner's ordered
path vector. The old loop searched from the beginning and physically removed one element for every
requested path, making duplicate-heavy updates repeatedly shift the remaining vector and turning
large edits into quadratic work.

The new helper counts requested removals in a preallocated `HashMap<AssetUri, usize>` and performs
one stable `retain` pass. Duplicate requests still remove exactly that many occurrences, unknown
paths remain harmless, and surviving paths preserve their original order. Dependency additions,
owner refresh, unresolved diagnostics, and registry output order are unchanged.

The ignored Windows Release benchmark emits `RUNTIME613_COUNTED_DEPENDENCY_PATH_REMOVAL_BENCH_V1`
over 17 alternating sample pairs, 4,096 source paths, and 2,048 removals. The gate requires the
counted-removal P95 to be at most 25% of repeated position/remove P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regressions, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime613 is prepared with Editor613 under request
`runtime613-editor613-counted-removal-occupancy-performance-20260901id-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
