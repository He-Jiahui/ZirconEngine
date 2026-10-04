record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/parser/collection_view_performance_tests.rs
  - tools/tests/test_runtime_interface03_collection_view_move_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/parser/collection_view_performance_tests.rs::runtime_interface03_batch44_46_moved_collection_view_arguments_release_benchmark
---

# Moved collection-view arguments

## Scope

The fixed-arity `collection_view(...)` constructor previously removed arguments 2 and 0 from an
eight-element `Vec<UiBindingValue>`, shifting the remaining large enum values twice. It now
preserves the existing validation order, then consumes the vector iterator and moves the two owned
identifiers directly. Constructor syntax, validation errors, provider/schema identities, and the
resulting binding value are unchanged.

## Verification

- TDD RED: the focused contract found two `Vec::remove` operations and no fixed-arity move
  benchmark.
- Focused Batch45 collection-view move contract after implementation: `2/2` passed.
- Batched static regression after Batch44-45: `111/111` passed (`99` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares the consuming implementation with the former removal oracle for
  valid input, every argument type failure, both zero-version failures, wrong arity, and multiple
  simultaneous failures that exercise the existing argument-check order.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: initial exact-path lease request `c00ad1e207114288af41856bf4490a4d`;
refreshed Batch44-45 lease request `8f8be80b6ffa464fbd4427be503b524b`; baseline attribution
request `c16fd7e066d94e8a90d68f66bf002e6f` (`attributed`).

The complete Batch34/37/38/39 dependency closure plus Batch44-45 snapshot `2741` was created by
request `b8783730ea834ae99975877f99a3925b`. The batched managed validation request
`08c07f0322d84536afebf83965f94800` was rejected before queueing by the external `E:\Git\zr_vm`
dirty-worktree preflight. No validation ticket, Cargo run, terminal benchmark result, integration,
push, or WeCom report exists for Batch45.

Batch44-46 was resubmitted in combined snapshot `2742` (request `714a35d557244e65b16f9f2055849148`);
managed validation request `9a89aca769ae4ed987c86b74aa6f817b` was again rejected before queueing by
the same external `E:\Git\zr_vm` dirty-worktree preflight. No terminal performance number exists.

## Performance contract

The ignored release benchmark consumes 65,536 prebuilt eight-element argument vectors over 11
alternating samples. It compares the former two shifting removals with iterator consumption and
requires at least 20% P95 improvement. Input allocation is outside the timed region. Terminal
nanosecond values must come from the managed Windows receipt before integration, push, or WeCom
reporting.
