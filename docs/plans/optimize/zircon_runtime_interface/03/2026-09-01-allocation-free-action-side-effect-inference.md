---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/action_policy/side_effect_class.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/action_policy/side_effect_class/performance_tests.rs
  - tools/tests/test_runtime_interface03_action_side_effect_inference_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/action_policy/side_effect_class/performance_tests.rs::runtime_interface03_batch51_52_borrowed_side_effect_inference_release_benchmark
---

# Allocation-free action side-effect inference

## Scope

Action-policy classification previously allocated lowercase copies of both optional inputs and then
allocated a third formatted string before keyword matching. Inference now checks borrowed byte
windows with ASCII case-insensitive equality. Category priority, mixed-case matching, substring
matching, independent route/action admission, and the public enum are unchanged.

## Verification

- TDD RED: the focused contract found two `to_ascii_lowercase` allocations, one `format!`
  allocation, and no release benchmark child.
- Focused Batch51-52 performance contracts after implementation: `4/4` passed.
- Batched static regression after Batch51-52: `124/124` passed (`112` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares the borrowed implementation with the former allocating oracle
  for empty, local, editor, asset, scene, process, network, mixed-case, Unicode-adjacent, substring,
  and cross-category priority inputs.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: initial exact-path lease request `235e24885213443bb878db1f16cf795b`; refreshed
batch lease request `0834608c8fcb433399a23eb7221fb3ce`; baseline attribution request
`60c68e0f2a594ae9a4a4a5fb44045f70` (`attributed`).

## Performance contract

The ignored release benchmark classifies 100,000 mixed-case policy inputs over 11 alternating
samples. It compares three temporary strings with borrowed case-insensitive matching and requires at
least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.

Combined Batch51-52 snapshot `2746` was created by request
`e770dff93eeb455693db534f540a2720`. Batched release request
`runtime-interface03-batch51-52-20260901-r1` was rejected before ticket creation by
`validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`. No Cargo run,
commit, push, or terminal performance value exists; the external worktree remains untouched.
