# Editor196 Project Title Direct Split

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime250-editor196-performance-batch-20260826hd-v1`

## Problem

Workbench title publication copied the complete displayed project path and replaced every Windows
separator only to select the last nonempty path segment. Long project roots paid a full-path
allocation before allocating the much shorter title.

## Optimization

- Trim both separator kinds directly on the displayed path.
- Reverse-split on `/` and `\\` without creating a normalized path copy.
- Preserve the original displayed path fallback for separator-only roots.

## Regression Contract

The `optimization_batch_20260826hd_` Editor tests preserve Windows, Unix, mixed-separator,
trailing-separator, and root fallback behavior; enforce replacement-free splitting; and provide an
ignored paired release benchmark emitting `EDITOR196_PROJECT_TITLE_DIRECT_SPLIT_BENCH_V1`. It
repeatedly extracts a title from a long displayed path and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
