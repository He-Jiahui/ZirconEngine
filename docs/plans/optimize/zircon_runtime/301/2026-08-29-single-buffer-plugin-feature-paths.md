# Runtime301 Single-Buffer Plugin Feature Paths

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime301-editor247-performance-batch-20260829ab-v1`

## Problem

Default feature crate names and local feature runtime paths first sanitized the feature ID into a
temporary `String` and then formatted that value into the final path. Each lookup allocated and
copied the sanitized feature identifier twice.

## Optimization

- Reserve the final crate-name or local runtime-path buffer from fixed segments and input lengths.
- Sanitize feature characters directly into the final result, retaining lowercase ASCII rules and
  underscore replacement for unsupported characters.
- Preserve explicit runtime crate overrides and external provider package paths.

## Regression Contract

The `optimization_batch_20260829ab_` Runtime tests cover crate names, local paths, Unicode
sanitization, explicit overrides, and external providers and guard removal of the intermediate stem.
The ignored paired release benchmark emits `RUNTIME301_SINGLE_BUFFER_PLUGIN_FEATURE_PATHS_BENCH_V1`.
It performs 100,000 feature crate-name resolutions per sample, reduces result allocations per name
from two to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
