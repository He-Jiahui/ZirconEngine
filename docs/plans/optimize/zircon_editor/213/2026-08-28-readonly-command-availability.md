# Editor213 Readonly Command Availability

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime267-editor213-performance-batch-20260828hu-v1`

## Problem

Every UI asset pane projection cloned the complete authoring document seven times to test move,
reparent, wrap, and unwrap command availability by mutating temporary copies. The copies were
discarded immediately and repeated on every presentation rebuild.

## Optimization

- Resolve move availability from the selected child index and parent child count.
- Resolve sibling reparent and outdent availability from adjacent-node and ancestor structure.
- Resolve wrap and unwrap availability from parent mount and child-count invariants.
- Preserve diagnostics gating and all other command projection behavior.

## Regression Contract

The `optimization_batch_20260828hu_` Editor tests compare every readonly result with the legacy
clone-and-edit algorithm across root, sibling, container, nested, missing, and empty selections.
They enforce a clone-free production query and provide an ignored paired release benchmark
emitting `EDITOR213_READONLY_COMMAND_AVAILABILITY_BENCH_V1`. It evaluates a 512-node document with
4 KiB text values eight times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
