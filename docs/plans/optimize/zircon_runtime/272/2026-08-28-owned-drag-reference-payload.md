# Runtime272 Owned Drag Reference Payload

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime272-editor218-performance-batch-20260828hz-v1`

## Problem

Runtime reference-drop handling received an owned drag payload but cloned the complete optional
source metadata before moving the payload reference into component state. Asset-browser metadata can
carry seven independently allocated strings, so every accepted sourced drop duplicated all of them
only to discard the original payload immediately afterward.

## Optimization

- Validate the payload kind before consuming it, preserving rejected-drop diagnostics.
- Split the accepted owned payload into kind, reference, and source metadata.
- Move both reference and source allocations directly into component state.
- Preserve asset/instance value routing, source removal, property ownership, and validation behavior.

## Regression Contract

The `optimization_batch_20260828hz_` Runtime tests prove reference and metadata allocation identity
across payload splitting and prevent the accepted-drop path from restoring the source clone. The
ignored paired release benchmark emits `RUNTIME272_OWNED_DRAG_REFERENCE_PAYLOAD_BENCH_V1`. It splits
128 payloads per sample, each carrying one 8-KiB reference and seven 8-KiB source fields, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
