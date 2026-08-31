# Runtime287 Lazy Meta Tag Scope

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime287-editor233-performance-batch-20260828io-v1`

## Problem

Asset metadata validation eagerly formatted an `entries[index]` String for every entry in three
validation passes. Valid metadata discarded every formatted scope, adding avoidable allocator work
to deserialization, current-document validation, and serialized-tag checks.

## Optimization

- Represent entry scopes as a copyable index that implements `Display`.
- Format the exact legacy scope text only when constructing a validation error.
- Preserve root and standalone-entry validation behavior and all typed error payloads.

## Regression Contract

The `optimization_batch_20260828io_` Runtime tests prove exact entry error text and guard all three
entry validation paths against eager formatting. The ignored paired release benchmark emits
`RUNTIME287_LAZY_META_TAG_SCOPE_BENCH_V1`. It performs 32,768 valid empty-tag checks per sample,
removes 32,768 scope String allocations, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
