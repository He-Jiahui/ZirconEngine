---
related_code:
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/scene/world/messages.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/ecs/query/query_state.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/ecs/commands/commands.rs
  - zircon_runtime/src/scene/ecs/system/res.rs
  - zircon_runtime/src/scene/ecs/system/messages.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/observer.rs
  - zircon_runtime/src/scene/ecs/messages.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/system/query.rs
  - dev/bevy/crates/bevy_ecs/src/system/commands/mod.rs
  - dev/bevy/crates/bevy_ecs/src/query/filter.rs
  - dev/bevy/crates/bevy_ecs/src/observer/mod.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/component.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_transform/src/systems.rs
implementation_files:
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/scene/world/messages.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/ecs/query/query_state.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/ecs/commands/commands.rs
  - zircon_runtime/src/scene/ecs/system/res.rs
  - zircon_runtime/src/scene/ecs/system/messages.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/observer.rs
  - zircon_runtime/src/scene/ecs/messages.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
plan_sources:
  - user: 2026-05-08 Bevy-grade ECS / Reflect / Scene / Transform roadmap implementation
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - docs/zircon_runtime/scene/ecs.md
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
tests:
  - zircon_runtime/src/scene/tests/ecs_typed_api.rs
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/scene/tests/ecs_observers_messages.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - cargo check -p zircon_runtime --lib --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never
  - cargo test -p zircon_runtime --lib scene::tests::ecs --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib scene::tests --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture
  - cargo build --workspace --locked --verbose --jobs 1 --target-dir E:\cargo-targets\zircon-m12-ci-gate --color never
  - cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir E:\cargo-targets\zircon-m12-plugin-ci --color never
doc_type: module-detail
---

# Bevy-Parity ECS Examples

This document records the M12 examples for the Bevy-grade ECS / Reflect / Scene / Transform roadmap. They are examples of the supported Zircon API shape, not new API promises. When Bevy and Zircon diverge, the rule is explicit: Bevy supplies the capability model, while `zircon_runtime::scene::World` remains the public world authority and stable `EntityId = u64` remains the editor/asset identity.

## Bevy Source Anchors

| Capability | Bevy anchor | Zircon equivalent |
| --- | --- | --- |
| World and typed spawn | `dev/bevy/crates/bevy_ecs/src/world/mod.rs:98` | `World::spawn`, `World::insert`, `World::get`, `World::get_mut`, and fixed-component setters over the same component registry |
| Query and run-window filters | `dev/bevy/crates/bevy_ecs/src/system/query.rs:487`, `dev/bevy/crates/bevy_ecs/src/query/filter.rs:956` | `QueryState`, system `Query`, `With<T>`, `Without<T>`, `Added<T>`, `Changed<T>`, and cached query state |
| Deferred commands | `dev/bevy/crates/bevy_ecs/src/system/commands/mod.rs:104` | `CommandsParam`, `World::commands()`, `apply_deferred()`, and typed resource insertion |
| Observers and messages | `dev/bevy/crates/bevy_ecs/src/observer/mod.rs:1` | immediate lifecycle/entity observers plus scheduled `Messages<T>` readers/writers |
| Reflect component routing | `dev/bevy/crates/bevy_reflect/src/type_registry.rs:29`, `dev/bevy/crates/bevy_ecs/src/reflect/component.rs:81` | runtime `TypeRegistry`, `ReflectComponent`, `ReflectResource`, `WorldReflection`, fixed adapters, and dynamic plugin JSON adapters |
| Scene and transform | `dev/bevy/crates/bevy_scene/src/scene.rs:49`, `dev/bevy/crates/bevy_transform/src/systems.rs:13` | `DynamicScene`, `SceneAssetSerializer`, fixed scene product maps, typed ECS runtime state, and scene transform systems |

## Typed Components And Queries

Bevy-style spawn/query code maps to typed Zircon components under the public scene world. The stable `EntityId` is returned directly; the internal generational entity is still hidden inside the ECS kernel.

```rust
use zircon_runtime::scene::components::Name;
use zircon_runtime::scene::ecs::{Component, QueryState, SystemState, With};
use zircon_runtime::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);
impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Player;
impl Component for Player {}

let mut world = World::empty();
let player = world
    .spawn((Name("Player".to_string()), Health(10), Player))
    .unwrap();
let enemy = world
    .spawn((Name("Enemy".to_string()), Health(4)))
    .unwrap();

type PlayerHealth = QueryState<(EntityId, &'static Health), With<Player>>;
let mut system = SystemState::<PlayerHealth>::new(&mut world).unwrap();

let player_rows = system.run(&mut world, |query| {
    query
        .iter()
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>()
});

assert_eq!(player_rows, vec![(player, 10)]);
assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
```

The same pattern is covered by `ecs_typed_api.rs`, `ecs_query.rs`, `ecs_systems.rs`, and the M11 query-cache tests. M12 keeps fixed component setters as product-state APIs, but new system-facing behavior should enter through typed components and queries rather than new fixed-map loops.

## Deferred Commands And Resources

Zircon's `CommandsParam` follows Bevy's deferred-command shape: a system can queue entity/resource mutation, and the world applies it at the explicit deferred boundary.

```rust
use zircon_runtime::scene::components::Name;
use zircon_runtime::scene::ecs::{
    CommandsParam, Component, QueryState, ResMutParam, Resource, SystemState, With,
};
use zircon_runtime::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);
impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Player;
impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct Score(u32);
impl Resource for Score {}

let mut world = World::empty();
world.insert_resource(Score(1));
let player = world
    .spawn((Name("Player".to_string()), Health(10), Player))
    .unwrap();
let enemy = world
    .spawn((Name("Enemy".to_string()), Health(4)))
    .unwrap();

type Params = (
    QueryState<&'static mut Health, With<Player>>,
    ResMutParam<Score>,
    CommandsParam,
);
let mut system = SystemState::<Params>::new(&mut world).unwrap();

system.run(&mut world, |(mut health_query, mut score, mut commands)| {
    health_query.for_each_mut(|health| health.0 += 2);
    score.0 += 1;
    commands.insert(enemy, Player);
});

assert_eq!(world.get::<Health>(player), Some(&Health(12)));
assert!(world.get::<Player>(enemy).is_none());
world.apply_deferred();
assert_eq!(world.get::<Player>(enemy), Some(&Player));
assert_eq!(world.resource::<Score>(), &Score(2));
```

This example is intentionally explicit about `apply_deferred()`. Zircon's scene driver can own that boundary in scheduled execution, but manual world examples should show the boundary instead of hiding it.

## Change Detection, Observers, And Messages

`Changed<T>` and `Added<T>` use component ticks and each `SystemState` run window, so repeated reads do not keep reporting the same change. Immediate observers are separate from scheduled messages:

- `World::observe_component_lifecycle::<T>(...)` and `World::observe_entity_event::<T>(...)` run callbacks during the trigger or mutation path.
- `World::send_message(...)`, `MessageWriterParam<T>`, and `MessageReaderParam<T>` use retained cursors so a system reads only messages it has not consumed.
- `RemovedComponentsParam<T>` observes component removals and recursive despawn cleanup without requiring the removed value to remain stored.

The coverage lives in `ecs_change_detection.rs` and `ecs_observers_messages.rs`. The Bevy comparison point is capability behavior, not identical observer storage: Zircon keeps the observer store behind `World`, and editor/remote reflection still routes through `WorldReflection`.

## Scene, Reflect, And Transform Boundaries

Bevy's current scene source models scene composition through `Scene` and `ResolvedScene`; Zircon's current route is product-state serialization plus reflection-aware dynamic scene application:

- `SceneAssetSerializer` bridges `SceneAsset` and `World` for project load/save.
- `DynamicScene` and scene patches apply reflected component values through the world-owned registry.
- fixed scene maps keep serialized product fields such as stable nodes, hierarchy rows, local transforms, render layers, and mobility.
- typed ECS storage keeps runtime-only component identity, resources, events/messages, change ticks, query caches, and schedule state.
- `WorldMatrix`, `ActiveInHierarchy`, and `node_cache` are derived caches; they are rebuilt from input state and are not serialized truth.

This is the concrete M12 cutover rule for examples: show typed ECS for runtime behavior, show fixed maps only where the product format still owns data, and never present `WorldReflection` as an independent storage layer.

## Validation Notes

The examples above are grounded in existing focused tests rather than new tutorial-only code paths. The latest recorded M12 scene/ECS gate for this dirty workspace is:

- `cargo test -p zircon_runtime --lib scene::tests::ecs --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture`: `145 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib scene::tests --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture`: `179 passed; 0 failed`.
- `cargo check -p zircon_runtime --lib --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never`: passed with remaining warnings outside the scene/ECS surface.

Full workspace CI is blocked in this dirty checkout by an unrelated editor UI/material primitive compile error, not by scene/ECS. `cargo build --workspace --locked --verbose --jobs 1 --target-dir E:\cargo-targets\zircon-m12-ci-gate --color never` reached `zircon_editor` and failed with `E0689` at `zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/chip.rs:157` (`.min(...)` on an ambiguous float). `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir E:\cargo-targets\zircon-m12-plugin-ci --color never` hit the same editor dependency failure before plugin-specific validation could complete.
