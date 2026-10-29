use std::time::Duration;

use serde_json::{json, Value};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::runtime::modules::{TasksModule, TimeModule};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::{
    RuntimeExtensionRegistry, RuntimePluginCatalog, RuntimePluginRegistrationReport,
};
use zircon_runtime::scene::{self, LevelSystem, NodeKind, World};

use crate::scene_system::{SIMULATION_MODULE, TICK_SYSTEM};
use crate::{
    runtime_plugin, ParticleRuntimeSnapshot, ParticleShape, ParticleSimulationBackend,
    ParticleSystemAsset, ParticleSystemComponent, ParticleVec3Range, ParticlesManager,
    ParticlesRuntimePlugin, PARTICLES_MODULE_NAME, PARTICLE_SYSTEM_COMPONENT_TYPE,
};

use super::support::spawn_rate_asset;

fn extensions(
    plugin: &ParticlesRuntimePlugin,
    target: RuntimeTargetMode,
) -> RuntimeExtensionRegistry {
    let mut report = RuntimePluginRegistrationReport::from_plugin(plugin);
    assert!(report.is_success(), "{:?}", report.diagnostics);
    report.project_selection.enabled = true;
    let manifest = ProjectPluginManifest {
        selections: vec![report.project_selection.clone()],
    };
    let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    let report = catalog.runtime_extensions_for_project(&manifest, target);
    assert!(!report.has_fatal_diagnostics(), "{:?}", report.diagnostics);
    report.registry.clone()
}

struct Fixture {
    runtime: CoreRuntime,
    level: LevelSystem,
    manager: ParticlesManager,
}

impl Fixture {
    fn new(plugin: &ParticlesRuntimePlugin, target: RuntimeTargetMode) -> Self {
        let registry = extensions(plugin, target);
        let runtime = CoreRuntime::new();
        // Use the real scene driver and its declared dependency spine, without
        // a renderer, project filesystem, editor host, or test clock shortcut.
        for descriptor in [
            TasksModule.descriptor(),
            TimeModule.descriptor(),
            zircon_runtime::asset::module_descriptor(),
            scene::module_descriptor(),
        ] {
            runtime.register_module(descriptor).unwrap();
        }
        for descriptor in registry.modules() {
            runtime.register_module(descriptor.clone()).unwrap();
        }
        runtime.activate_registered_modules().unwrap();
        scene::install_world_runtime_extension_plan(
            &runtime.handle(),
            registry.world_runtime_extension_plan().unwrap(),
        )
        .unwrap();
        let level =
            scene::create_level(&runtime.handle(), World::empty(), Default::default()).unwrap();
        Self {
            runtime,
            level,
            manager: plugin.manager(),
        }
    }

    fn client(plugin: &ParticlesRuntimePlugin) -> Self {
        Self::new(plugin, RuntimeTargetMode::ClientRuntime)
    }

    fn tick(&self) {
        tick(&self.runtime, &self.level);
    }

    fn snapshot(&self) -> ParticleRuntimeSnapshot {
        self.manager.snapshot()
    }

    fn end(&self) {
        self.runtime
            .deactivate_module(PARTICLES_MODULE_NAME)
            .unwrap();
    }
}

fn tick(runtime: &CoreRuntime, level: &LevelSystem) {
    let frame = runtime.advance_time_by(Duration::from_millis(250), 1);
    level.tick(&runtime.handle(), frame).unwrap();
}

fn add(level: &LevelSystem, value: Value) -> u64 {
    level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .set_dynamic_component(entity, PARTICLE_SYSTEM_COMPONENT_TYPE, value)
            .unwrap();
        entity
    })
}

fn set(level: &LevelSystem, entity: u64, value: Value) {
    level.with_world_mut(|world| {
        world
            .set_dynamic_component(entity, PARTICLE_SYSTEM_COMPONENT_TYPE, value)
            .unwrap();
    });
}

fn component() -> Value {
    json!({"asset": spawn_rate_asset(8.0, 16)})
}

#[test]
fn scene_components_create_reuse_and_tick_deterministically_through_runtime_schedule() {
    let fixture = Fixture::client(&runtime_plugin());
    let mut asset = spawn_rate_asset(8.0, 16).with_seed(123);
    asset.emitters[0].shape = ParticleShape::Sphere { radius: 0.5 };
    asset.emitters[0].initial_velocity = ParticleVec3Range::constant(Vec3::X);
    let entity = add(&fixture.level, json!({"asset": asset}));
    let control = ParticlesManager::default();
    control
        .instantiate(ParticleSystemComponent::new(entity, asset))
        .unwrap();
    assert!(fixture.snapshot().emitters.is_empty());

    let mut handle = None;
    let mut first_position = None;
    for frame in 1..=4 {
        fixture.tick();
        control.tick(0.25).unwrap();
        let snapshot = fixture.snapshot();
        assert_eq!(snapshot.emitters.len(), 1);
        assert_eq!(snapshot.emitters[0].entity, entity);
        assert_eq!(snapshot.emitters[0].live_particles, frame * 2);
        assert_eq!(snapshot.sprites, control.snapshot().sprites);
        if let Some(handle) = handle {
            assert_eq!(snapshot.emitters[0].handle, handle);
        } else {
            handle = Some(snapshot.emitters[0].handle);
            first_position = Some(snapshot.sprites[0].position);
        }
    }
    assert_ne!(
        fixture.snapshot().sprites[0].position,
        first_position.unwrap()
    );
    fixture.end();
    assert!(fixture.snapshot().emitters.is_empty());
}

#[test]
fn scene_virtual_pause_preserves_particle_state_until_runtime_resumes() {
    let fixture = Fixture::client(&runtime_plugin());
    add(&fixture.level, component());
    fixture.tick();
    let before = fixture.snapshot();
    fixture.level.pause_virtual_time();
    for _ in 0..3 {
        fixture.tick();
    }
    assert_eq!(fixture.snapshot(), before);
    fixture.level.unpause_virtual_time();
    fixture.tick();
    assert_eq!(fixture.snapshot().emitters[0].live_particles, 4);
    fixture.end();
}

#[test]
fn scene_disable_component_removal_and_entity_removal_release_instances() {
    let fixture = Fixture::client(&runtime_plugin());
    let mut value = component();
    value["playing"] = json!(false);
    let entity = add(&fixture.level, value.clone());
    fixture.tick();
    assert!(fixture.snapshot().emitters.is_empty());

    let mut previous = None;
    for removal in 0..3 {
        value["playing"] = json!(true);
        set(&fixture.level, entity, value.clone());
        fixture.tick();
        let snapshot = fixture.snapshot();
        let handle = snapshot.emitters[0].handle;
        assert_eq!(snapshot.emitters[0].live_particles, 2);
        assert_ne!(Some(handle), previous);
        match removal {
            0 => {
                value["playing"] = json!(false);
                set(&fixture.level, entity, value.clone());
            }
            1 => fixture.level.with_world_mut(|world| {
                world
                    .remove_dynamic_component(entity, PARTICLE_SYSTEM_COMPONENT_TYPE)
                    .unwrap();
            }),
            _ => fixture
                .level
                .with_world_mut(|world| world.remove_entity(entity).unwrap()),
        }
        fixture.tick();
        assert!(fixture.snapshot().emitters.is_empty());
        assert!(fixture.snapshot().sprites.is_empty());
        assert!(fixture.manager.play(handle).is_err());
        previous = Some(handle);
    }
    fixture.end();
}

#[test]
fn scene_inactive_hierarchy_releases_emitters_and_reactivation_starts_fresh() {
    let fixture = Fixture::client(&runtime_plugin());
    let child = add(&fixture.level, component());
    let parent = fixture.level.with_world_mut(|world| {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world.set_parent_checked(child, Some(parent)).unwrap();
        world.set_active_self(parent, false).unwrap();
        parent
    });
    fixture.tick();
    assert!(fixture.snapshot().emitters.is_empty());
    fixture
        .level
        .with_world_mut(|world| world.set_active_self(parent, true).unwrap());
    fixture.tick();
    let handle = fixture.snapshot().emitters[0].handle;
    fixture
        .level
        .with_world_mut(|world| world.set_active_self(parent, false).unwrap());
    fixture.tick();
    assert!(fixture.snapshot().emitters.is_empty());
    fixture
        .level
        .with_world_mut(|world| world.set_active_self(parent, true).unwrap());
    fixture.tick();
    assert_ne!(fixture.snapshot().emitters[0].handle, handle);
    assert_eq!(fixture.snapshot().emitters[0].live_particles, 2);
    fixture.end();
}

#[test]
fn scene_transform_time_scale_seed_and_backend_edits_reconcile_without_duplicate_instances() {
    let fixture = Fixture::client(&runtime_plugin());
    let entity = add(&fixture.level, component());
    fixture.tick();
    let first = fixture.snapshot().emitters[0].handle;
    let mut value = component();
    value["time_scale"] = json!(0.5);
    set(&fixture.level, entity, value.clone());
    fixture.level.with_world_mut(|world| {
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            )
            .unwrap();
    });
    fixture.tick();
    let snapshot = fixture.snapshot();
    assert_eq!(snapshot.emitters.len(), 1);
    assert_eq!(snapshot.emitters[0].handle, first);
    assert_eq!(snapshot.emitters[0].live_particles, 3);
    assert!(snapshot
        .sprites
        .iter()
        .all(|sprite| sprite.position.x >= 10.0));

    value["seed"] = json!(42);
    value["backend"] = json!("gpu");
    set(&fixture.level, entity, value);
    fixture.tick();
    let snapshot = fixture.snapshot();
    assert_eq!(snapshot.emitters.len(), 1);
    assert_ne!(snapshot.emitters[0].handle, first);
    assert_eq!(snapshot.emitters[0].live_particles, 1);
    assert_eq!(snapshot.emitters[0].backend, ParticleSimulationBackend::Gpu);
    assert!(snapshot.emitters[0].fallback_to_cpu);
    assert!(fixture.manager.play(first).is_err());
    fixture.end();
}

#[test]
fn scene_ticks_and_world_replacement_preserve_other_worlds_and_manual_emitters() {
    let fixture = Fixture::client(&runtime_plugin());
    let manual = fixture
        .manager
        .instantiate(ParticleSystemComponent::new(999, spawn_rate_asset(8.0, 16)))
        .unwrap();
    add(&fixture.level, component());
    fixture.tick();
    let first = fixture.snapshot().emitters[1].handle;
    let second_level = scene::create_level(
        &fixture.runtime.handle(),
        World::empty(),
        Default::default(),
    )
    .unwrap();
    add(&second_level, component());
    let frame = fixture
        .runtime
        .advance_time_by(Duration::from_millis(250), 1);
    second_level.tick(&fixture.runtime.handle(), frame).unwrap();
    let second = fixture.snapshot().emitters[2].handle;
    assert_eq!(fixture.snapshot().emitters[1].live_particles, 2);
    fixture
        .level
        .tick(&fixture.runtime.handle(), frame)
        .unwrap();
    let snapshot = fixture.snapshot();
    for (handle, expected) in [(manual, 0), (first, 4), (second, 2)] {
        assert_eq!(
            snapshot
                .emitters
                .iter()
                .find(|row| row.handle == handle)
                .unwrap()
                .live_particles,
            expected
        );
    }
    fixture
        .level
        .replace_world_and_reset_runtime_state(World::empty());
    let snapshot = fixture.snapshot();
    assert!(!snapshot.emitters.iter().any(|row| row.handle == first));
    assert_eq!(snapshot.emitters.len(), 2);
    fixture.end();
    let snapshot = fixture.snapshot();
    assert_eq!(snapshot.emitters.len(), 1);
    assert_eq!(snapshot.emitters[0].handle, manual);
    fixture.manager.remove(manual).unwrap();
}

#[test]
fn scene_session_shutdown_is_scoped_and_repeated_sessions_do_not_leak() {
    let plugin = runtime_plugin();
    let first = Fixture::client(&plugin);
    add(&first.level, component());
    first.tick();
    let first_handle = first.snapshot().emitters[0].handle;
    let second = Fixture::client(&plugin);
    add(&second.level, component());
    second.tick();
    let second_handle = second.snapshot().emitters[1].handle;
    first.end();
    let snapshot = second.snapshot();
    assert_eq!(snapshot.emitters.len(), 1);
    assert_eq!(snapshot.emitters[0].handle, second_handle);
    drop(first);
    second.tick();
    assert_eq!(second.snapshot().emitters[0].live_particles, 4);
    second.end();
    assert!(plugin.manager().snapshot().emitters.is_empty());
    drop(second);

    let mut previous = second_handle;
    for _ in 0..3 {
        let session = Fixture::client(&plugin);
        add(&session.level, component());
        session.tick();
        let snapshot = session.snapshot();
        assert_eq!(snapshot.emitters.len(), 1);
        assert_eq!(snapshot.emitters[0].live_particles, 2);
        assert_ne!(snapshot.emitters[0].handle, previous);
        assert_ne!(snapshot.emitters[0].handle, first_handle);
        previous = snapshot.emitters[0].handle;
        session.end();
        assert!(
            session.snapshot().emitters.is_empty(),
            "cleanup must work with Level still retained"
        );
        assert!(session.snapshot().sprites.is_empty());
    }
}

#[test]
fn scene_system_target_filter_runs_client_simulation_and_keeps_editor_authoring_idle() {
    let plugin = runtime_plugin();
    for (target, expected) in [
        (RuntimeTargetMode::ClientRuntime, true),
        (RuntimeTargetMode::EditorHost, false),
    ] {
        let registry = extensions(&plugin, target);
        assert_eq!(
            registry.plugin_runtime_systems().any(|(owner, system)| {
                system.id == TICK_SYSTEM
                    && registry.plugin_module_name(owner) == Some(SIMULATION_MODULE)
            }),
            expected
        );
        assert!(registry
            .components()
            .iter()
            .any(|row| row.type_id == PARTICLE_SYSTEM_COMPONENT_TYPE));
        assert!(!registry.plugin_options().is_empty());
        let fixture = Fixture::new(&plugin, target);
        add(&fixture.level, component());
        fixture.tick();
        assert_eq!(!fixture.snapshot().emitters.is_empty(), expected);
        fixture.end();
    }
}

#[test]
fn scene_invalid_or_missing_assets_fail_without_spawning_defaults_or_leaking_old_instances() {
    let fixture = Fixture::client(&runtime_plugin());
    let entity = add(&fixture.level, component());
    fixture.tick();
    for value in [
        json!({}),
        json!({"asset": "res://unsupported.particles"}),
        json!({"asset": {"emitters": []}}),
    ] {
        set(&fixture.level, entity, value);
        let frame = fixture
            .runtime
            .advance_time_by(Duration::from_millis(250), 1);
        assert!(fixture
            .level
            .tick(&fixture.runtime.handle(), frame)
            .is_err());
        assert!(fixture.snapshot().emitters.is_empty());
        assert!(fixture.snapshot().sprites.is_empty());
    }
    set(&fixture.level, entity, component());
    fixture.tick();
    assert_eq!(fixture.snapshot().emitters[0].live_particles, 2);
    fixture.end();
}

#[test]
fn scene_inline_asset_roundtrip_preserves_existing_particle_defaults() {
    let asset = ParticleSystemAsset::default();
    let encoded = serde_json::to_value(&asset).unwrap();
    assert_eq!(
        serde_json::from_value::<ParticleSystemAsset>(encoded).unwrap(),
        asset
    );
    let partial: ParticleSystemAsset = serde_json::from_value(json!({"id": "inline"})).unwrap();
    assert_eq!(partial, ParticleSystemAsset::new("inline"));
}
