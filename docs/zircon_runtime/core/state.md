---
related_code:
  - zircon_runtime/src/core/framework/state/mod.rs
  - zircon_runtime/src/core/framework/state/state_spec.rs
  - zircon_runtime/src/core/framework/state/state.rs
  - zircon_runtime/src/core/framework/state/next_state.rs
  - zircon_runtime/src/core/framework/state/state_transition_event.rs
  - zircon_runtime/src/core/framework/state/on_enter.rs
  - zircon_runtime/src/core/framework/state/on_exit.rs
  - zircon_runtime/src/core/framework/state/on_transition.rs
  - zircon_runtime/src/core/framework/state/registry.rs
  - zircon_runtime/src/core/framework/state/machine.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/prelude.rs
implementation_files:
  - zircon_runtime/src/core/framework/state/mod.rs
  - zircon_runtime/src/core/framework/state/state_spec.rs
  - zircon_runtime/src/core/framework/state/state.rs
  - zircon_runtime/src/core/framework/state/next_state.rs
  - zircon_runtime/src/core/framework/state/state_transition_event.rs
  - zircon_runtime/src/core/framework/state/on_enter.rs
  - zircon_runtime/src/core/framework/state/on_exit.rs
  - zircon_runtime/src/core/framework/state/on_transition.rs
  - zircon_runtime/src/core/framework/state/registry.rs
  - zircon_runtime/src/core/framework/state/machine.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
plan_sources:
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M3 State
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/core/runtime/handle/states.rs::tests::core_handle_state_accessors_recover_poisoned_state_registry_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs::runtime_15_core_runtime_state_module_uses_owner_name
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/tests/prelude.rs
  - .github/workflows/ci.yml
  - rustfmt --edition 2021 --check zircon_runtime\src\core\runtime\handle\states.rs zircon_runtime\src\tests\state.rs (2026-06-11 M5 state handle direct projection: passed)
  - repeated-initialization behavior guard for returning the current value without recording another transition or dispatching another enter hook; the handle-local unit test separately keeps the single-registry-guard implementation boundary current
  - conflict-marker, trailing-whitespace, and git diff --check scans over zircon_runtime/src/core/runtime/handle/states.rs, zircon_runtime/src/tests/state.rs, docs/zircon_runtime/core/state.md, and .codex/sessions/20260604-1232-runtime-architecture-review.md (2026-06-11 M5 state handle direct projection: passed with expected LF-to-CRLF warnings only for tracked files)
  - cargo validation for M5 state handle direct projection (2026-06-11: deferred because active shared Cargo/rustc lanes were running; no new Cargo command was started and no Cargo pass/fail is claimed)
doc_type: module-detail
---

# Runtime Core State

`zircon_runtime::core::framework::state` owns the M3 runtime-wide finite-state machine contracts for Bevy-aligned application state without importing Bevy ECS. The subsystem is a framework contract layer because it defines neutral typed state and transition vocabulary, while runtime handle methods own the actual registry mutation path.

The 2026-06-12 runtime 02 M2.2 migration moved the former `core/state/` root directory under `core/framework/state/`. Callers that need the namespace owner use `core::framework::state`; the root `core` facade still re-exports the individual state contracts for prelude stability.

## Public Surface

- `StateSpec` is the marker contract for typed runtime state values. It requires `'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug`, matching the reusable part of Bevy `States` while avoiding derive macros.
- `State<T>` stores the current value for one state machine and exposes `get()` plus `Deref`.
- `NextState<T>` queues `Unchanged`, `Pending(T)`, or `PendingIfNeq(T)` transitions.
- `StateTransitionEvent<T>` records `exited`, `entered`, and whether identity transitions are allowed.
- `OnEnter<T>`, `OnExit<T>`, and `OnTransition<T>` label hooks for state-specific transition work.
- `CoreRuntime` and `CoreHandle` expose typed facade methods: `init_state`, `insert_state`, `state`, `next_state`, `set_next_state`, `set_next_state_if_neq`, `reset_next_state`, `apply_state_transition`, `state_transition_events`, and hook registration methods.

## Runtime Ownership

The state registry lives inside `CoreRuntimeInner`, protected by the same runtime ownership path as config, events, scheduler, and runtime extension hooks. This keeps runtime state authority in `zircon_runtime::core::runtime` and prevents `zircon_app`, `zircon_editor`, or `zircon_runtime::scene::ecs` from becoming competing owners.

### Runtime 15 M2 core runtime state module naming hard cutover

`runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred`

Runtime 15 M2 applies R2.3 naming discipline to the runtime state owner: the old `core/runtime/state/runtime_inner.rs` file is removed, and `core/runtime/state/core_runtime_state.rs` now owns the `CoreRuntimeInner` storage struct. `core/runtime/state/mod.rs` declares only `mod core_runtime_state;` for this owner and re-exports `CoreRuntimeInner` from that module, so no banned-name compatibility module or old-path alias remains.

`runtime_15_core_runtime_state_module_uses_owner_name` keeps the source layout, registration structure fixture, this document, Runtime 15 plan rows, runtime index, review findings, module convention docs, and status-output expectations synchronized under the same status anchor. Cargo remains deferred by the Runtime 15 implementation cadence while external Cargo/Rust lanes are active; this slice closes only the core runtime state file-name debt, not the full banned-name sweep.

The implementation stores one typed state machine per `TypeId`. Each machine keeps current state, queued next state, transition event history, and registered hooks. Hooks are cloned into a dispatch bundle while the registry is locked, then invoked after the lock is released. This avoids registry re-entrancy deadlocks when hooks later inspect runtime state or enqueue follow-up transitions.

### Runtime 15 M3 core handle states lock poison recovery

Runtime 15 M3 extends the E9/F2 poison-safe lock rule to this registry access surface. `core/runtime/handle/states.rs` now uses a private `lock_states()` helper before touching `StateRegistry`, so state initialization, insertion, current/next reads, pending transition writes, transition application, event history reads, and hook registration recover a poisoned runtime state mutex instead of panicking. `core_handle_state_accessors_recover_poisoned_state_registry_lock` deliberately poisons the state registry lock and verifies init, set/apply transition, state reads, event history, and reset behavior still work. `structure_convention/lock_poison_policy.rs::runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry` keeps this document, Runtime 15 status rows, and plan mirrors synchronized under `runtime_15_core_handle_states_lock_poison_recovery_static_passed_cargo_deferred`.

## Transition Semantics

`init_state::<T>()` installs `T::default()` when the state machine is absent and records the initial `None -> Some(default)` event. Repeated initialization is idempotent: it returns a synthetic `StateTransitionEvent { exited: None, entered: Some(current), allow_same_state_transitions: true }` DTO, but does not record that DTO as another transition or dispatch another enter hook. Callers must not interpret the repeated-init return value as a published transition receipt.

M5 follow-up: the repeated-initialization path in `CoreHandle::init_state(...)` now reads the current value through `states.state::<T>().map(State::into_inner)` while retaining the same `lock_states()` guard used for initialization. This preserves the idempotent event result without taking a second registry lock. The public regression test verifies the behavioral contract instead of freezing one obsolete source spelling: no second transition is recorded and no second enter hook is dispatched.

`set_next_state(value)` queues an explicit transition. Applying it records an event even when `value` equals the current state and runs matching identity hooks. `set_next_state_if_neq(value)` queues a transition that is suppressed if the value still equals the current state when applied. In both cases the queue resets to `NextState::Unchanged` after `apply_state_transition` consumes it.

When a non-suppressed transition applies, hooks run in deterministic Bevy-style order:

1. `OnExit(exited)` hooks.
2. `OnTransition { exited, entered }` hooks.
3. `OnEnter(entered)` hooks.

State machines are orthogonal by type. `GameFlow` and `PauseMode` can transition independently and maintain separate event histories even though they share the same core registry.

## Bevy Alignment And Divergence

The design follows Bevy's core model from `dev/bevy/crates/bevy_state/src/state/states.rs`, `resources.rs`, `transitions.rs`, and `app.rs`: default initialization, current state, next-state queue, transition events, identity-transition control, and enter/exit/transition hooks.

Zircon deliberately diverges from Bevy in these ways:

- no Bevy ECS resources, schedules, or derive macros are introduced;
- hooks are plain typed callbacks registered through `CoreRuntime`/`CoreHandle` rather than ECS schedules;
- computed states, substates, run conditions, and entity-scoped despawn behavior are left for later milestones if the roadmap requires them;
- scene scheduling consumes this core state through future runtime/app integration instead of owning a parallel scene-local state machine.

## Validation

`zircon_runtime/src/tests/state.rs` covers initial events, pending transition application, `PendingIfNeq` identity suppression, explicit identity transitions, hook ordering, and independent orthogonal state machines. `zircon_runtime/src/tests/prelude.rs` and `zircon_app/src/tests/prelude.rs` verify that the state contracts flow through the runtime and app preludes without moving ownership into `zircon_app`.

M3 testing-stage evidence from 2026-05-08:

- `rustfmt --edition 2021 --check <M3 state/prelude files>` passed.
- `git diff --check -- <M3 state/prelude/docs/session files>` passed with line-ending conversion warnings only.
- `rustc --edition 2021 --crate-type lib zircon_runtime/src/core/state/mod.rs --out-dir C:\Users\HeJiahui\AppData\Local\Temp\opencode` passed for the isolated state module with dead-code warnings expected from standalone compilation.
- `cargo check -p zircon_runtime --lib --locked --message-format short` did not reach M3 acceptance because active asset-stack M3 migration currently breaks `zircon_runtime::asset::importer` call sites around the hard-cut `AssetImportOutcome { entries }` contract. The active owner is `.codex/sessions/20260508-0141-bevy-asset-stack-m1.md`; this M3 State lane did not patch that owned area.
- `cargo check -p zircon_app --lib --locked --message-format short` is blocked by the same `zircon_runtime` asset importer errors before app prelude validation runs.
