# Native ECS Systems Design

## Summary

Advance ZirconRuntime ECS by making native Rust ECS systems schedulable runtime objects. The current ECS already has `SystemState<P>`, `SystemParam`, queries, resources, commands, events, messages, change detection, removal readers, and a deterministic scene stage runner. The missing lower-layer foundation is a native system abstraction that lets these params run through `Schedule` instead of remaining manual test/runtime primitives.

This slice keeps `zircon_runtime::scene::World` as the public world authority, preserves stable external `EntityId = u64`, does not import `bevy_ecs`, and does not move editor authoring state into runtime world storage.

## Milestone Placement

- Roadmap: `.codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md`.
- Primary milestone: M4 Schedule / Systems / Commands, with existing M5/M6 params reused as system inputs.
- Follow-on relationship: this should land before M9 dynamic scene serialization, M10 editor ECS integration, or M11 performance-parity scheduling because those layers need runtime-native systems as a stable execution carrier.

## Current Baseline

Current ECS source already includes:

- `zircon_runtime/src/scene/ecs/system/system_state.rs`: cached `SystemState<P>` with per-system `last_run` windows.
- `zircon_runtime/src/scene/ecs/system/system_param.rs`: tuple `SystemParam` support up to eight params.
- `zircon_runtime/src/scene/ecs/system/system_param_access.rs`: resource/component/event/message/deferred access descriptors.
- `zircon_runtime/src/scene/ecs/commands/commands.rs`: deferred commands and `CommandsParam`.
- `zircon_runtime/src/scene/ecs/scene_system_descriptor.rs`: stage/order/id descriptor, currently tied to `InternalSceneSystem`.
- `zircon_runtime/src/scene/ecs/scene_system_registry.rs`: deterministic built-in registry.
- `zircon_runtime/src/scene/ecs/schedule_runner.rs`: serial runner that merges built-ins and plugin hooks.
- `zircon_runtime/src/scene/module/world_driver.rs`: world tick entry that runs stages through `SceneScheduleRunner`.

The gap is that `Schedule` cannot store ordinary native ECS systems. It can only store built-in `InternalSceneSystem` descriptors plus plugin hooks supplied by `CoreRuntime`.

## Reference Evidence

### Bevy

- `dev/bevy/crates/bevy_ecs/src/system/system.rs`: Bevy `System` owns persistent param state, access metadata, last-run tick, deferred buffers, and a run method.
- `dev/bevy/crates/bevy_ecs/src/system/function_system.rs`: Bevy converts functions/closures into persistent `FunctionSystem` objects around system-param state.
- `dev/bevy/crates/bevy_ecs/src/system/schedule_system.rs`: Bevy stores schedule systems as boxed `System<In = (), Out = ()>`.
- `dev/bevy/crates/bevy_ecs/src/schedule/executor/single_threaded.rs`: Bevy's serial executor tracks unapplied systems and applies deferred buffers at sync points.
- `dev/bevy/crates/bevy_ecs/src/schedule/auto_insert_apply_deferred.rs`: Bevy auto-inserts sync points for deferred params along dependency edges.

### Fyrox

- `dev/Fyrox/fyrox-impl/src/plugin/mod.rs`: Fyrox exposes lifecycle methods such as `update` and `post_update` through a plugin context rather than turning every hook into an ECS system. This supports keeping Zircon plugin hooks as a parallel ordered lane for this slice.

### Godot

- `dev/godot/scene/main/scene_tree.cpp`: Godot `SceneTree::process`, `physics_process`, and `_process` run staged process groups deterministically and flush deferred queues around frame boundaries. This supports Zircon's serial deterministic runner before parallel scheduling.

## Chosen Architecture

Use a minimal native ECS system runner inside `zircon_runtime::scene::ecs`.

### Owner Boundary

- Owner crate: `zircon_runtime`.
- Owner module: `zircon_runtime::scene::ecs`.
- Public authority: `zircon_runtime::scene::World` remains authoritative for entities, components, resources, commands, and schedule execution state.
- Editor boundary: `zircon_editor` continues to own selection, gizmos, overlays, undo/redo, and authoring tools.

### New Runtime Abstractions

Add a native system object family under `zircon_runtime/src/scene/ecs/system/native/`:

- `SceneSystem`: trait object boundary for scheduled native ECS systems.
- `BoxedSceneSystem`: boxed trait object stored by schedule/registry.
- `FunctionSceneSystem<P, F>`: wrapper around `SystemState<P>` plus `FnMut(P::Item<'_>)`.
- `IntoSceneSystem`: helper trait to convert supported closures/functions into boxed native systems.
- `SceneSystemKind`: descriptor/runtime split that distinguishes built-in internal systems from native ECS systems without forcing native systems into `InternalSceneSystem`.

The system object should expose:

- stable `id: &str` for diagnostics and deterministic sorting;
- `access: &SystemParamAccess` after initialization;
- `has_deferred_commands()` from access metadata;
- `run(&mut World)` for serial schedule execution;
- enough debug metadata to report duplicate ids, invalid descriptors, and initialization errors.

### Schedule And Descriptor Shape

Do not keep extending `SceneSystemDescriptor` as if every scheduled item is an `InternalSceneSystem`. Instead, split metadata from executable payload:

- `SceneSystemDescriptor` remains the serializable metadata shape for id/stage/order and built-in enum descriptors.
- Native systems are runtime-only and skipped by scene serialization.
- `SceneSystemRegistry` owns built-in descriptors plus runtime-native systems behind a separate runtime-only collection.
- `Schedule::register_system(...)` keeps existing built-in behavior.
- Add `Schedule::register_native_system(...)`, taking id/stage/order plus a system closure.
- Add `World::schedule_mut()` as the public world-level mutation path for schedule registration.

The API can be minimal and test-facing first, but it must not require editor state or plugin hook state to enter `World` serialization.

### Execution Model

Execution remains serial and deterministic:

`stage -> order -> id`

Within a stage, `SceneScheduleRunner` merges:

- built-in internal systems;
- native ECS systems;
- plugin scene hooks.

The first slice should preserve existing plugin-hook semantics:

- plugin hooks remain `SceneRuntimeHookRegistration` values owned by `CoreRuntime`;
- plugin hooks are not forced through `SystemParam` yet;
- native systems and hooks sort by the same order/id rule inside the current stage.

### Deferred Commands

Use the existing `SystemParamAccess::has_deferred_commands()` flag:

- If a native system uses `CommandsParam`, its queued commands are invisible while the system closure runs.
- After that native system returns, the runner calls `World::apply_deferred()` before later ordered steps observe the world.
- Existing `InternalSceneSystem::ApplyDeferred` remains an explicit sync point.
- Built-ins and hooks can keep the current post-step `apply_deferred()` behavior unless implementation finds a reason to narrow it without changing existing tests.

This intentionally diverges from Bevy's full graph-aware auto-insertion pass. Zircon currently has no dependency graph or parallel executor, so a serial post-deferred-system flush is the correct lower-layer foundation.

### Change Detection And Persistent State

Native systems must own and reuse their `SystemState<P>` across schedule ticks. This is required for:

- `LocalParam<T>` persistence;
- `MessageReaderParam<T>` cursor persistence;
- `RemovedComponentsParam<T>` cursor persistence;
- `Added<T>` and `Changed<T>` system run windows;
- consistent per-system `last_run` tick semantics.

Creating a new `SystemState<P>` every frame is a design failure for this slice.

### Error Handling

Registration should return structured `ScheduleError` variants instead of panicking for normal user mistakes:

- duplicate system id;
- empty or whitespace-padded id;
- invalid native system params, including missing required resources at initialization;
- potentially conflicting descriptor/runtime identity if the same id is used by a built-in and native system.

Runtime system execution can remain panic-propagating for user closure panics in this slice. A later diagnostics milestone can add catch/unwind or structured runtime system error reports if needed.

## Rejected Alternatives

### Full Schedule Graph Now

Adding sets, dependency edges, ambiguity diagnostics, run conditions, graph rebuilds, and parallel executor support now would align more closely with Bevy, but it is too large for this slice. It would also churn plugin-hook ordering and internal built-in execution before the native system carrier exists.

### Plugin-Hook Bridge Only

Mapping plugin hooks more tightly into ECS params would not solve the core gap: `SystemState<P>` would still lack a runtime schedule carrier. Native ECS systems should land first, then plugin-facing system registration can build on it.

## Validation Plan

Focused tests should land under existing ECS test files unless file size or responsibility pushes a folder split.

Runtime system coverage:

- Native systems register with id/stage/order and run in deterministic `stage -> order -> id` order.
- Duplicate and blank native system ids return `ScheduleError`.
- Native system initialization reports missing required resources through `SystemParamError` routed into schedule registration.
- `LocalParam<T>` persists across multiple scheduled ticks.
- `MessageReaderParam<T>` reads each scheduled message once per system state.
- `Added<T>` and `Changed<T>` observe per-system windows across scheduled ticks.

Deferred command coverage:

- Commands queued by a native system are not visible inside that system before the sync point.
- Later ordered native systems in the same stage observe command effects after the runner flushes deferred work.
- Explicit `ApplyDeferred` remains a valid named sync point.

Integration ordering coverage:

- Built-in internal systems, native ECS systems, and plugin hooks sort deterministically together.
- Existing `PostUpdate` derived-state and `RenderExtract` dirty-state tests remain valid.

Documentation coverage:

- Update `docs/zircon_runtime/scene/ecs.md` with native systems, runtime-only serialization behavior, ordering, deferred sync, and validation evidence.

Expected testing-stage commands for the eventual implementation milestone:

```powershell
cargo test -p zircon_runtime --lib scene::tests::ecs_systems --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_schedule --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_change_detection --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests --locked --message-format short
```

Do not claim root workspace green unless the full root and plugin workspace commands from `CLAUDE.md` pass freshly.

## Intentional Divergence

- Zircon does not import or wrap Bevy ECS.
- Native systems use Zircon's `u64`-backed `ChangeTick`, not Bevy's wrapped `u32` maintenance path.
- The first native executor is serial and deterministic; no parallel executor or ambiguity graph lands in this slice.
- Deferred sync uses existing serial stage order rather than Bevy's dependency-edge auto-insertion.
- Plugin hooks remain lifecycle hook registrations, matching Zircon/Fyrox-style plugin boundaries, until a later plugin-facing ECS system registration milestone.

## Acceptance Criteria

- `Schedule` can own runtime-native ECS systems without serializing them into scene/project data.
- Native systems reuse persistent `SystemState<P>` across ticks.
- Existing built-in systems and plugin hooks continue to run through `WorldDriver` with deterministic ordering.
- Deferred command visibility is deterministic and tested.
- Change detection, local state, message cursors, and removed-component cursors work from scheduled native systems, not only manual tests.
- `docs/zircon_runtime/scene/ecs.md` records the new module behavior and test evidence.
