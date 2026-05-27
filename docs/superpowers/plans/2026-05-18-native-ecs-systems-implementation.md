# Native ECS Systems Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make ZirconRuntime native Rust ECS systems first-class schedulable runtime objects while preserving `zircon_runtime::scene::World` authority and stable `EntityId = u64`.

**Architecture:** Add a runtime-only native system subtree under `zircon_runtime::scene::ecs::system::native`, store native systems separately from serializable built-in `InternalSceneSystem` descriptors, and extend the serial `SceneScheduleRunner` to merge built-ins, native systems, and plugin hooks by `stage -> order -> id`. Native systems wrap persistent `SystemState<P>` so `LocalParam`, message cursors, removed-component cursors, and change windows survive across scheduled ticks.

**Tech Stack:** Rust, Cargo, `serde`, `thiserror`, ZirconRuntime self-owned ECS, repository-local Bevy/Fyrox/Godot reference evidence. Do not add `bevy_ecs` or `bevy_reflect` dependencies.

---

## Source Context

- Design spec: `docs/superpowers/specs/2026-05-18-native-ecs-systems-design.md`.
- Roadmap: `.codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md`, primarily M4 Schedule / Systems / Commands.
- Existing ECS docs owner: `docs/zircon_runtime/scene/ecs.md`.
- Main policy: work directly on `main`; do not create branches or worktrees.
- Cadence policy: implementation slices may add code and unit-test code, but Cargo build/test commands run in named milestone testing stages.
- Disk policy: before Cargo testing, check the target drive; if free space is `<= 50 GB`, clean the active target directory or choose a pre-approved shared target directory.

## File Structure

Create:

- `zircon_runtime/src/scene/ecs/system/native/mod.rs`: structural module declarations and curated public re-exports only.
- `zircon_runtime/src/scene/ecs/system/native/scene_system.rs`: `SceneSystem` trait and `BoxedSceneSystem` type alias.
- `zircon_runtime/src/scene/ecs/system/native/function_scene_system.rs`: `FunctionSceneSystem<P, F>` wrapper around persistent `SystemState<P>` and `FnMut(P::Item<'_>)`.
- `zircon_runtime/src/scene/ecs/system/native/into_scene_system.rs`: `IntoSceneSystem<P>` conversion trait for function/closure systems.
- `zircon_runtime/src/scene/ecs/system/native/scene_system_metadata.rs`: runtime metadata fields shared by native system objects: id, stage, order.
- `zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs`: runtime execution step enum used by `SceneScheduleRunner` to sort built-ins, native systems, and plugin hooks together.

Modify:

- `zircon_runtime/src/scene/ecs/system/mod.rs`: add `mod native;` plus public re-exports for native system types.
- `zircon_runtime/src/scene/ecs/mod.rs`: re-export native system types from the ECS public surface.
- `zircon_runtime/src/scene/mod.rs`: re-export native system types from `zircon_runtime::scene` where the existing ECS surface is already curated.
- `zircon_runtime/src/scene/ecs/schedule_error.rs`: add structured native-system registration and initialization variants.
- `zircon_runtime/src/scene/ecs/scene_system_registry.rs`: store built-in descriptors and runtime-only native systems in separate collections; add native registration and stage iteration.
- `zircon_runtime/src/scene/ecs/schedule.rs`: add `Schedule::register_native_system(...)`, `Schedule::native_systems_for_stage_mut(...)`, and any read-only inspection helpers needed by tests.
- `zircon_runtime/src/scene/world/query.rs`: add `World::schedule_mut()` beside `World::schedule()`.
- `zircon_runtime/src/scene/ecs/schedule_runner.rs`: replace the local two-variant step enum with a step list that includes native ECS systems and invokes persistent system state.
- `zircon_runtime/src/scene/module/world_driver.rs`: collect only built-in descriptors by value and let `SceneScheduleRunner` access native systems through `World` during stage execution.
- `zircon_runtime/src/scene/tests/ecs_systems.rs`: add native-system persistent state tests.
- `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs`: add focused scheduled native-system message cursor, change-window, removed-component, and deferred command tests.
- `zircon_runtime/src/scene/tests/ecs_schedule.rs`: add native/built-in/plugin ordering, duplicate id, blank id, and missing resource initialization tests.
- `docs/zircon_runtime/scene/ecs.md`: update header lists plus native system behavior, ordering, deferred sync, runtime-only serialization, and fresh validation evidence.
- `.codex/sessions/20260518-1848-ecs-continuation-design.md`: update live status and validation evidence during execution.

Do not modify:

- `zircon_runtime_interface`: no ABI or DTO changes are needed for this runtime-only slice.
- `zircon_editor`: editor authoring state remains outside runtime ECS.
- `zircon_plugins`: plugin hooks stay as `SceneRuntimeHookRegistration`; plugin-facing ECS system registration is a later milestone.

---

## Milestone 1: Native System Object Foundation

### Goal

Add the native ECS system object model and registration API without changing frame execution yet.

### In-Scope Behaviors

- A native system stores id, stage, order, access metadata, and a persistent `SystemState<P>`.
- `Schedule::register_native_system(...)` validates id and initializes params once.
- Native systems are runtime-only and not included in serialized `SceneSystemDescriptor` output.
- Duplicate ids are rejected across both built-in descriptors and native systems.
- Missing required resources during native system initialization return `ScheduleError::SystemParam` with the native system id.

### Dependencies

- Existing `SystemState<P>` and `SystemParamAccess` must remain unchanged in semantics.
- Existing built-in `Schedule::register_system(...)` behavior must keep passing current tests.

### Implementation Slices

- [ ] **Slice 1: Add native system metadata and trait files**

Create `zircon_runtime/src/scene/ecs/system/native/scene_system_metadata.rs`:

```rust
use crate::scene::ecs::SystemStage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneSystemMetadata {
    id: String,
    stage: SystemStage,
    order: i32,
}

impl SceneSystemMetadata {
    pub fn new(id: impl Into<String>, stage: SystemStage, order: i32) -> Self {
        Self {
            id: id.into(),
            stage,
            order,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn stage(&self) -> SystemStage {
        self.stage
    }

    pub fn order(&self) -> i32 {
        self.order
    }
}
```

Create `zircon_runtime/src/scene/ecs/system/native/scene_system.rs`:

```rust
use std::fmt;

use crate::scene::ecs::{SceneSystemMetadata, SystemParamAccess, SystemStage};
use crate::scene::World;

pub type BoxedSceneSystem = Box<dyn SceneSystem>;

pub trait SceneSystem: Send + 'static {
    fn metadata(&self) -> &SceneSystemMetadata;
    fn access(&self) -> &SystemParamAccess;
    fn run(&mut self, world: &mut World);

    fn id(&self) -> &str {
        self.metadata().id()
    }

    fn stage(&self) -> SystemStage {
        self.metadata().stage()
    }

    fn order(&self) -> i32 {
        self.metadata().order()
    }

    fn has_deferred_commands(&self) -> bool {
        self.access().has_deferred_commands()
    }
}

impl fmt::Debug for dyn SceneSystem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneSystem")
            .field("id", &self.id())
            .field("stage", &self.stage())
            .field("order", &self.order())
            .field("has_deferred_commands", &self.has_deferred_commands())
            .finish_non_exhaustive()
    }
}
```

- [ ] **Slice 2: Add persistent function system wrapper**

Create `zircon_runtime/src/scene/ecs/system/native/function_scene_system.rs`:

```rust
use std::marker::PhantomData;

use crate::scene::ecs::{SceneSystem, SceneSystemMetadata, SystemParam, SystemParamAccess, SystemState};
use crate::scene::World;

pub struct FunctionSceneSystem<P, F>
where
    P: SystemParam,
{
    metadata: SceneSystemMetadata,
    state: SystemState<P>,
    system: F,
    _marker: PhantomData<fn() -> P>,
}

impl<P, F> FunctionSceneSystem<P, F>
where
    P: SystemParam,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    pub fn new(metadata: SceneSystemMetadata, world: &mut World, system: F) -> Result<Self, crate::scene::ecs::SystemParamError> {
        let state = SystemState::<P>::new(world)?;
        Ok(Self {
            metadata,
            state,
            system,
            _marker: PhantomData,
        })
    }
}

impl<P, F> SceneSystem for FunctionSceneSystem<P, F>
where
    P: SystemParam,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        self.state.access()
    }

    fn run(&mut self, world: &mut World) {
        self.state.run(world, |params| (self.system)(params));
    }
}
```

If Rust rejects the higher-ranked closure bound for `FnMut(P::Item<'world>)`, use the same explicit `P` marker approach at registration sites and keep the public API shaped as `register_native_system::<P, _>(...)` so the closure parameter type remains inferred from `P::Item<'_>`.

- [ ] **Slice 3: Add native system conversion module**

Create `zircon_runtime/src/scene/ecs/system/native/into_scene_system.rs`:

```rust
use crate::scene::ecs::{BoxedSceneSystem, FunctionSceneSystem, SceneSystemMetadata, SystemParam, SystemParamError};
use crate::scene::World;

pub trait IntoSceneSystem<P>
where
    P: SystemParam,
{
    fn into_scene_system(
        self,
        metadata: SceneSystemMetadata,
        world: &mut World,
    ) -> Result<BoxedSceneSystem, SystemParamError>;
}

impl<P, F> IntoSceneSystem<P> for F
where
    P: SystemParam + 'static,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn into_scene_system(
        self,
        metadata: SceneSystemMetadata,
        world: &mut World,
    ) -> Result<BoxedSceneSystem, SystemParamError> {
        Ok(Box::new(FunctionSceneSystem::<P, F>::new(metadata, world, self)?))
    }
}
```

Create `zircon_runtime/src/scene/ecs/system/native/mod.rs`:

```rust
mod function_scene_system;
mod into_scene_system;
mod scene_system;
mod scene_system_metadata;
mod scheduled_scene_step;

pub use function_scene_system::FunctionSceneSystem;
pub use into_scene_system::IntoSceneSystem;
pub use scene_system::{BoxedSceneSystem, SceneSystem};
pub use scene_system_metadata::SceneSystemMetadata;

pub(crate) use scheduled_scene_step::ScheduledSceneStep;
```

- [ ] **Slice 4: Extend schedule errors**

Modify `zircon_runtime/src/scene/ecs/schedule_error.rs` to include the existing variants plus native init context:

```rust
use thiserror::Error;

use crate::scene::ecs::SystemParamError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("system id cannot be empty")]
    EmptySystemId,
    #[error("system {0} already registered")]
    DuplicateSystem(String),
    #[error("system {system_id} failed to initialize params: {source}")]
    SystemParam {
        system_id: String,
        source: SystemParamError,
    },
}
```

Keep existing display text for `EmptySystemId` and `DuplicateSystem` unchanged so current tests still pass.

- [ ] **Slice 5: Store native systems in the registry**

Modify `zircon_runtime/src/scene/ecs/scene_system_registry.rs`:

```rust
use serde::{Deserialize, Serialize};

use super::{
    BoxedSceneSystem, InternalSceneSystem, IntoSceneSystem, SceneSystemDescriptor,
    SceneSystemMetadata, ScheduleError, SystemParam, SystemStage,
};

#[derive(Serialize, Deserialize)]
pub struct SceneSystemRegistry {
    systems: Vec<SceneSystemDescriptor>,
    #[serde(skip, default)]
    native_systems: Vec<BoxedSceneSystem>,
}
```

Add a manual `Debug` implementation that prints `systems` and the native system ids. Add a manual `PartialEq` implementation that preserves existing runtime-state skip semantics by comparing only serialized built-in descriptors.

Add registration helpers:

```rust
pub fn register_native_system<P, S>(
    &mut self,
    id: impl Into<String>,
    stage: SystemStage,
    order: i32,
    world: &mut crate::scene::World,
    system: S,
) -> Result<(), ScheduleError>
where
    P: SystemParam + 'static,
    S: IntoSceneSystem<P>,
{
    let id = id.into();
    validate_system_id(&id)?;
    self.ensure_unique_system_id(&id)?;
    let metadata = SceneSystemMetadata::new(id.clone(), stage, order);
    let system = system
        .into_scene_system(metadata, world)
        .map_err(|source| ScheduleError::SystemParam {
            system_id: id,
            source,
        })?;
    self.native_systems.push(system);
    sort_native_systems(&mut self.native_systems);
    Ok(())
}
```

Add read and mutable stage iterators:

```rust
pub fn native_systems_for_stage(
    &self,
    stage: SystemStage,
) -> impl Iterator<Item = &dyn crate::scene::ecs::SceneSystem> {
    self.native_systems
        .iter()
        .map(|system| system.as_ref())
        .filter(move |system| system.stage() == stage)
}

pub(crate) fn native_systems_for_stage_mut(
    &mut self,
    stage: SystemStage,
) -> impl Iterator<Item = &mut BoxedSceneSystem> {
    self.native_systems
        .iter_mut()
        .filter(move |system| system.stage() == stage)
}
```

Use one shared id validator for built-ins and native systems:

```rust
fn validate_system_id(id: &str) -> Result<(), ScheduleError> {
    if id.trim().is_empty() || id.trim() != id {
        return Err(ScheduleError::EmptySystemId);
    }
    Ok(())
}
```

Ensure `ensure_unique_system_id` checks both `systems` and `native_systems`.

- [ ] **Slice 6: Add schedule and world mutation API**

Modify `zircon_runtime/src/scene/ecs/schedule.rs` to expose:

```rust
pub fn register_native_system<P, S>(
    &mut self,
    id: impl Into<String>,
    stage: SystemStage,
    order: i32,
    world: &mut crate::scene::World,
    system: S,
) -> Result<(), ScheduleError>
where
    P: crate::scene::ecs::SystemParam + 'static,
    S: crate::scene::ecs::IntoSceneSystem<P>,
{
    self.systems
        .register_native_system::<P, S>(id, stage, order, world, system)
}

pub fn native_systems_for_stage(
    &self,
    stage: SystemStage,
) -> impl Iterator<Item = &dyn crate::scene::ecs::SceneSystem> {
    self.systems.native_systems_for_stage(stage)
}

pub(crate) fn native_systems_for_stage_mut(
    &mut self,
    stage: SystemStage,
) -> impl Iterator<Item = &mut crate::scene::ecs::BoxedSceneSystem> {
    self.systems.native_systems_for_stage_mut(stage)
}
```

Modify `zircon_runtime/src/scene/world/query.rs` to add:

```rust
pub fn schedule_mut(&mut self) -> &mut Schedule {
    &mut self.schedule
}
```

Because `Schedule::register_native_system` needs both `&mut Schedule` and `&mut World`, also add a `World` convenience method in `query.rs` to avoid external double-borrow issues:

```rust
pub fn register_native_system<P, S>(
    &mut self,
    id: impl Into<String>,
    stage: crate::scene::ecs::SystemStage,
    order: i32,
    system: S,
) -> Result<(), crate::scene::ecs::ScheduleError>
where
    P: crate::scene::ecs::SystemParam + 'static,
    S: crate::scene::ecs::IntoSceneSystem<P>,
{
    let mut schedule = std::mem::take(&mut self.schedule);
    let result = schedule.register_native_system::<P, S>(id, stage, order, self, system);
    self.schedule = schedule;
    result
}
```

This keeps the ergonomic world-level API while still satisfying Rust borrowing rules.

- [ ] **Slice 7: Wire re-exports**

Modify `zircon_runtime/src/scene/ecs/system/mod.rs`:

```rust
mod native;

pub use native::{
    BoxedSceneSystem, FunctionSceneSystem, IntoSceneSystem, SceneSystem, SceneSystemMetadata,
};
pub(crate) use native::ScheduledSceneStep;
```

Modify `zircon_runtime/src/scene/ecs/mod.rs` and `zircon_runtime/src/scene/mod.rs` to include:

```rust
BoxedSceneSystem, FunctionSceneSystem, IntoSceneSystem, SceneSystem, SceneSystemMetadata,
```

Keep root modules structural; do not add behavior to `mod.rs` files.

### Unit-Test Code To Add Before Testing Stage

Add these tests to `zircon_runtime/src/scene/tests/ecs_schedule.rs`:

```rust
#[test]
fn schedule_rejects_duplicate_native_and_builtin_system_ids() {
    let mut world = crate::scene::World::empty();
    let duplicate_builtin = world
        .register_native_system::<(), _>(
            "zircon.scene.node_cache",
            SystemStage::Update,
            0,
            |_| {},
        )
        .unwrap_err();
    assert!(duplicate_builtin
        .to_string()
        .contains("system zircon.scene.node_cache already registered"));

    world
        .register_native_system::<(), _>("gameplay.first", SystemStage::Update, 0, |_| {})
        .unwrap();
    let duplicate_native = world
        .register_native_system::<(), _>("gameplay.first", SystemStage::Update, 1, |_| {})
        .unwrap_err();
    assert!(duplicate_native
        .to_string()
        .contains("system gameplay.first already registered"));
}

#[test]
fn native_system_registration_reports_missing_required_resources() {
    let mut world = crate::scene::World::empty();
    let error = world
        .register_native_system::<crate::scene::ecs::ResParam<MissingScheduleResource>, _>(
            "gameplay.requires_missing_resource",
            SystemStage::Update,
            0,
            |_| {},
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("system gameplay.requires_missing_resource failed to initialize params"));
    assert!(error
        .to_string()
        .contains(std::any::type_name::<MissingScheduleResource>()));
}

#[derive(Debug, PartialEq, Eq)]
struct MissingScheduleResource;

impl crate::scene::ecs::Resource for MissingScheduleResource {}
```

### Lightweight Checks

- During implementation, a single scoped syntax/type check is allowed after all Milestone 1 slices compile locally in the editor: `cargo check -p zircon_runtime --lib --locked --message-format short`.
- Do not run per-slice tests before the testing stage unless a concrete compiler diagnostic requires a focused reproduction.

### Testing Stage

Run after all Milestone 1 slices are implemented:

```powershell
cargo test -p zircon_runtime --lib scene::tests::ecs_schedule --locked --message-format short
cargo check -p zircon_runtime --lib --locked --message-format short
```

Debug/correction loop:

- If `ScheduleError` display text regresses existing tests, restore the original display strings for existing variants before changing tests.
- If `World::register_native_system` hits borrow errors, keep the `std::mem::take` pattern so `SystemState<P>::new(self)` can initialize against the full world.
- If closure lifetime inference fails, make test closures explicitly typed by using `P = ()`, `P = LocalParam<T>`, or tuple param aliases rather than weakening persistent `SystemState` ownership.

### Exit Evidence

- `scene::tests::ecs_schedule` passes.
- `cargo check -p zircon_runtime --lib --locked --message-format short` passes or reports only unrelated pre-existing failures with evidence.
- Native system registry state remains skipped from `World` serialization by `serde(skip, default)`.

---

## Milestone 2: Schedule Runner Integration

### Goal

Run native ECS systems from `WorldDriver` in the same deterministic stage lane as built-ins and plugin hooks.

### In-Scope Behaviors

- `SceneScheduleRunner` merges built-ins, native systems, and plugin hooks by `order` then `id` inside each stage.
- Native systems execute with persistent state.
- Native systems with `CommandsParam` flush deferred commands before later ordered steps observe the world.
- Existing built-ins and plugin hooks retain their current dirty-state and deferred-command behavior.

### Dependencies

- Milestone 1 native system registry and world registration API.
- Existing `WorldDriver::tick_level` stage loop.

### Implementation Slices

- [ ] **Slice 1: Add scheduled step sorting helper**

Create `zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs`:

```rust
use crate::plugin::SceneRuntimeHookRegistration;
use crate::scene::ecs::{BoxedSceneSystem, SceneSystemDescriptor};

pub(crate) enum ScheduledSceneStep<'a> {
    Internal(SceneSystemDescriptor),
    Native(&'a mut BoxedSceneSystem),
    Hook(SceneRuntimeHookRegistration),
}

impl ScheduledSceneStep<'_> {
    pub(crate) fn order(&self) -> i32 {
        match self {
            Self::Internal(system) => system.order,
            Self::Native(system) => system.order(),
            Self::Hook(hook) => hook.descriptor().order,
        }
    }

    pub(crate) fn id(&self) -> &str {
        match self {
            Self::Internal(system) => system.id.as_str(),
            Self::Native(system) => system.id(),
            Self::Hook(hook) => hook.descriptor().id.as_str(),
        }
    }
}
```

- [ ] **Slice 2: Update `SceneScheduleRunner::run_stage`**

Modify `zircon_runtime/src/scene/ecs/schedule_runner.rs` so `run_stage` receives built-in descriptors and hooks as it does now, then borrows native systems through the world while executing the stage.

Target control flow:

```rust
let result = (|| {
    level.with_world_mut(|world| {
        let mut steps = internal_systems
            .into_iter()
            .filter(|system| system.stage == stage)
            .map(ScheduledSceneStep::Internal)
            .chain(
                world
                    .schedule
                    .native_systems_for_stage_mut(stage)
                    .map(ScheduledSceneStep::Native),
            )
            .chain(hooks.into_iter().map(ScheduledSceneStep::Hook))
            .collect::<Vec<_>>();
        steps.sort_by(|left, right| left.order().cmp(&right.order()).then(left.id().cmp(right.id())));
        steps
    })
})();
```

Do not keep the exact sketch if it creates an invalid borrow across `level.with_world_mut` and hook execution. The final implementation must avoid holding the `World` mutex while running plugin hooks that call `level.with_world_mut` again. Use this safe execution shape instead:

```text
1. Build a sorted list of step keys where native steps contain native system ids, not `&mut` references.
2. For each internal step, borrow world briefly and call `run_internal_scene_system`.
3. For each native step, borrow world mutably, look up the native system by id, temporarily remove it from the registry, run it, record whether it has deferred commands, insert it back, and apply deferred if required.
4. For each hook step, run the hook outside any active world borrow, then call `world.apply_deferred()` as the current runner does.
```

Add registry helpers for safe temporary native execution:

```rust
pub(crate) fn with_native_system_mut<R>(
    &mut self,
    id: &str,
    run: impl FnOnce(&mut BoxedSceneSystem) -> R,
) -> Option<R> {
    let index = self.native_systems.iter().position(|system| system.id() == id)?;
    Some(run(&mut self.native_systems[index]))
}
```

If running a native system needs `&mut World` at the same time as `&mut BoxedSceneSystem`, remove and reinsert the system:

```rust
pub(crate) fn take_native_system(&mut self, id: &str) -> Option<BoxedSceneSystem> {
    let index = self.native_systems.iter().position(|system| system.id() == id)?;
    Some(self.native_systems.remove(index))
}

pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem) {
    self.native_systems.push(system);
    sort_native_systems(&mut self.native_systems);
}
```

Use take/restore in the runner to satisfy borrowing rules.

- [ ] **Slice 3: Update `WorldDriver` built-in collection**

Modify `zircon_runtime/src/scene/module/world_driver.rs` only if needed. The existing `systems = world.schedule().systems().to_vec()` should continue to collect serializable built-ins. Native systems must stay inside `World` and should not be cloned.

Expected shape remains:

```rust
let (stages, systems) = level.with_world(|world| {
    (
        world.schedule().stages.clone(),
        world.schedule().systems().to_vec(),
    )
});
```

- [ ] **Slice 4: Add runner ordering tests**

Extend `zircon_runtime/src/scene/tests/ecs_schedule.rs` with a native/built-in/plugin ordering test:

```rust
#[test]
fn world_driver_orders_native_systems_with_plugin_hooks() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let events = Arc::new(Mutex::new(Vec::<String>::new()));

    level
        .with_world_mut(|world| {
            let first = events.clone();
            world
                .register_native_system::<(), _>(
                    "gameplay.native.before_hook",
                    SystemStage::Update,
                    -10,
                    move |_| first.lock().unwrap().push("native-before-hook".to_string()),
                )
                .unwrap();
            let second = events.clone();
            world
                .register_native_system::<(), _>(
                    "gameplay.native.after_hook",
                    SystemStage::Update,
                    10,
                    move |_| second.lock().unwrap().push("native-after-hook".to_string()),
                )
                .unwrap();
        });

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new(
                "weather.scene.update",
                "weather",
                SystemStage::Update,
            )
            .with_order(0),
            RecordingOrderHook { events: events.clone() },
        ))
        .unwrap();
    runtime.install_scene_runtime_hooks(&registry).unwrap();

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "native-before-hook".to_string(),
            "hook".to_string(),
            "native-after-hook".to_string(),
        ]
    );
}

#[derive(Debug)]
struct RecordingOrderHook {
    events: Arc<Mutex<Vec<String>>>,
}

impl SceneRuntimeHook for RecordingOrderHook {
    fn run(&self, _context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        self.events.lock().unwrap().push("hook".to_string());
        Ok(())
    }
}
```

If `RecordingOrderHook` duplicates existing helper types too much, keep it in the same test file because it is a narrow fixture for one behavior.

### Unit-Test Code To Add Before Testing Stage

Add to `zircon_runtime/src/scene/tests/ecs_systems.rs`:

```rust
#[test]
fn scheduled_native_system_keeps_local_state_between_ticks() {
    let mut world = World::empty();
    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_system = observed.clone();

    world
        .register_native_system::<LocalParam<LocalCounter>, _>(
            "gameplay.local_counter",
            crate::scene::ecs::SystemStage::Update,
            0,
            move |mut counter| {
                counter.0 += 1;
                observed_system.lock().unwrap().push(counter.0);
            },
        )
        .unwrap();

    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![1, 2]);
}
```

If `run_native_scene_systems_for_stage` does not exist after runner integration, add a crate-visible `World` helper used only by tests and `SceneScheduleRunner`:

```rust
pub(crate) fn run_native_scene_systems_for_stage(&mut self, stage: SystemStage) {
    let ids = self
        .schedule
        .native_systems_for_stage(stage)
        .map(|system| system.id().to_string())
        .collect::<Vec<_>>();
    for id in ids {
        self.run_native_scene_system_by_id(&id);
    }
}
```

### Lightweight Checks

- Use `cargo check -p zircon_runtime --lib --locked --message-format short` after Milestone 2 code and tests are written if no broad testing stage has started yet.

### Testing Stage

Run after all Milestone 2 slices are implemented:

```powershell
cargo test -p zircon_runtime --lib scene::tests::ecs_schedule --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_systems --locked --message-format short
```

Debug/correction loop:

- If plugin hook tests deadlock, inspect for a held world lock before hook execution and move hook execution outside `level.with_world_mut`.
- If native system state is lost between ticks, verify registry stores the boxed system persistently and the runner does not recreate it.
- If deferred commands apply too early, verify `CommandsParam` only queues during `SystemState::run` and `World::apply_deferred()` is called after the native system returns.

### Exit Evidence

- `scene::tests::ecs_schedule` proves native/built-in/plugin ordering.
- `scene::tests::ecs_systems` proves persistent state from scheduled native systems.
- Existing `world_driver_defers_hook_mutations_until_builtin_post_update_systems_run` and `world_driver_runs_native_render_extract_system_before_render_extract_hooks` remain passing.

---

## Milestone 3: Change Windows, Messages, Removed Components, And Deferred Sync

### Goal

Prove scheduled native systems use the same M5/M6 params correctly when run by the schedule path.

### In-Scope Behaviors

- `Added<T>` and `Changed<T>` filters observe per-system windows across scheduled ticks.
- `MessageReaderParam<T>` cursor state persists per native system.
- `RemovedComponentsParam<T>` cursor state persists per native system.
- A later ordered native system sees deferred command effects from an earlier native system that used `CommandsParam`.
- A native system does not see its own deferred commands before the sync point.

### Dependencies

- Milestone 2 schedule runner executes persistent native system objects.

### Implementation Slices

- [ ] **Slice 1: Add scheduled change-window test**

Extend `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs`:

```rust
#[test]
fn scheduled_native_system_uses_added_and_changed_windows() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()), Health(10))).unwrap();
    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<Vec<EntityId>>::new()));
    let observed_system = observed.clone();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    world
        .register_native_system::<ChangedHealth, _>(
            "gameplay.changed_health",
            crate::scene::ecs::SystemStage::Update,
            0,
            move |query| {
                observed_system
                    .lock()
                    .unwrap()
                    .push(query.iter().map(|(entity, _)| entity).collect());
            },
        )
        .unwrap();

    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.get_mut::<Health>(first).unwrap().0 += 1;
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![vec![first], vec![], vec![first]]);
}
```

- [ ] **Slice 2: Add scheduled message cursor test**

Extend `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs`:

```rust
#[test]
fn scheduled_native_message_reader_keeps_cursor() {
    let mut world = World::empty();
    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<Vec<u32>>::new()));
    let observed_system = observed.clone();

    type Reader = MessageReaderParam<HitMessage>;
    world
        .register_native_system::<Reader, _>(
            "gameplay.message_reader",
            crate::scene::ecs::SystemStage::Update,
            0,
            move |mut reader| {
                observed_system.lock().unwrap().push(
                    reader
                        .read()
                        .map(|(_, message)| message.0)
                        .collect::<Vec<_>>(),
                );
            },
        )
        .unwrap();

    world.send_message(HitMessage(1));
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.send_message(HitMessage(2));
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![vec![1], vec![], vec![2]]);
}

#[derive(Debug, PartialEq, Eq)]
struct HitMessage(u32);

impl crate::scene::ecs::Message for HitMessage {}
```

- [ ] **Slice 3: Add scheduled removed-components cursor test**

Extend `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs`:

```rust
#[test]
fn scheduled_native_removed_components_reader_keeps_cursor() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()), Health(1))).unwrap();
    let second = world.spawn((Name("Second".to_string()), Health(2))).unwrap();
    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<Vec<EntityId>>::new()));
    let observed_system = observed.clone();

    type RemovedHealth = RemovedComponentsParam<Health>;
    world
        .register_native_system::<RemovedHealth, _>(
            "gameplay.removed_health",
            crate::scene::ecs::SystemStage::Update,
            0,
            move |mut removed| {
                observed_system
                    .lock()
                    .unwrap()
                    .push(removed.read().collect::<Vec<_>>());
            },
        )
        .unwrap();

    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.remove::<Health>(first).unwrap();
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);
    world.remove::<Health>(second).unwrap();
    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![vec![], vec![first], vec![], vec![second]]);
}
```

- [ ] **Slice 4: Add scheduled deferred visibility test**

Extend `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs`:

```rust
#[test]
fn scheduled_native_commands_flush_before_later_ordered_systems() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Target".to_string()),)).unwrap();
    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<bool>::new()));

    world
        .register_native_system::<CommandsParam, _>(
            "gameplay.queue_marker",
            crate::scene::ecs::SystemStage::Update,
            -10,
            move |mut commands| {
                commands.entity(entity).insert((Marker,));
            },
        )
        .unwrap();

    let observed_system = observed.clone();
    type MarkerQuery = QueryState<&'static Marker>;
    world
        .register_native_system::<MarkerQuery, _>(
            "gameplay.observe_marker",
            crate::scene::ecs::SystemStage::Update,
            0,
            move |query| {
                observed_system
                    .lock()
                    .unwrap()
                    .push(query.iter().next().is_some());
            },
        )
        .unwrap();

    world.run_native_scene_systems_for_stage(crate::scene::ecs::SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![true]);
}
```

### Lightweight Checks

- Use one scoped check if compilation drift appears after test code is added: `cargo check -p zircon_runtime --lib --locked --message-format short`.

### Testing Stage

Run after all Milestone 3 slices are implemented:

```powershell
cargo test -p zircon_runtime --lib scene::tests::ecs_scheduled_native_systems --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_change_detection --locked --message-format short
```

Debug/correction loop:

- If `Changed<T>` reports every tick, inspect `SystemState::last_run` persistence inside `FunctionSceneSystem`.
- If message or removed cursors reread old events, inspect whether native systems are recreated or moved without preserving their boxed state.
- If deferred marker visibility fails, inspect `SceneScheduleRunner` native-system post-run flush.

### Exit Evidence

- Scheduled native systems prove parity with manual `SystemState` behavior for local state, messages, removed components, change windows, and deferred commands.

---

## Milestone 4: Documentation, Acceptance, And Promotion Gate

### Goal

Document the native ECS system layer and run the scoped acceptance gate for the runtime ECS milestone.

### In-Scope Behaviors

- Module docs explain native system ownership, runtime-only storage, registration API, ordering, deferred sync, and intentional divergence from Bevy.
- Acceptance evidence records exact commands and outcomes.
- Active coordination note is updated with final status or blocker evidence.

### Dependencies

- Milestones 1-3 implemented.

### Implementation Slices

- [ ] **Slice 1: Update ECS module docs header**

Modify `docs/zircon_runtime/scene/ecs.md` frontmatter:

- Add `zircon_runtime/src/scene/ecs/system/native/mod.rs` to `related_code` and `implementation_files`.
- Add each new native system file under `related_code` and `implementation_files`.
- Add `docs/superpowers/specs/2026-05-18-native-ecs-systems-design.md` and this implementation plan under `plan_sources`.
- Add `zircon_runtime/src/scene/tests/ecs_systems.rs`, `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs`, and `zircon_runtime/src/scene/tests/ecs_schedule.rs` under `tests` if not already present.

- [ ] **Slice 2: Update ECS docs body**

Add a section after `## System Params, Commands, And Change Detection` or before `## Schedule Model`:

```markdown
## Native ECS Systems

Native ECS systems are runtime-only schedule entries that wrap persistent `SystemState<P>` values. They let ordinary Rust closures consume `SystemParam` items from the same world authority used by manual systems, while preserving per-system state such as `LocalParam<T>`, message cursors, removed-component cursors, and change-detection windows.

`Schedule::register_native_system(...)` and `World::register_native_system(...)` validate the system id, initialize params once, and store the boxed system separately from serialized built-in `SceneSystemDescriptor` values. This keeps scene/project serialization stable: saved worlds still contain scene data, not runtime function objects.

`SceneScheduleRunner` executes built-in systems, native ECS systems, and plugin scene hooks in deterministic `stage -> order -> id` order. Native systems that declare `CommandsParam` flush through `World::apply_deferred()` after the system returns, so later ordered steps observe queued work while the running system itself still sees deferred commands as invisible.
```

Update `## Schedule Model` to mention native systems in the stage runner paragraph.

- [ ] **Slice 3: Update coordination note**

Modify `.codex/sessions/20260518-1848-ecs-continuation-design.md` with:

```markdown
- status: active-testing or completed, depending on validation result
- current step: Native ECS Systems implementation validation
- touched modules: list the new native system files plus schedule/registry/runner/tests/docs
- blockers: preserve root workspace and WSL blockers if they still apply
- next step: either fix validation failures or retire/archive this note after final user-visible closeout
```

### Testing Stage

Before testing, check free space for the active Cargo target drive. If using `E:\cargo-targets\zircon-native-ecs-systems` and `E:` free space is `<= 50 GB`, run cleanup for that target directory before Cargo commands.

Scoped acceptance commands:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-native-ecs-systems"
cargo test -p zircon_runtime --lib scene::tests::ecs_scheduled_native_systems --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_schedule --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_change_detection --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests --locked --message-format short
```

Optional broader validator for this crate-local runtime milestone if scoped tests pass and disk allows:

```powershell
.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir "E:\cargo-targets\zircon-native-ecs-systems"
```

Do not claim root workspace green unless these exact commands pass freshly:

```powershell
cargo build --workspace --locked --verbose
cargo test --workspace --locked --verbose
cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked --verbose
cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose
cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose
```

Debug/correction loop:

- If a high-level schedule test fails, first inspect lower native system registration and persistent state before changing plugin hook behavior.
- If serialization equality changes, inspect `serde(skip, default)` and manual `PartialEq` for `SceneSystemRegistry` before changing scene tests.
- If docs headers drift from implementation files, update `related_code`, `implementation_files`, `plan_sources`, and `tests` before closeout.

### Exit Evidence

- All scoped acceptance commands pass, or failures are recorded with exact command output and root cause classification.
- `docs/zircon_runtime/scene/ecs.md` reflects the implemented native system layer.
- `.codex/sessions/20260518-1848-ecs-continuation-design.md` is retired or updated with a clear handoff if validation remains blocked.

---

## Plan Self-Review

- Spec coverage: the plan covers native system object model, runtime-only storage, `Schedule::register_native_system(...)`, `World::schedule_mut()`, persistent `SystemState<P>`, deterministic merged stage execution, deferred command sync, structured registration errors, tests, docs, and validation.
- Scope control: the plan does not add parallel scheduling, schedule sets, run conditions, plugin-facing ECS registration, editor integration, dynamic scene serialization, or query performance rewrites.
- Type consistency: all new types live under `zircon_runtime::scene::ecs::system::native`; public re-exports flow through `system/mod.rs`, `ecs/mod.rs`, and `scene/mod.rs`; schedule errors route through `ScheduleError::SystemParam { system_id, source }`.
- Validation consistency: implementation slices do not require per-slice Cargo test loops; every milestone has a named testing stage with exact commands and correction rules.
