---
related_code:
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/state/state_spec.rs
  - zircon_runtime/src/core/state/state.rs
  - zircon_runtime/src/core/state/next_state.rs
  - zircon_runtime/src/core/state/state_transition_event.rs
  - zircon_runtime/src/core/state/on_enter.rs
  - zircon_runtime/src/core/state/on_exit.rs
  - zircon_runtime/src/core/state/on_transition.rs
  - zircon_runtime/src/core/state/registry.rs
  - zircon_runtime/src/core/state/machine.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/prelude.rs
implementation_files:
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/state/state_spec.rs
  - zircon_runtime/src/core/state/state.rs
  - zircon_runtime/src/core/state/next_state.rs
  - zircon_runtime/src/core/state/state_transition_event.rs
  - zircon_runtime/src/core/state/on_enter.rs
  - zircon_runtime/src/core/state/on_exit.rs
  - zircon_runtime/src/core/state/on_transition.rs
  - zircon_runtime/src/core/state/registry.rs
  - zircon_runtime/src/core/state/machine.rs
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
doc_type: module-detail
---

# Runtime Core State

`zircon_runtime::core::state` owns the M3 runtime-wide finite-state machine contracts for Bevy-aligned application state without importing Bevy ECS. The subsystem is a core-runtime facility because it controls app/runtime mode transitions and scene scheduling decisions that must remain outside editor authoring state.

## Public Surface

- `StateSpec` is the marker contract for typed runtime state values. It requires `'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug`, matching the reusable part of Bevy `States` while avoiding derive macros.
- `State<T>` stores the current value for one state machine and exposes `get()` plus `Deref`.
- `NextState<T>` queues `Unchanged`, `Pending(T)`, or `PendingIfNeq(T)` transitions.
- `StateTransitionEvent<T>` records `exited`, `entered`, and whether identity transitions are allowed.
- `OnEnter<T>`, `OnExit<T>`, and `OnTransition<T>` label hooks for state-specific transition work.
- `CoreRuntime` and `CoreHandle` expose typed facade methods: `init_state`, `insert_state`, `state`, `next_state`, `set_next_state`, `set_next_state_if_neq`, `reset_next_state`, `apply_state_transition`, `state_transition_events`, and hook registration methods.

## Runtime Ownership

The state registry lives inside `CoreRuntimeInner`, protected by the same core ownership path as config, events, scheduler, and runtime extension hooks. This keeps the state authority in `zircon_runtime::core` and prevents `zircon_app`, `zircon_editor`, or `zircon_runtime::scene::ecs` from becoming competing owners.

The implementation stores one typed state machine per `TypeId`. Each machine keeps current state, queued next state, transition event history, and registered hooks. Hooks are cloned into a dispatch bundle while the registry is locked, then invoked after the lock is released. This avoids registry re-entrancy deadlocks when hooks later inspect runtime state or enqueue follow-up transitions.

## Transition Semantics

`init_state::<T>()` installs `T::default()` when the state machine is absent and records the initial `None -> Some(default)` event. Repeated initialization is idempotent and returns the current value without recording another transition.

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
