record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_text_line_metrics_index_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_shape/performance_tests.rs::runtime_interface03_batch10_indexed_shaped_text_run_geometry_release_benchmark
---

# Indexed shaped-text run geometry

## Scope

The shaped-text projection previously recomputed grapheme boundaries and glyph-prefix geometry for
every run in a line. Dense rich text therefore rescanned the same line once per run. Multi-cluster
lines now build one local `ShapedTextLineMetrics` index containing grapheme byte boundaries and,
when complete glyph data is available, sanitized advance prefixes. Each run reuses that index;
single-cluster lines retain the existing allocation-free fast path.

Run ordering, grapheme floor/ceil behavior, glyph/fallback geometry, and serialized fields are
unchanged. The index is local to one projection and is dropped immediately after the line.

## Verification

- TDD RED: the focused contract found no line metrics index or indexed geometry helper.
- Focused text projection/index static contracts after implementation: `6/6` passed (`3 + 3`).
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `50/50`
  passed (`29 + 9 + 12`).
- Rust behavior coverage compares indexed shaped projections with the prior per-run geometry path.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission:

- snapshot: `2711`;
- snapshot request: `7859f51441d7436d9e949792c7a6fc22`;
- attribution request: `f5036fb4bb064e7796820b129d7a6c3f`;
- submit request: `afb049c341f44003a674ca1f0905c112`;
- validation ticket: `8a95d1a407c4447f9e948a2656ec28f1`;
- source manifest: `44b86be514ba0e0f0b6b93b2a3d9c61c801755514f0ce71e8d7597427c9ddffa`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch10 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark projects 128 runs in one shaped line over 512 iterations and 11
alternating samples. The P95 gate requires at least 30% improvement over repeated grapheme and
glyph scans. Terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.
