# Editor218 Borrowed Preview Node ID

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime272-editor218-performance-batch-20260828hz-v1`

## Problem

Every Editor UI preview interaction cloned the selected node ID before performing read-only document
and binding lookup. The dispatch builder already accepts a borrowed node ID and creates the single
owned value required by the returned dispatch, making the preliminary clone redundant.

## Optimization

- Borrow the selected node ID from the selection model through `Option::as_deref`.
- Reuse that borrow for document lookup and dispatch construction.
- Preserve invalid-index errors, missing-binding clearing, tool-mode changes, and returned dispatch
  ownership.

## Regression Contract

The `optimization_batch_20260828hz_` Editor tests prove the selected node helper returns the original
string allocation and prevent the preview-dispatch path from cloning `primary_node_id`. The ignored
paired release benchmark emits `EDITOR218_BORROWED_PREVIEW_NODE_ID_BENCH_V1`. It resolves a 80-KiB
node ID 512 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
