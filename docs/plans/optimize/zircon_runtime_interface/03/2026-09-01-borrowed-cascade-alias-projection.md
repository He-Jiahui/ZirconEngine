record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs
related_tests:
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry/custom_property_alias_performance_tests.rs
  - tools/tests/test_runtime_interface03_cascade_alias_projection_performance_contract.py
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry/custom_property_alias_performance_tests.rs::runtime_interface03_batch35_borrowed_cascade_alias_release_benchmark
---

# Borrowed cascade alias projection

## Scope

The editor design-token cascade previously cloned every canonical token name into an intermediate
vector before allocating the final CSS custom-property alias and reference strings. Alias projection
now borrows the ordered canonical map keys, creates only the required destination entries, and then
extends the map. Legacy density aliases and deterministic BTree ordering are unchanged.

## Verification

- TDD RED: the focused contract found `.keys().cloned()` in cascade projection and no release
  benchmark module.
- Focused Batch35 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `91/91` passed (`79` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares borrowed and former cloned projections for color, float, and
  integer canonical token entries.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch35-36 snapshot `2736` was created by request `94eb9072f7694ceba8e25fe145cef819`.
The batched managed validation request `9f1698aab5e74d038a1c2e076251e711` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch35.

Ownership receipt: exact-path lease request `ea51eec776024691aba6993b77b98c2b`; baseline
attribution request `5f7ab54239e94bcc9226f4830f5c84d0` (`attributed`).

## Performance contract

The ignored release benchmark projects 4,096 canonical token names 64 times over 11 alternating
samples. It compares the former intermediate key clone with borrowed projection and requires at
least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
