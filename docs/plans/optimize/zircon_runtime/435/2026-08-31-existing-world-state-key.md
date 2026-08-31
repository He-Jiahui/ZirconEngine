---
title: Runtime435 Existing World State Key Update
category: zircon_runtime
report_id: Runtime435-existing-world-state-key-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime435 Existing World State Key Update

World-space UI state reduction now probes its fixed property keys by borrowed string and overwrites
an existing value in place. Repeated transform and surface updates therefore avoid allocating a new
`String` for each of the nine fixed property writes.

The first write still owns and inserts the property key, and every write still clears any reference
source before replacing the value. Validation errors, numeric clamping, camera-target ownership,
and final component state remain unchanged. Regression tests cover repeated transform replacement
and the borrow-before-own source contract.

The ignored Windows Release benchmark emits `RUNTIME435_EXISTING_WORLD_STATE_KEY_BENCH_V1` over 17
alternating paired samples, each replacing an existing world-state value 65,536 times. The legacy
path allocates 65,536 keys per sample and the optimized path allocates none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime435 is prepared with Editor363 under request
`runtime435-editor363-performance-batch-20260831ea-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
