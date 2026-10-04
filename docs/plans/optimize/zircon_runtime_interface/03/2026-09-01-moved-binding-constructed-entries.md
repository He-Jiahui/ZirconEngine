record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/parser/constructed_entry_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_constructed_entry_move_performance_contract.py
  - tools/tests/test_runtime_ui_input_routing_receipt_contract.py
  - zircon_runtime_interface/src/ui/binding/model/parser/constructed_entry_performance_tests.rs::runtime_interface03_batch34_moved_constructed_entry_release_benchmark
---

# Moved binding constructed entries

## Scope

The binding model parser already owned the complete argument vector for `record(...)` and `map(...)`
but cloned record names, map string keys, and every value while building the destination container.
Both constructors now consume paired arguments and move owned key/value data directly. Arity, key
type, duplicate record field, and downstream value validation behavior are unchanged.

## Verification

- TDD RED: the focused contract found borrowed pair iteration with deep clones and no release
  benchmark module.
- Focused Batch34 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `89/89` passed (`77` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares moved and former cloning constructors for valid record/map data,
  duplicate record fields, invalid key types, and odd arity.
- The input-routing receipt guard was updated to follow the current
  `runtime_ui/input_routing.rs` module split; the physical receipt remains consumed in production.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch33-34 snapshot `2735` was created by request `bf76138479f841be829b173b6786fe80`.
The batched managed validation request `c8d37d27613041a7bd2260ddaa7f65f4` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch34.

Ownership receipt: exact-path lease request `59c6af2aeeec43cdbbea0f8abed255fc`; baseline
attribution request `73b1697dcfbc4117bace87a75d31e8af` (`attributed`).

## Performance contract

The ignored release benchmark constructs a 32-entry record with owned 96-byte key suffixes and
128-byte values, 512 times over 11 alternating samples. It compares the former extra deep clone with
move consumption and requires at least 20% P95 improvement. Terminal nanosecond values must come
from the managed Windows receipt before integration, push, or WeCom reporting.
