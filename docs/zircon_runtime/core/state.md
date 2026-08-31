---
related_code:
  - zircon_runtime/src/core/runtime/state_machine/mod.rs
  - zircon_runtime/src/core/runtime/state_machine/state_spec.rs
  - zircon_runtime/src/core/runtime/state_machine/state.rs
  - zircon_runtime/src/core/runtime/state_machine/next_state.rs
  - zircon_runtime/src/core/runtime/state_machine/state_transition_event.rs
  - zircon_runtime/src/core/runtime/state_machine/on_enter.rs
  - zircon_runtime/src/core/runtime/state_machine/on_exit.rs
  - zircon_runtime/src/core/runtime/state_machine/on_transition.rs
  - zircon_runtime/src/core/runtime/state_machine/hook.rs
  - zircon_runtime/src/core/runtime/state_machine/hook_index.rs
  - zircon_runtime/src/core/runtime/state_machine/registry.rs
  - zircon_runtime/src/core/runtime/state_machine/machine.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/prelude.rs
implementation_files:
  - zircon_runtime/src/core/runtime/state_machine/mod.rs
  - zircon_runtime/src/core/runtime/state_machine/state_spec.rs
  - zircon_runtime/src/core/runtime/state_machine/state.rs
  - zircon_runtime/src/core/runtime/state_machine/next_state.rs
  - zircon_runtime/src/core/runtime/state_machine/state_transition_event.rs
  - zircon_runtime/src/core/runtime/state_machine/on_enter.rs
  - zircon_runtime/src/core/runtime/state_machine/on_exit.rs
  - zircon_runtime/src/core/runtime/state_machine/on_transition.rs
  - zircon_runtime/src/core/runtime/state_machine/hook.rs
  - zircon_runtime/src/core/runtime/state_machine/hook_index.rs
  - zircon_runtime/src/core/runtime/state_machine/registry.rs
  - zircon_runtime/src/core/runtime/state_machine/machine.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01/2026-08-25-m1-state-transition-retention-performance-review.md
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M3 State
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - tools/tests/test_frameworks_01_state_kernel_owner_boundary.py
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

`zircon_runtime::core::runtime::state_machine` owns the runtime-wide finite-state machine vocabulary and executable registry/transition behavior without importing Bevy ECS. It is a runtime-kernel implementation owner, not a pure framework contract layer.

Frameworks01 M1 removed the old `core/framework/state` owner and moved all 12 files to
`core/runtime/state_machine`, where they will follow the future `zr_kernel` physical cut. Callers that
need the namespace owner use `core::runtime::state_machine`; the root `core` facade and prelude keep
explicit product projections. The deleted framework path has no forwarding module or alias.

## Runtime48 M0 Truth Freeze

As of 2026-08-22, this is an experimental, test-only capability. Production
callers outside the `CoreRuntime` to `CoreHandle` forwarding path are absent
from `zircon_app`, `zircon_editor`, the plugin SDK, and the ZrVM runtime. It
must not be used as a product state authority until Runtime48 supplies its
runtime-owned service and schedule integration.

The current API is retained only as migration baseline, not as the target
contract:

- `init_state` publishes an initial transition only when it creates a machine.
  A repeated call returns an unrecorded synthetic DTO for compatibility; it is
  not a transition receipt and callers must not infer publication from it.
- `insert_state` and `apply_state_transition` remain direct mutation paths;
  there is no admitted request queue, producer identity, or schedule barrier.
- Each machine retains only its latest accepted transition in constant space;
  `latest_state_transition` clones one typed DTO and permanent history is not a
  kernel responsibility.
- Hook callbacks have no subscription token, owner generation, quiescence, or
  unload lifecycle. Their current lock-free dispatch shape is not sufficient
  for plugin or module ownership.

Runtime48 M1 still must replace the direct machine/registry authority with a scheduled runtime-owned
state service, bounded journal, and lifecycle-aware subscriptions. Neutral descriptors, requests,
snapshots, and receipts may move to `zr_contracts` only after they exist as behavior-free contracts;
the current direct facade will be hard-cut rather than preserved as a compatibility path.

## Public Surface

- `StateSpec` is the marker contract for typed runtime state values. It requires `'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug`, matching the reusable part of Bevy `States` while avoiding derive macros.
- `State<T>` stores the current value for one state machine and exposes `get()` plus `Deref`.
- `NextState<T>` queues `Unchanged`, `Pending(T)`, or `PendingIfNeq(T)` transitions.
- `StateTransitionEvent<T>` records `exited`, `entered`, and whether identity transitions are allowed.
- `OnEnter<T>`, `OnExit<T>`, and `OnTransition<T>` label hooks for state-specific transition work.
- `CoreRuntime` and `CoreHandle` expose typed facade methods: `init_state`, `insert_state`, `state`, `next_state`, `set_next_state`, `set_next_state_if_neq`, `reset_next_state`, `apply_state_transition`, `latest_state_transition`, and hook registration methods. The deleted plural history API has no compatibility alias.

## Runtime Ownership

The state registry and transition machinery live under `core/runtime/state_machine`; the registry
instance is stored inside `CoreRuntimeInner` and protected by the same runtime ownership path as
config, events, scheduler, and runtime extension hooks. This prevents `zircon_app`, `zircon_editor`,
or `zircon_runtime::scene::ecs` from becoming competing owners.

### Runtime 15 M2 core runtime state module naming hard cutover

`runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred`

Runtime 15 M2 applies R2.3 naming discipline to the runtime state owner: the old `core/runtime/state/runtime_inner.rs` file is removed, and `core/runtime/state/core_runtime_state.rs` now owns the `CoreRuntimeInner` storage struct. `core/runtime/state/mod.rs` declares only `mod core_runtime_state;` for this owner and re-exports `CoreRuntimeInner` from that module, so no banned-name compatibility module or old-path alias remains.

`runtime_15_core_runtime_state_module_uses_owner_name` keeps the source layout, registration structure fixture, this document, Runtime 15 plan rows, runtime index, review findings, module convention docs, and status-output expectations synchronized under the same status anchor. Cargo remains deferred by the Runtime 15 implementation cadence while external Cargo/Rust lanes are active; this slice closes only the core runtime state file-name debt, not the full banned-name sweep.

The implementation stores one typed state machine per `TypeId`. Each machine keeps current state, queued next state, one latest transition, and registered hooks. Hooks are cloned into a dispatch bundle while the registry is locked, then invoked after the lock is released. This avoids registry re-entrancy deadlocks when hooks later inspect runtime state or enqueue follow-up transitions.

### Runtime 15 M3 core handle states lock poison recovery

Runtime 15 M3 extends the E9/F2 poison-safe lock rule to this registry access surface. `core/runtime/handle/states.rs` uses a private `lock_states()` helper before touching `StateRegistry`, so state initialization, insertion, current/next reads, pending transition writes, transition application, latest-event reads, and hook registration recover a poisoned runtime state mutex instead of panicking. `core_handle_state_accessors_recover_poisoned_state_registry_lock` deliberately poisons the state registry lock and verifies init, set/apply transition, state reads, latest-event observation, and reset behavior still work. `structure_convention/lock_poison_policy.rs::runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry` keeps this document, Runtime 15 status rows, and plan mirrors synchronized under `runtime_15_core_handle_states_lock_poison_recovery_static_passed_cargo_deferred`.

## Transition Semantics

`init_state::<T>()` installs `T::default()` when the state machine is absent and records the initial `None -> Some(default)` event. Repeated initialization is idempotent and does not record another transition or dispatch another enter hook. Its current synthetic DTO return is migration baseline only; Runtime48 replaces it with an explicit initialization receipt before product callers are admitted.

M5 follow-up: the repeated-initialization path in `CoreHandle::init_state(...)` now reads the current value through `states.state::<T>().map(State::into_inner)` while retaining the same `lock_states()` guard used for initialization. This preserves the idempotent event result without taking a second registry lock. The public regression test verifies the behavioral contract instead of freezing one obsolete source spelling: no second transition is recorded and no second enter hook is dispatched.

`set_next_state(value)` queues an explicit transition. Applying it records an event even when `value` equals the current state and runs matching identity hooks. `set_next_state_if_neq(value)` queues a transition that is suppressed if the value still equals the current state when applied. In both cases the queue resets to `NextState::Unchanged` after `apply_state_transition` consumes it.

When a non-suppressed transition applies, hooks run in deterministic Bevy-style order:

1. `OnExit(exited)` hooks.
2. `OnTransition { exited, entered }` hooks.
3. `OnEnter(entered)` hooks.

State machines are orthogonal by type. `GameFlow` and `PauseMode` can transition independently and maintain separate latest-transition observations even though they share the same core registry.

## Bevy Alignment And Divergence

The design follows Bevy's core model from `dev/bevy/crates/bevy_state/src/state/states.rs`, `resources.rs`, `transitions.rs`, and `app.rs`: default initialization, current state, next-state queue, transition events, identity-transition control, and enter/exit/transition hooks.

Zircon deliberately diverges from Bevy in these ways:

- no Bevy ECS resources, schedules, or derive macros are introduced;
- hooks are plain typed callbacks registered through `CoreRuntime`/`CoreHandle` rather than ECS schedules;
- Bevy keeps transition messages in update-aged buffers, while this pre-scheduler kernel keeps only one latest event per state type; both designs reject permanent in-machine history;
- computed states, substates, run conditions, and entity-scoped despawn behavior are left for later milestones if the roadmap requires them;
- scene scheduling consumes this core state through future runtime/app integration instead of owning a parallel scene-local state machine.

## Validation

`zircon_runtime/src/tests/state.rs` covers initial events, repeated-initialization non-publication, pending transition application, `PendingIfNeq` identity suppression, explicit identity transitions, hook ordering, independent orthogonal state machines, and 100,000 accepted transitions retaining only the latest observation. `tools/tests/test_frameworks_01_state_kernel_owner_boundary.py` rejects the deleted `Vec` history and plural API. `zircon_runtime/src/tests/prelude.rs` and `zircon_app/src/tests/prelude.rs` verify that the state contracts flow through the runtime and app preludes without moving ownership into `zircon_app`.

Frameworks01 2026-08-25 current-source evidence: state owner boundary `6/6` GREEN in 32.119 seconds. The pre-change D-drive optimized model measured 40 MiB retained payload and 45.498 ms median whole-history query after 1,000,000 transitions; the singular prototype retained 40 B and remained constant-time. Managed Rust product validation and post-cut in-product profiling remain pending, so no frame-time, energy, or power improvement is claimed yet.

M3 testing-stage evidence from 2026-05-08:

- `rustfmt --edition 2021 --check <M3 state/prelude files>` passed.
- `git diff --check -- <M3 state/prelude/docs/session files>` passed with line-ending conversion warnings only.
- `rustc --edition 2021 --crate-type lib zircon_runtime/src/core/state/mod.rs --out-dir C:\Users\HeJiahui\AppData\Local\Temp\opencode` passed for the isolated state module with dead-code warnings expected from standalone compilation.
- `cargo check -p zircon_runtime --lib --locked --message-format short` did not reach M3 acceptance because active asset-stack M3 migration currently breaks `zircon_runtime::asset::importer` call sites around the hard-cut `AssetImportOutcome { entries }` contract. The active owner is `.codex/sessions/20260508-0141-bevy-asset-stack-m1.md`; this M3 State lane did not patch that owned area.
- `cargo check -p zircon_app --lib --locked --message-format short` is blocked by the same `zircon_runtime` asset importer errors before app prelude validation runs.
