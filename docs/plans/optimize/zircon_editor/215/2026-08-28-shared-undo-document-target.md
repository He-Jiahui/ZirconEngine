# Editor215 Shared Undo Document Target

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime269-editor215-performance-batch-20260828hw-v1`

## Problem

Each UI asset undo or redo cloned its transition while retaining the original entry on the opposite
stack. `UiAssetDocumentDiff` stored its immutable target document inline, so that transition clone
deeply copied the entire recursive authoring document before replay copied it again into the live
document.

## Optimization

- Store the immutable diff target in `Arc<UiAssetDocument>`.
- Make undo/redo transition clones share the target allocation in O(1).
- Preserve equality, unchanged detection, and the independent deep copy required when applying a
  target into the mutable live document.

## Regression Contract

The `optimization_batch_20260828hw_` Editor tests prove that cloned diffs share the exact target
allocation while applying an equal independent document. The source contract enforces the Arc
target and explicit apply clone. The ignored paired release benchmark emits
`EDITOR215_SHARED_UNDO_DOCUMENT_TARGET_BENCH_V1`. It clones a diff targeting a 1,024-property
document with 4 KiB values 128 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
