---
related_code:
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/service_lifecycle.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
implementation_files:
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/activation/service_lifecycle.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/reactivation.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/fixture.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/mod.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/reactivation.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/root_contract.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/startup.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - cargo build -p zircon_runtime --locked
  - focused manager reactivation behavior and structure gates
doc_type: milestone-detail
---

# Frameworks05 M4 Manager Service Reactivation Lifecycle

> Owner: [`../05-subsystem-decoupling-contracts.md`](../05-subsystem-decoupling-contracts.md) | Date: 2026-07-14 | Session: `frameworks05-service-reactivation-lifecycle-20260714`

## Completion status

| Milestone | Slice | Status | Evidence |
|---|---|---|---|
| M4 | Complete module service reactivation | completed | Single and batch activation restore every `ModuleEntry::service_names` slot before build; Immediate and lazy services retain their distinct startup semantics. |
| M4 | Identity and rollback invariants | completed | Slot index remains stable; unload advances generation; failed reactivation advances generation only when a newly constructed instance is discarded; module and all slots return to `Unloaded`. |
| M4 | Folder-backed structure | completed | Transition mutation is owned by `handle/activation/service_lifecycle.rs`; behavior and structure coverage have dedicated child owners and enforced budgets. |
| M4 | Runtime15 failure return | completed | Original `deactivation_invalidates_registered_manager_identity_before_reactivation` reproduction now passes without resolver exceptions or compatibility APIs. |

Current status identifier: `frameworks_05_m4_manager_service_reactivation_lifecycle_current_source_passed`

## Architecture result

- `activate_module` snapshots the previous module lifecycle plus immutable complete and startup service lists. Only a module transitioning from `Unloaded` prepares service slots; first activation retains its existing `Registered` rollback semantics.
- `activate_registered_modules` validates all reactivated modules before mutating any slot, then prepares the batch under one service-registry lock. A validation error cannot leave a partially restored batch.
- `ServiceEntry::prepare_for_reactivation` changes availability without changing identity. `reset_after_failed_reactivation` returns availability to `Unloaded` and advances generation only when it discards an instance.
- Waiters are notified after unload, prepare, first-activation reset, and reactivation rollback, always after releasing the service registry lock.
- No stale-handle relaxation, named manager resolver, Arc-holder, compatibility shim, fallback, or test-only branch was added.

## Validation evidence

- Fresh managed Windows default-feature `cargo build -p zircon_runtime --locked`: **passed**, 650.7s, coordinator target key `4069e3844d425556a8a455a02feb84e651e896a66296cf6d2d98424da29bde30`.
- Current-source standalone activation structure harness: **7 passed / 0 failed**, including reactivation owner, original startup exact-count paths, unload mutation, blocked dependency/unload and service-list guards.
- Current built rlib public API probe: **3 passed / 0 failed** for single reactivation, batch reactivation, and finish-error rollback with Immediate/lazy managers.
- Cargo-built Runtime lib-test implementation checkpoint: reactivation behavior **3 passed / 0 failed**; original Runtime15 stale identity reproduction **1 passed / 0 failed**; focused reactivation structure **1 passed / 0 failed**.
- Frameworks05 versioned manager handle architecture guard: **1 passed / 0 failed**.
- Runtime15 global oversized test-file guard: **1 passed / 0 failed**; new behavior owner is 260 lines.
- F18 asset-manager resolution review guard: **1 passed / 0 failed**.
- Scoped rustfmt and `git diff --check`: passed; tracked files only emitted existing LF/CRLF warnings.

## Remaining scope

- This record corrects Frameworks05 M4 reactivation semantics; it does not mark the whole Frameworks05 plan complete. M3 shared text-service work and remaining M4/M5 cross-domain boundaries remain pending in the parent plan.
- Runtime15 still depends on the active Render18 owner reducing `gpu.rs` (897 lines), mesh draw-command tests (515 lines), and compiled-scene `render.rs` (510 lines). Those failures are independent of manager reactivation.
- The active Shader04 shared Cargo job continues its own default lib-test recompilation and scene gate; its result is not claimed by this record.
