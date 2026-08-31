# Editor220 In-Place Profile Session ID Merge

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime274-editor220-performance-batch-20260828ib-v1`

## Problem

Editor profile-snapshot merge allocated a fresh session-ID string even when the Editor and Runtime
session IDs were equal. Different-session merges also built a replacement string rather than
reusing the Editor snapshot's existing buffer.

## Optimization

- Retain the Editor session-ID buffer unchanged when both snapshot IDs are equal.
- Reserve once and append the separator and Runtime ID for different sessions.
- Preserve empty-profile replacement, span remapping, activity flags, samples, and retention data.
- Keep the existing repeated-merge session-ID semantics without allocating an intermediate string.

## Regression Contract

The `optimization_batch_20260828ib_` Editor tests prove buffer identity for equal and pre-reserved
different session IDs and prevent assignment-based merging from returning. The ignored paired
release benchmark emits `EDITOR220_IN_PLACE_PROFILE_SESSION_ID_BENCH_V1`. It performs 512 equal-ID
merges with 64-KiB session IDs per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
