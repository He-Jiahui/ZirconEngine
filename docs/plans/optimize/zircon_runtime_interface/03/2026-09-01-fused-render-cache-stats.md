record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/cache/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_render_debug_cache_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/cache/performance_tests.rs::runtime_interface03_batch8_fused_render_cache_stats_release_benchmark
---

# Fused render-cache statistics

## Scope

`UiRenderCachePlan::from_paint_elements_and_batches` previously built paint and batch cache-entry
vectors, then scanned both complete vectors again to count reused entries. The second traversal
made cache-plan construction perform avoidable memory reads proportional to paint and batch count.

The entry builders now count reused statuses while each entry is constructed. Total and rebuilt
counts are derived from the completed vector lengths, removing both post-construction scans while
preserving every entry, status, reason, and public statistic.

## Verification

- TDD RED: the focused contract found the post-construction `UiRenderCacheStats::from_entries`
  scans and no fused counters.
- Focused render debug/cache static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `44/44`
  passed (`23 + 9 + 12`).
- Rust behavior coverage compares all generated entries and the reused count with the prior scan.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2702`;
- snapshot request: `b1062211b55a4e0ba69dc283c9a505b8`;
- attribution request: `04d054b019854a2f8edcbadaf13cd23f`;
- submit request: `e97c7f16f6c2457092a36b5b6e9789b6`;
- coordinator request: `8cc11b631c96486abd1b03e05812ba7f`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch8 -- --include-ignored --nocapture`;
- submitted state: accepted; the response timed out after submission and ticket reconciliation is
  intentionally deferred while independent work continues.

## Performance contract

The ignored release benchmark constructs 131,072 paint cache entries over 11 alternating samples,
comparing post-construction scanning with fused counting. The P95 gate requires at least 10%
improvement. Terminal P50/P95 nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.
