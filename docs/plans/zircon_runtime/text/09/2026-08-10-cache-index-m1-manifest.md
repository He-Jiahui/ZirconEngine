Plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
Milestone: M1
Status: implementation_complete_second_review_complete_validation_pending
Files: ["zircon_runtime/src/text/cache/frame_dedup.rs", "zircon_runtime/src/text/cache/index.rs", "zircon_runtime/src/text/cache/layout_cache.rs", "zircon_runtime/src/text/cache/measure_cache.rs", "zircon_runtime/src/text/cache/shaped_cache.rs", "zircon_runtime/src/text/cache/tests.rs", "zircon_runtime/src/ui/text/measure_cache.rs", "docs/plans/zircon_runtime/text/09/failure-2026-07-18-text-cache-linear-lookup-and-eviction.md"]

# Text09 M1 Indexed Cache Completion Manifest

## Scope Delivered

The frame, measure, layout, and shaped-run caches retain exact-text collision checks while using
bucket-local lookup, stable entry slots, and linked LRU order. Oldest eviction now returns the
actual victim work to cache reports: one directly selected LRU-head candidate and zero resident
entry moves. Measure, layout, and shaped capacity trims accumulate those values instead of leaving
the reporting fields at their default values.

Borrowed frame/measure/layout convenience lookups allocate an `Arc<str>` only on insertion. The
production UI measure and layout paths also clone the persistent entry's existing text owner into
the frame dedup on a cross-frame hit, so they do not copy source text before establishing whether
the persistent cache already contains the exact key and text.

## Fresh Testing Evidence

The deterministic cache matrix covers 16, 256, 1024, 2048, and 4096 resident entries. At every
size it performs an exact hit, absent miss, exact update, and capacity eviction for all three
persistent caches, while frame dedup covers hit/miss/update and whole-frame clear. The matrix
requires bucket-local lookup candidates, one direct victim check per eviction, and zero stable
entry moves. Additional tests preserve stored-text identity across persistent hits.

Scoped Rustfmt parse/check and `git diff --check` pass apart from existing line-ending notices.
Managed Windows Cargo and product trace execution remain pending coordinator validation.

## Review

An independent read-only second review followed lookup, update, trim, collision, width-validity,
direction-alias, and UI owner-reuse paths in the current worktree and found no P0/P1/P2. This
source review is not a runtime performance result and does not close the linked Failure before
the coordinator records the required validation evidence.
