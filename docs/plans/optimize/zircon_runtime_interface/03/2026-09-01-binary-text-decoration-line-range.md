record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/decoration_performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_text_decoration_line_range_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/decoration_performance_tests.rs::runtime_interface03_batch12_binary_decoration_line_range_release_benchmark
---

# Binary text-decoration line range

## Scope

Selection and IME range decorations previously visited every resolved line for every decoration,
then rejected nonintersecting lines before creating a source map. A localized clause in a large
document therefore paid O(lines) range checks despite ordered line source ranges.

The projection now computes the half-open intersecting line interval with two `partition_point`
queries and visits only that interval when line source ranges are monotonic. Interface DTOs do not
declare that ordering at the type boundary, so the cache performs one order check and uses a linear
intersection fallback for unordered lines. Decoration declaration order, line order, source-map
reuse, span order, half-open intersection semantics, and emitted geometry remain unchanged. The
same transient map cache is also handed to caret projection when range decorations are present,
avoiding a second projection for a caret on an already-touched line.

## Verification

- TDD RED: the focused contract found the full `0..lines.len()` scan and no benchmark module.
- Focused caret/decoration range static contracts after implementation: `4/4` passed (`2 + 2`).
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `54/54`
  passed (`33 + 9 + 12`).
- Rust behavior coverage compares binary and linear intersections for empty, localized, multiline,
  terminal, and outside ranges, and swaps line records to verify the unordered fallback. A shared
  range/caret regression confirms one source-map initialization for a touched line.
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

The ignored release benchmark resolves a localized decoration near the end of 65,536 ordered lines
64 times over 11 alternating samples. It compares the retained linear oracle with binary range
lookup and requires at least 90% P95 improvement. The unordered defensive path is a correctness
fallback and is excluded from this fast-path timing gate. Terminal nanosecond values must come from
the managed Windows receipt before integration, push, or WeCom reporting.
