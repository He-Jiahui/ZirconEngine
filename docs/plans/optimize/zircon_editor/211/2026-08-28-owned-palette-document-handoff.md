# Editor211 Owned Palette Document Handoff

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime265-editor211-performance-batch-20260828hs-v1`

## Problem

Palette insertion and node outdent each cloned the complete UI asset document to create an editable
copy, then cloned that edited document again immediately before handing it to the command pipeline.
Large widget trees, stylesheets, imports, and property values were therefore copied twice per edit.

## Optimization

- Move the already-owned edited document into palette insertion command handoff.
- Move the already-owned edited document into node reparent command handoff.
- Preserve selection projection, structured tree-edit metadata, serialization, undo, and replay.

## Regression Contract

The `optimization_batch_20260828hs_` Editor tests preserve palette insertion and node outdent
behavior, enforce clone-free document handoff in both production branches, and provide an ignored
paired release benchmark emitting `EDITOR211_OWNED_PALETTE_DOCUMENT_HANDOFF_BENCH_V1`. It prepares
documents with 512 widget imports carrying 4 KiB paths eight times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
