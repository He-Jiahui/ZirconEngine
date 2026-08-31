---
title: Editor382 Allocation-Free Artifact Extension
category: zircon_editor
report_id: Editor382-allocation-free-artifact-extension-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor382 Allocation-Free Artifact Extension

Native dynamic export staging now classifies library and debug artifact extensions without creating
a lowercase `String`. The classifier first dispatches on the only valid extension lengths, then
performs ASCII case-insensitive comparisons against the possible values for that length.

Recognition of `dll`, `so`, `dylib`, `pdb`, `dbg`, and `dsym`, mixed-case behavior, rejection of
unrelated extensions, and rejection of paths without extensions remain unchanged. Regression
coverage compares all supported families and negative cases with the former lowercase-and-match
implementation.

The ignored Windows Release benchmark emits
`EDITOR382_ALLOCATION_FREE_ARTIFACT_EXTENSION_BENCH_V1` over 17 alternating paired samples. Each
sample performs 262,144 mixed-case `.DSYM` checks. The legacy path creates one lowercase string per
check; the optimized path performs one length-selected comparison and allocates nothing. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.55`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass. This change is inside the Editor export workflow and does
not modify the deferred tooling layer.

## Current batched validation handoff (2026-08-31)

Editor382 is prepared with Runtime452 under request
`runtime452-editor382-performance-batch-20260831et-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
