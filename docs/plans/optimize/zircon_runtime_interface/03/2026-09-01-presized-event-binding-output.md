record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/event_binding.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/event_binding/capacity_performance_tests.rs
  - tools/tests/test_runtime_interface03_event_binding_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/event_binding/capacity_performance_tests.rs::runtime_interface03_batch44_46_presized_event_binding_release_benchmark
---

# Presized event binding output

## Scope

`UiEventBinding::native_binding` previously grew its single output buffer from zero capacity even
though the complete event path and the action symbol are already borrowed. It now reserves the
known path bytes, separators, action symbol, and delimiters before writing. Argument payloads still
stream into the same buffer and may extend it; output bytes, parsing behavior, and public types are
unchanged.

## Verification

- TDD RED: the focused contract found `String::new()` and no known-output capacity calculation.
- Focused Batch44 event-binding capacity contract after implementation: `2/2` passed.
- Batched static regression after Batch44-45: `111/111` passed (`99` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares presized and zero-capacity projection for actionless bindings,
  no-argument actions, and nested argument payloads.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: refreshed exact-path lease request `8f8be80b6ffa464fbd4427be503b524b`;
Batch44-45 baseline attribution request `c16fd7e066d94e8a90d68f66bf002e6f` (`attributed`).

The complete Batch34/37/38/39 dependency closure plus Batch44-45 snapshot `2741` was created by
request `b8783730ea834ae99975877f99a3925b`. The batched managed validation request
`08c07f0322d84536afebf83965f94800` was rejected before queueing by the external `E:\Git\zr_vm`
dirty-worktree preflight. No validation ticket, Cargo run, terminal benchmark result, integration,
push, or WeCom report exists for Batch44.

Batch44-46 was resubmitted in combined snapshot `2742` (request `714a35d557244e65b16f9f2055849148`);
managed validation request `9a89aca769ae4ed987c86b74aa6f817b` was again rejected before queueing by
the same external `E:\Git\zr_vm` dirty-worktree preflight. No terminal performance number exists.

## Performance contract

The ignored release benchmark projects a long no-argument event binding 500,000 times over 11
alternating samples. It compares zero-capacity growth with the known-prefix reservation and requires
at least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
