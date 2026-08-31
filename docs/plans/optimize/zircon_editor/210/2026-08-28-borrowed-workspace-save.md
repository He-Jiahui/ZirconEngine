# Editor210 Borrowed Workspace Save

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime264-editor210-performance-batch-20260828hr-v1`

## Problem

Saving the editor workspace deep-cloned the complete `ProjectEditorWorkspace` into a temporary
document before JSON serialization. Large open-view payloads, layout strings, and view state were
therefore copied once solely to produce bytes and immediately discarded.

## Optimization

- Serialize a borrowed workspace document with the same field names and ordering.
- Preserve the owned workspace document for loading, diagnostics, and round-trip compatibility.
- Keep atomic persistence and missing-workspace behavior unchanged.

## Regression Contract

The `optimization_batch_20260828hr_` Editor tests require byte-for-byte pretty-JSON parity with the
legacy owned document, preserve the owned decode round trip, enforce removal of `workspace.clone()`
from save, and provide an ignored paired release benchmark emitting
`EDITOR210_BORROWED_WORKSPACE_SAVE_BENCH_V1`. It serializes 512 open views with 4 KiB JSON payloads
eight times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
