# Runtime266 Borrowed Project Manifest Save

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime266-editor212-performance-batch-20260828ht-v1`

## Problem

Project manifest save cloned the complete manifest solely to replace `format_version` before TOML
serialization. Large UI root lists and other owned project metadata were copied even though the
serializer only needed read access to those values.

## Optimization

- Serialize the current manifest format through a borrowed payload view.
- Preserve derived field order and every existing optional or empty-field omission rule.
- Keep validation, pretty TOML output, and atomic file replacement unchanged.

## Regression Contract

The `optimization_batch_20260828ht_` Runtime tests require byte-for-byte parity with the legacy
clone-based TOML, preserve the decode round trip, enforce the borrowed save path, and provide an
ignored paired release benchmark emitting `RUNTIME266_BORROWED_PROJECT_MANIFEST_SAVE_BENCH_V1`.
It serializes 512 UI roots with 4 KiB URI tails eight times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
