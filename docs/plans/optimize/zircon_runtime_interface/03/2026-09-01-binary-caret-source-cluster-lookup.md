record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_caret_source_cluster_lookup_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/performance_tests.rs::runtime_interface03_batch14_binary_caret_source_lookup_release_benchmark
---

# Binary caret source-cluster lookup

## Scope

`UiTextLineSourceMap::visual_offset_for_caret` previously scanned every visual cluster to resolve
an exact or interior source offset and scanned again for boundary fallbacks. Typical LTR lines have
source ranges in visual order and do not overlap, making these repeated scans unnecessary.

Source-map construction now records whether source ranges are ordered and disjoint. Only those
maps use affinity-specific binary partitioning and unique-cluster visual edges; overlapping,
reordered, bidi, or aggregate-cluster maps retain the original linear algorithm. Exact upstream
and downstream edges, interior affinity, line fallbacks, and public results remain unchanged.

## Verification

- TDD RED: the focused contract found no source-order qualification, binary helper, or benchmark.
- Focused visual-cluster/caret-source static contracts after implementation: `4/4` passed
  (`2 + 2`).
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `58/58`
  passed (`37 + 9 + 12`).
- Rust behavior coverage compares binary and linear lookup at the first, interior, terminal, and
  after-cluster boundaries for both affinities.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending.

## Performance contract

The ignored release benchmark resolves a caret near the end of 65,536 disjoint source clusters 64
times over 11 alternating samples. It compares the retained linear oracle with binary lookup and
requires at least 90% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
