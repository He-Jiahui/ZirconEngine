---
title: Runtime445 Preallocated IBL Cache Path
category: zircon_runtime
report_id: Runtime445-preallocated-ibl-cache-path-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime445 Preallocated IBL Cache Path

Runtime IBL cache path construction now retains one cloned root `PathBuf`, reserves the complete
known suffix capacity, and appends the cache directory, algorithm version, request identity, and
artifact file name in place. The former four-element `join` chain allocated a new path buffer and
copied the growing prefix at every step.

The cache directory, version encoding, request identity hash, face and mip formatting, extension,
and returned path remain unchanged. Regression coverage compares the optimized path with the former
layout and requires the single reserved buffer plus sequential pushes.

The ignored Windows Release benchmark emits `RUNTIME445_PREALLOCATED_IBL_CACHE_PATH_BENCH_V1` over
17 alternating paired samples. Each sample constructs 2,048 paths beneath a long cache root while
both models retain the production request hashing and component formatting. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.80`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime445 is prepared with Editor373 under request
`runtime445-editor373-performance-batch-20260831ek-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
