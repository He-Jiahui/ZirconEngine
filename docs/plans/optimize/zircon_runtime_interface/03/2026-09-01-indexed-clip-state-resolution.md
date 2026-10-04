record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/batch/clip.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/clip/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_clip_state_resolution_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/batch/clip/performance_tests.rs::runtime_interface03_batch17_clip_resolution_release_benchmark
---

# Indexed clip-state resolution

## Scope

`UiClipStack::resolve` previously scanned every interned clip state even though
`UiBatchClipStates` already maintains an identity-to-index table for interning.

Resolution now reuses that index and validates the indexed structural value before returning it.
If serde has restored `states` while intentionally skipping `indices`, or if an identity entry is
missing or stale, the retained linear lookup preserves compatibility. No clip state is cloned or
allocated during resolution.

## Verification

- TDD RED: the focused contract found no indexed resolve path, fallback oracle, or benchmark.
- Focused Clip/Pipeline static contracts after implementation: `4/4` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `57/57`
  passed (`45 + 9 + 3`).
- Rust behavior coverage compares indexed and linear resolution and proves a serde-restored empty
  index takes the equality-preserving fallback.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission:

- attribution request `69fe23f5649f453989cff0d1e0246421`;
- snapshot `2729`, create request `4e335ac3f8b646c8b69b8a009d5bb6e9`;
- validation idempotency request `6dff1b2c8ea54fa49db8ea0f41dd0e75` was rejected before
  queueing with `validation_ticket_external_worktree_dirty` because `E:\Git\zr_vm` is dirty;
- the unrelated external worktree is left unchanged; this batch will be resubmitted with later
  tasks after the managed preflight becomes available.

## Performance contract

The ignored release benchmark resolves the last of 4,096 interned clip states 256 times over 11
alternating samples. It compares the retained O(n) equality scan with the O(1) identity lookup and
requires at least 80% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
