record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/model_context.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/model_context/resolve_performance_tests.rs
  - tools/tests/test_runtime_interface03_model_context_resolve_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/model_context/resolve_performance_tests.rs::runtime_interface03_batch40_selective_model_context_clone_release_benchmark
---

# Selective model context cloning

## Scope

Resolved model context construction previously cloned all four parent provider keys before applying
the patch, then discarded and re-cloned every overridden or cleared layer. Resolution now selects
the parent, replacement, or empty source independently per layer and clones only the provider that
survives into the result. Layer precedence, bind/clear behavior, serialized fields, and iterator
order are unchanged.

## Verification

- TDD RED: the focused contract found `parent.cloned()` followed by mutable per-layer overrides.
- Focused Batch40 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `103/103` passed (`91` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares selective resolution with the former clone-then-overwrite oracle
  for no patch, bind, clear, mixed four-layer overrides, and an absent parent.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch40-41 snapshot `2739` was created by request `ec8e50b9722e4062b599bb33fbb9d3a8`.
The batched managed validation request `30e8173993274545b549664a3f94f38b` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch40. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `86aa2eb6acd34242b54c1d70741e7949`; baseline
attribution is pending for the final source/test/guard/record union.

## Performance contract

The ignored release benchmark resolves four replaced provider layers with long validated IDs
200,000 times over 11 alternating samples. It compares clone-then-overwrite with selective cloning
and requires at least 50% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
