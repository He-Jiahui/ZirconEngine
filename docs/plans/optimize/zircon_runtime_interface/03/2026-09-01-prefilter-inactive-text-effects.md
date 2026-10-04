record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_effects.rs
  - zircon_runtime_interface/src/ui/surface/render/text_effects/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_text_effects_normalization_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_effects/performance_tests.rs::runtime_interface03_batch55_61_prefiltered_text_effects_release_benchmark
---

# Prefilter inactive text effects

## Scope

`UiTextDistanceFieldEffects::normalized` previously normalized every declared outline, shadow,
and glow before filtering inactive values. Normalization owns its result and therefore allocated
a color `String` even when non-finite or zero geometry, or a transparent color, immediately made
the effect inactive.

The render-facing normalization now checks the same raw geometry and trimmed alpha conditions
before constructing the owned normalized effect. Empty and whitespace-only colors still select
the effect-specific visible fallback, finite positive values still clamp to the public 64-pixel
limit, and active colors are still trimmed into owned output. Only values whose legacy normalized
form is inactive bypass allocation.

## Verification

- TDD contract covers the prefilter helpers, absence of normalize-then-filter composition, the
  legacy behavior oracle, and the ignored release benchmark marker.
- Focused static performance contract: `2/2` passed.
- Rust behavior coverage compares the optimized result with the retained legacy algorithm across
  default, non-finite, zero, transparent, clamped, fallback-color, and active effects.
- Scoped Rust 1.94.1 formatting and scoped diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending a later batched
  coordinator submission; no terminal performance number is claimed yet.
- Batched request `runtime-interface03-batch55-59-20260901-r1` was accepted for asynchronous
  reconciliation, but coordinator post-response timed out with request `ed8c6d3ced7d448d9d805f8302709365`;
  no terminal Cargo or benchmark output is available yet.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

## Performance contract

The ignored release benchmark normalizes 250,000 inactive three-effect payloads over 11
alternating samples. It compares the former allocate-then-filter implementation with the
prefiltered path and requires at least 20% P95 improvement. Terminal P50/P95 nanosecond values
must come from the managed Windows receipt before integration, push, or WeCom reporting.
