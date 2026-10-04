record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/component/descriptor/default_node_template.rs
related_tests:
  - zircon_runtime_interface/src/ui/component/descriptor/default_node_template/prefix_performance_tests.rs
  - tools/tests/test_runtime_interface03_default_node_prefix_performance_contract.py
  - zircon_runtime_interface/src/ui/component/descriptor/default_node_template/prefix_performance_tests.rs::runtime_interface03_batch53_single_pass_default_node_prefix_release_benchmark
---

# Single-pass default node prefixes

## Scope

`UiDefaultNodeTemplate::native` previously scanned the widget type once to build a normalized
node prefix, then scanned it again to build the control prefix; the node path also created an
intermediate string before lowercasing. The implementation now performs one character pass and
builds both owned prefixes directly, preserving ASCII normalization, separator trimming, fallback
names, and the public template shape.

## Verification

- TDD RED: the focused contract failed because `native` had no shared `native_prefixes` helper.
- Focused Batch53 static performance contract after implementation: `2/2` passed.
- Behavior coverage compares the one-pass prefixes with the former two-pass implementation across
  separators, Unicode, empty-normalized values, and mixed-case widget names.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

## Performance contract

The ignored release benchmark generates both prefixes 200,000 times over 11 alternating samples.
It compares the former two-pass allocating implementation with the one-pass helper and requires at
least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.

Combined Batch53-54 snapshot `2748` was created by request `6218dd60cbea490b9064cb14f11e0666`.
The corrected batched managed validation request `runtime-interface03-batch53-54-20260901-r2`
was rejected before ticket creation by `validation_ticket_external_worktree_dirty` for external
worktree `E:\\Git\\zr_vm`. No Cargo run, commit, push, or terminal performance value exists; the
external worktree remains untouched.
