record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/diff.rs
related_tests:
  - tools/tests/test_runtime_interface03_ecs_sorted_diff_performance_contract.py
  - zircon_runtime_interface/src/ui/ecs/diff.rs::runtime_interface03_batch4_linear_sorted_ecs_diff_release_benchmark
---

# Linear sorted ECS projection diff

## Scope

`UiEcsProjectionSnapshot::diff_from` previously built two `BTreeMap` indices on every comparison,
then walked both maps to derive updates/removals followed by additions. Production snapshots are
normally strictly ordered by unique `UiNodeId`, so that work added O(n log n) insertion and two
temporary tree allocations before the actual comparison.

The diff now detects strictly ordered unique node slices and uses two cursor scans. One scan emits
updated/removed nodes in previous-node order; a second emits additions in current-node order,
preserving the existing public change ordering. Unordered or duplicate-ID inputs continue through
the original map semantics, including last-duplicate ownership. The algorithm and its tests live
in `ui/ecs/diff.rs`, reducing `ecs.rs` from 909 to 874 lines.

## Verification

- TDD RED: the static contract found both map constructions in `diff_from` and no diff module.
- Focused sorted-diff contracts after implementation: `2/2` passed.
- Rust behavior tests compare complete ordered-path output with map fallback and prove unordered /
  duplicate inputs retain map semantics.
- `python -m compileall`, scoped Rust 1.94.1 `rustfmt --check`, and scoped `git diff --check`:
  passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2670`;
- snapshot request: `6dbaf1e72c844c1cbfff45fb357d4afa`;
- attribution request: `fb4c476f1e384b5dabe553b6bd7c4cff`;
- submit request: `0cfd4f76f39c4a8fa48f280630c6122e`;
- validation ticket: `6d7426ad3bc245418c0da3a3ed139819`;
- source manifest: `ec180fdd1a78340b5d94fb4f1ec05b00682560f6744bb4fea669f47677c8c421`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1
  runtime_interface03_batch4 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark compares 4,096-node snapshots with 128 removals, 128 additions, and
regular overlapping updates over 11 alternating samples. The P95 gate requires the allocation-free
linear cursor path to be at least 20% faster than the map path. Terminal P50/P95 nanosecond values
must come from the managed Windows receipt before integration, push, or WeCom reporting.
