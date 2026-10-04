record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/cluster.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/cluster/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_visual_cluster_run_cursor_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/cluster/performance_tests.rs::runtime_interface03_batch13_visual_cluster_run_cursor_release_benchmark
---

# Cursor-based visual-cluster run projection

## Scope

`visual_source_clusters` previously scanned every resolved run for every grapheme. Dense rich-text
lines with one run per grapheme therefore projected source ownership in O(graphemes x runs).

When run visual starts are monotonic, projection now advances a `run_start` cursor past completed
runs and stops candidate scanning once the next run starts after the grapheme. Nonmonotonic input
falls back to the retained full-scan projection. Combining graphemes spanning multiple runs, bidi
direction, isomorphic source edges, cluster order, and output fields are unchanged.

## Verification

- TDD RED: the focused contract found no run-order check, cursor, or benchmark module.
- Focused visual-cluster cursor static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `58/58`
  passed (`37 + 9 + 12`).
- Rust behavior coverage compares cursor and linear projection for dense ordered runs and verifies
  an intentionally reversed run list takes the equality-preserving fallback.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending.

## Performance contract

The ignored release benchmark projects 2,048 single-grapheme runs four times over 11 alternating
samples. It compares the retained O(graphemes x runs) oracle with the monotonic cursor and requires
at least 90% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
