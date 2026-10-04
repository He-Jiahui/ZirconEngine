record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_source_span_cluster_range_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/performance_tests.rs::runtime_interface03_batch15_binary_source_span_range_release_benchmark
---

# Binary source-span cluster range

## Scope

`UiTextLineSourceMap::visual_spans_for_source_range` previously filtered every visual cluster for
every source selection, even when the map had already proven that source ranges were ordered and
disjoint.

Qualified maps now use two `partition_point` lookups to borrow only the intersecting cluster
slice before projecting visual spans. Overlapping, reordered, bidi, and aggregate source maps
retain the original full-filter path. Empty, boundary, interior, terminal, and out-of-line ranges
preserve the previous projection.

## Verification

- TDD RED: the focused contract found no binary source range helper or benchmark.
- Focused caret/source-span/source-edge static contracts after implementation: `6/6` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `53/53`
  passed (`41 + 9 + 3`).
- Rust behavior coverage compares binary and retained linear projections across empty, first,
  middle, last, and outside ranges.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission:

- attribution request `112389fbfa2e4e6bbb91ba4d19cfa69b`;
- snapshot `2727`, create request `ae6abb8d130b4381a2d8ddff9be5b768`;
- validation idempotency request `c7f231ff3e1b4ed1a5d7edda991d4fd2`;
- the submit client timed out before returning a ticket receipt; status is intentionally not polled
  while the coordinator continues asynchronously.

## Performance contract

The ignored release benchmark projects a one-cluster source range near the end of 65,536 ordered
clusters 64 times over 11 alternating samples. It compares the retained O(n) filter with the O(log
n + k) slice projection and requires at least 90% P95 improvement. Terminal nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.
