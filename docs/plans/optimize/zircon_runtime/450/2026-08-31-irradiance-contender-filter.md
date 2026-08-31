---
title: Runtime450 Irradiance Contender Filter
category: zircon_runtime
report_id: Runtime450-irradiance-contender-filter-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime450 Irradiance Contender Filter

Per-view irradiance volume selection now rejects candidates that rank below the current winner
before computing their matrix determinant or testing every visible world position. A lower-ranked
candidate cannot affect the final selection, so the former geometry work was discarded after the
final `max_by` comparison.

Intensity and layer admission, finite and non-degenerate transform requirements, visible-position
containment, priority ordering, lower-ID tie breaking, and the legacy last-equal-candidate rule all
remain unchanged. Regression coverage compares the former filter-and-max model with the contender
loop, including invalid higher-ranked and exactly tied candidates.

The ignored Windows Release benchmark emits `RUNTIME450_IRRADIANCE_CONTENDER_FILTER_BENCH_V1`
over 17 alternating paired samples. Each sample performs 128 selections across 256 volumes and 16
visible positions, with the winning volume first. The legacy path runs expensive geometry checks
for all 256 candidates per selection; the optimized path runs them for one. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.20`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime450 is prepared with Editor380 under request
`runtime450-editor380-performance-batch-20260831er-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
