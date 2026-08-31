# Editor171 Status Chip Clip Allocation Gate

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime225-editor171-performance-batch-20260826gd-v1`

## Problem

Editor status-chip text split a label into owned label/value strings before discovering that the
chip text base was completely outside the active clip and could not emit a command.

## Optimization

- Reject a valid base frame immediately when it does not intersect the clip, before text splitting.
- Retain the per-command frame/clip check as a local invariant for label and value subframes.
- Preserve status parsing, label/value alignment, colors, order, typography, clipping, opacity, and
  all visible command output.

## Regression Contract

The `optimization_batch_20260826gd_` Editor tests cover disjoint-clip behavior and enforce the clip
gate before text splitting, and provide an ignored paired release benchmark emitting
`EDITOR171_STATUS_CHIP_CLIP_ALLOCATION_GATE_BENCH_V1`. It rejects 8,192 split labels of 4,096 bytes
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
