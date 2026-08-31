# Editor212 Owned Wrap And Unwrap Document Handoff

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime266-editor212-performance-batch-20260828ht-v1`

## Problem

Node wrap and unwrap each cloned the complete UI asset document to create an editable copy, then
cloned that edited document again immediately before command handoff. Large widget trees, imports,
stylesheets, and property values were therefore copied twice per edit.

## Optimization

- Move the already-owned wrapped document into command handoff.
- Move the already-owned unwrapped document into command handoff.
- Preserve selection projection, structured tree-edit metadata, serialization, undo, and replay.

## Regression Contract

The `optimization_batch_20260828ht_` Editor tests preserve wrap followed by unwrap behavior,
enforce single-clone document preparation in both production branches, and provide an ignored
paired release benchmark emitting `EDITOR212_OWNED_WRAP_DOCUMENT_HANDOFF_BENCH_V1`. It prepares
documents with 512 widget imports carrying 4 KiB paths eight times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
