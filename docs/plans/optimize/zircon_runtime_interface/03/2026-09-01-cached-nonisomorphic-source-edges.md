record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_source_edge_cache_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/performance_tests.rs::runtime_interface03_batch16_cached_source_edges_release_benchmark
---

# Cached nonisomorphic source edges

## Scope

Complex visual clusters can share one nonisomorphic source range. Every caret edge query previously
rescanned all clusters to recompute that range's minimum and maximum visual offsets.

Unordered or overlapping maps now retain two zero-cache linear queries, then lazily build one
source-range-to-visual-bounds index for repeated queries. Ordered disjoint maps do not allocate the
cache, and isomorphic clusters keep their direct edge projection. Directional edge selection and
the retained linear bounds oracle remain unchanged.

## Verification

- TDD RED: the focused contract found no lazy source-edge cache, linear threshold, or benchmark.
- Focused caret/source-span/source-edge static contracts after implementation: `6/6` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `53/53`
  passed (`41 + 9 + 3`).
- Rust behavior coverage verifies cached and linear bounds across 65,536 clusters sharing a
  nonisomorphic source range and proves that the cache is initialized only after the threshold.
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

The ignored release benchmark resolves one repeated nonisomorphic source edge 64 times over 11
alternating samples in a 65,536-cluster line. It compares the retained O(n) bounds scan with the
warm O(1) hash lookup and requires at least 90% P95 improvement. Terminal nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.
