# Editor223 Reused Native Window Presentation Strings

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime277-editor223-performance-batch-20260828ie-v1`

## Problem

Editor native floating-window presentation replaced five owned string fields with fresh clones on
every presentation update. Bounds-only movement therefore reallocated two window IDs, two surface
tree IDs, and the title even though their values commonly remained stable.

## Optimization

- Update both presentation domains through one in-place helper.
- Use `String::clone_from` for the shell and surface IDs and the shell title.
- Reuse existing string capacities across bounds-only and same-sized identity updates.
- Preserve native mode, duplicated shell/surface identities, bounds, and presentation-match logic.

## Regression Contract

The `optimization_batch_20260828ie_` Editor tests prove allocation identity for all five string
fields and require the production helper to use `clone_from`. The ignored paired release benchmark
emits `EDITOR223_REUSED_NATIVE_PRESENTATION_STRINGS_BENCH_V1`. It performs 2,048 bounds-changing
updates with approximately 8-KiB IDs and titles per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
