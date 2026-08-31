# Runtime146 Depth Backend Borrowed Match

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime146-editor92-performance-batch-20260826dc-v1`

## Problem

Post-process depth resource construction lowercased the complete graphics backend name into a new
`String` before checking the fixed `gl` and `angle` tokens. Resource reconstruction therefore paid
an avoidable allocation for a read-only classification.

## Optimization

- Search borrowed backend bytes with ASCII case-insensitive windows.
- Preserve OpenGL, WebGL, and ANGLE fallback selection across mixed case.
- Preserve raw-depth selection for Vulkan, Direct3D, Metal, software, and unknown backends.

## Regression Contract

The shared `optimization_batch_20260826dc_` filter owns three Runtime tests: legacy-result parity,
allocation-free source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME146_DEPTH_BACKEND_BORROWED_MATCH_BENCH_V1`, performs 262,144 classifications per sample,
records lowercase allocations from 262,144 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
