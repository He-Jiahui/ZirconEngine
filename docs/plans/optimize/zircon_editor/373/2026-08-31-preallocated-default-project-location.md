---
title: Editor373 Preallocated Default Project Location
category: zircon_editor
report_id: Editor373-preallocated-default-project-location-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor373 Preallocated Default Project Location

Editor default Windows project location construction now converts the home directory into one
mutable `PathBuf`, reserves the known `Documents/ZirconProjects` suffix, and appends both components
in place. The former chained `join` calls allocated and copied an intermediate home/Documents path.

The USERPROFILE lookup, fallback order, Documents directory, ZirconProjects directory, and resulting
path remain unchanged. Regression coverage compares the optimized result with the former layout and
requires the reserved single-buffer construction.

The ignored Windows Release benchmark emits
`EDITOR373_PREALLOCATED_DEFAULT_PROJECT_LOCATION_BENCH_V1` over 17 alternating paired samples. Each
sample constructs 4,096 default locations beneath a long home directory. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor373 is prepared with Runtime445 under request
`runtime445-editor373-performance-batch-20260831ek-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
