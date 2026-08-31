# Editor247 Single-Buffer Folder-Picker Quoting

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime301-editor247-performance-batch-20260829ab-v1`

## Problem

Folder-picker command quoting escaped a path into a temporary string and then formatted the outer
quotes into another allocation. The macOS variant chained two replacement allocations before its
final formatting allocation.

## Optimization

- Count escapable quote characters and reserve the exact final Windows or macOS string capacity.
- Write outer quotes, ordinary Unicode characters, and platform escape sequences into one buffer.
- Preserve PowerShell doubled apostrophes and AppleScript backslash/double-quote escaping.

## Regression Contract

The `optimization_batch_20260829ab_` Editor tests compare platform quoting bytes against the legacy
replacement chains and guard both helpers against `replace` and `format!`. The ignored Windows
paired release benchmark emits `EDITOR247_SINGLE_BUFFER_FOLDER_PICKER_QUOTING_BENCH_V1`. It
performs 100,000 PowerShell path quotes per sample, reduces result allocations per quote from two to
one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
