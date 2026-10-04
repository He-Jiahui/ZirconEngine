record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/caret_performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_caret_line_lookup_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/caret_performance_tests.rs::runtime_interface03_batch11_binary_caret_line_lookup_release_benchmark
---

# Binary caret-line lookup

## Scope

Caret decoration previously searched every resolved line from the front for upstream affinity and
from the back for downstream affinity. Large text layouts therefore made each caret update O(n)
even though resolved line source ranges are published in layout order.

`caret_line` now uses affinity-specific `partition_point` queries: upstream partitions by range
end to preserve the earlier line at a shared boundary, while downstream partitions by range start
to preserve the later line. A candidate mismatch falls back to the original linear algorithm,
retaining outside-range and malformed-gap behavior. Empty layouts still produce no caret frame.

## Verification

- TDD RED: the focused contract found forward/reverse linear scans and no benchmark module.
- Focused caret-line lookup static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `52/52`
  passed before Batch 12, then `54/54` passed for the combined submission scope
  (`33 + 9 + 12`).
- Rust behavior coverage compares binary and linear results across both affinities, shared
  boundaries, before/after offsets, and empty layouts.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission:

- snapshot: `2719`;
- snapshot request: `83e34d1b354b4bad802771a7b1ec5c8c`;
- attribution request: `aaa99a7ec08c477abbf7cb6852b6c8e5`;
- submit request: `b39ebeaed7e947c5a6adb303c33dda2c`;
- validation ticket: `18fe3eeedc80475db90705cf50584969`;
- source manifest: `b06a421159e6909b6491672ace1f2f5fd2b6d1aba949ab7ccbe7392370eaf6ee`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch1 -- --include-ignored --nocapture`;
- submitted state: `queued` (isolated snapshot contains batch11/12 only; asynchronous and
  intentionally not polled).

## Performance contract

The ignored release benchmark performs 64 caret lookups near the end of a 65,536-line layout over
11 alternating samples. It compares the retained linear oracle with binary lookup and requires at
least 90% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
