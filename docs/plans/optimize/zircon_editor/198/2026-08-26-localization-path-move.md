# Editor198 Localization Path Move

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime252-editor198-performance-batch-20260826hf-v1`

## Problem

Localization diagnostic projection moved the input path into a temporary string, cloned the whole
path into the editor diagnostic, and then borrowed the temporary to derive the target node. Long
localization paths therefore paid an avoidable full-string allocation and copy on every projection.

## Optimization

- Derive the optional target node while the input diagnostic still owns its path.
- Move the original path directly into `UiAssetEditorDiagnostic`.
- Preserve code normalization, severity, message, source path, and target node semantics.

## Regression Contract

The `optimization_batch_20260826hf_` Editor tests preserve normalized and custom codes, severity,
message, path, and node projection; enforce path movement without `source_path.clone()`; and provide
an ignored paired release benchmark emitting `EDITOR198_LOCALIZATION_PATH_MOVE_BENCH_V1`. It
repeatedly projects a diagnostic with a 64 KiB source path and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
