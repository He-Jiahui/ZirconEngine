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
  - zircon_runtime/src/core/runtime/handle/states.rs
plan_sources:
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M3 State
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/tests/prelude.rs
  - .github/workflows/ci.yml
  - rustfmt --edition 2021 --check zircon_runtime\src\core\runtime\handle\states.rs zircon_runtime\src\tests\state.rs (2026-06-11 M5 state handle direct projection: passed)
  - state handle direct-projection source guard for direct `match self.state::<T>()`, `Some(state) => Some(state.into_inner())`, `StateTransitionEvent::new(None, entered, true)`, and no old `self.state::<T>().map(State::into_inner)` adapter (2026-06-11 M5 state handle direct projection: passed)
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

The implementation stores one typed state machine per `TypeId`. Each machine keeps current state, queued next state, transition event history, and registered hooks. Hooks are cloned into a dispatch bundle while the registry is locked, then invoked after the lock is released. This avoids registry re-entrancy deadlocks when hooks later inspect runtime state or enqueue follow-up transitions.

## Transition Semantics

`init_state::<T>()` installs `T::default()` when the state machine is absent and records the initial `None -> Some(default)` event. Repeated initialization is idempotent and returns the current value without recording another transition.

M5 follow-up: the repeated-initialization path in `CoreHandle::init_state(...)` now projects the current `State<T>` into the `entered` event value through a direct `match self.state::<T>()` branch before calling `StateTransitionEvent::new(None, entered, true)`. This preserves the same idempotent event result while avoiding the previous `Option::map(State::into_inner)` adapter on the already-initialized path.

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
