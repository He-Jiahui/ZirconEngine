//! #199 — exact-main registered Jolt probe, not a synthetic scene applier call.
use crate::{
    build_world_sync_state, module_descriptor, register_runtime_systems, BodyHandle,
    DefaultPhysicsManager, PhysicsBackend, DEFAULT_PHYSICS_MANAGER_NAME, PHYSICS_MODULE_NAME,
    PLUGIN_RUNTIME_MODULE_NAME,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPathIdentity};
use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsBodySyncState, PhysicsManager, PhysicsSettings,
    PhysicsSimulationMode,
};
use zircon_runtime::core::framework::scene::physics::PhysicsSleepPolicy;
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle, SCENE_MODULE_NAME};
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::core::runtime::modules::{TasksModule, TimeModule};
use zircon_runtime::core::{CoreError, CoreRuntime, TimePolicyTransaction};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::plugin::RuntimeExtensionRegistry;
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType, WorldMatrix,
};
use zircon_runtime::scene::{
    create_default_level, install_world_runtime_extension_plan, FixedStepFailurePhase,
    LevelTickError, SystemStage,
};

const DT: Duration = Duration::from_nanos(15_625_000);
const POOL: &str = "F:/cargo-targets/zircon-local/windows/ue-source-regressions";
const INITIAL: Vec3 = Vec3::new(2.0, -1.0, 3.0);
const VELOCITY: Vec3 = Vec3::new(64.0, 32.0, 0.0);
const EPS: f32 = 1e-5;

#[test]
fn moving_root_commits_both_outer_frames() {
    run("moving", VELOCITY, &[1, 1]);
}

#[test]
fn moving_root_commits_two_steps_in_one_outer_frame() {
    run("burst", VELOCITY, &[2]);
}

#[test]
fn resting_root_commits_without_an_authored_revision() {
    run("rest", Vec3::ZERO, &[1, 1]);
}

fn run(label: &str, velocity: Vec3, frames: &[u32]) {
    let config = OwnedConfig::new();
    let runtime = new_registered_runtime(config.root.operation_path().join("config.json"));
    let core = runtime.handle();
    let manager = core
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
    manager
        .store_settings(PhysicsSettings {
            backend: "jolt".into(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&core).unwrap();
    let handle = level.world_handle();
    let authored = Transform::from_translation(INITIAL);
    let shape = ColliderComponent {
        shape: ColliderShape::Sphere { radius: 0.25 },
        ..ColliderComponent::default()
    };
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world.update_transform(entity, authored).unwrap();
        world
            .set_rigid_body(
                entity,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: velocity,
                    angular_velocity: Vec3::ZERO,
                    gravity_scale: 0.0,
                    linear_damping: 0.0,
                    angular_damping: 0.0,
                    sleep_policy: PhysicsSleepPolicy::Never,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        world.set_collider(entity, Some(shape.clone())).unwrap();
        entity
    });
    let start = level.with_world(|world| build_world_sync_state(handle, world));
    assert_eq!((start.bodies.len(), start.colliders.len()), (1, 1));
    assert_eq!(start.bodies[0].transform, authored);
    assert_eq!(start.bodies[0].transform, start.colliders[0].transform);
    // Admit a real body at zero solver time so its identity is known before any registered step.
    manager.sync_world(start);
    let (original_key, original) = read_native(&manager, handle, entity);
    assert_eq!(original.transform, authored);
    let epoch = level.capture_world_replacement_epoch();
    let initial_gen = level.world_generation();
    let mut prior_commits = 0_u32;
    let mut prior_gen = initial_gen;
    for (index, &count) in frames.iter().enumerate() {
        let result = level.tick(&core, runtime.advance_time_by(DT * count, count));
        assert_eq!(level.world_handle(), handle);
        assert_eq!(level.capture_world_replacement_epoch(), epoch);
        let plan = level
            .last_physics_step_plan()
            .expect("real registered solver stage recorded a step");
        assert_eq!(plan.steps, 1);
        assert!((plan.step_seconds - DT.as_secs_f32()).abs() < EPS);
        let (key, native) = read_native(&manager, handle, entity);
        assert_eq!(key, original_key);
        assert!(Vec3::from_array(native.linear_velocity).distance(velocity) < EPS);
        assert!(Vec3::from_array(native.angular_velocity).length() < EPS);
        assert_eq!(native.transform.rotation, Quat::IDENTITY);
        assert_eq!(native.transform.scale, Vec3::ONE);
        let t = native.transform.translation;
        let (generation, scene, published) = level.with_world(|world| {
            assert_eq!(world.parent_of(entity), None);
            assert_eq!(world.collider(entity), Some(&shape));
            let local = world.local_transform(entity).unwrap();
            let projected = world.world_matrix(entity).unwrap();
            assert_eq!(local, native.transform);
            assert!(
                projected
                    .w_axis
                    .truncate()
                    .distance(native.transform.translation)
                    < EPS
            );
            assert_eq!(
                [
                    projected.x_axis.truncate(),
                    projected.y_axis.truncate(),
                    projected.z_axis.truncate()
                ],
                [Vec3::X, Vec3::Y, Vec3::Z]
            );
            (
                world.world_generation(),
                local.translation,
                world.get::<WorldMatrix>(entity).map(|p| p.0),
            )
        });
        let clock = level.world_time().fixed_time();
        let interp = level.fixed_interpolation_context();
        if let Err(error) = result {
            let LevelTickError::FixedStep {
                receipt,
                source: CoreError::Initialization(owner, detail),
            } = error
            else {
                panic!("unexpected fixed-step route: {error:?}");
            };
            assert_eq!(owner, "WorldDriver fixed-step world generation");
            assert_eq!(detail, format!("World generation changed during FixedPostUpdate: expected {prior_gen}, found {generation}"));
            assert_eq!(
                receipt.phase(),
                FixedStepFailurePhase::Stage(SystemStage::FixedPostUpdate)
            );
            assert_eq!(receipt.system_id(), None);
            assert_eq!(receipt.tick().tick_index(), u64::from(prior_commits) + 1);
            assert_eq!(receipt.tick().world_generation(), prior_gen);
            assert_eq!(receipt.observed_world_generation(), generation);
            assert_eq!(generation, prior_gen + 1);
            assert_eq!(receipt.committed_steps(), 0);
            assert_eq!(receipt.remaining_debt(), DT * count);
            assert_eq!(clock.frame_index(), u64::from(prior_commits));
            assert_eq!(clock.elapsed(), DT * prior_commits);
            assert_eq!(clock.overstep(), DT * count);
            assert_eq!(interp.remaining_debt(), DT * count);
            assert_eq!(interp.current().elapsed(), clock.elapsed());
            assert!(
                t.distance(INITIAL + velocity * (DT * (prior_commits + 1)).as_secs_f32()) < EPS
            );
            println!("FIXED_NATIVE label={label} outer={} status=RevisionRejected generation={prior_gen}->{generation} fixed={} debt_ns={} same_handle=true native={:?} scene={:?}", index + 1, clock.frame_index(), clock.overstep().as_nanos(), t.to_array(), scene.to_array());
            panic!("owning World writes must not abort a fixed step: {label}");
        }
        prior_commits += count;
        assert!(t.distance(INITIAL + velocity * (DT * prior_commits).as_secs_f32()) < EPS);
        assert!(published.unwrap().w_axis.truncate().distance(t) < EPS);
        assert_eq!(
            generation,
            initial_gen
                + if velocity == Vec3::ZERO {
                    0
                } else {
                    u64::from(prior_commits)
                }
        );
        assert_eq!(clock.frame_index(), u64::from(prior_commits));
        assert_eq!(clock.elapsed(), DT * prior_commits);
        assert_eq!(clock.overstep(), Duration::ZERO);
        assert_eq!(interp.remaining_debt(), Duration::ZERO);
        assert_eq!(interp.current().elapsed(), clock.elapsed());
        assert_eq!(
            interp.current().simulation_tick().unwrap().tick_index(),
            u64::from(prior_commits)
        );
        assert_eq!(interp.previous().elapsed(), DT * (prior_commits - 1));
        println!("FIXED_NATIVE label={label} outer={} status=Committed generation={prior_gen}->{generation} fixed={} debt_ns=0 same_handle=true native={:?} scene={:?}", index + 1, clock.frame_index(), t.to_array(), scene.to_array());
        prior_gen = generation;
    }
}

fn read_native(
    manager: &DefaultPhysicsManager,
    handle: WorldHandle,
    entity: EntityId,
) -> (BodyHandle, PhysicsBodySyncState) {
    let status = manager.backend_status();
    assert_eq!(
        (status.state, status.active_backend.as_deref()),
        (PhysicsBackendState::Ready, Some("jolt"))
    );
    let snapshot = manager.synchronized_world_snapshot(handle).unwrap();
    assert_eq!(
        (
            snapshot.world,
            snapshot.bodies.len(),
            snapshot.colliders.len()
        ),
        (handle, 1, 1)
    );
    assert_eq!(snapshot.bodies[0].entity, entity);
    let mut worlds = manager.jolt_worlds.lock().unwrap();
    let native = worlds.get_mut(&handle).unwrap();
    let expected = native.entities[&entity].body;
    let mut active = Vec::new();
    native.backend.read_active_states(&mut active);
    assert_eq!(active.len(), 1);
    let (handle, row) = active.pop().unwrap();
    assert_eq!(handle, expected);
    assert_eq!(row.transform, snapshot.bodies[0].transform);
    assert_eq!(row.linear_velocity, snapshot.bodies[0].linear_velocity);
    assert_eq!(row.angular_velocity, snapshot.bodies[0].angular_velocity);
    (handle, row)
}

fn new_registered_runtime(config_path: std::path::PathBuf) -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .apply_time_policy(TimePolicyTransaction::new(
            runtime.time_policy().with_fixed_timestep(DT),
        ))
        .unwrap();
    let mut foundation = zircon_runtime::foundation::module_descriptor();
    zircon_runtime::foundation::bind_config_file_path(&mut foundation, config_path).unwrap();
    for descriptor in [
        TasksModule.descriptor(),
        TimeModule.descriptor(),
        zircon_runtime::asset::module_descriptor(),
        foundation,
        zircon_runtime::scene::module_descriptor(),
        module_descriptor(),
    ] {
        runtime.register_module(descriptor).unwrap();
    }
    let mut registry = RuntimeExtensionRegistry::default();
    registry.register_module(module_descriptor()).unwrap();
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(&mut registry)
        .module(PLUGIN_RUNTIME_MODULE_NAME)
        .unwrap();
    register_runtime_systems(&mut module).unwrap();
    install_world_runtime_extension_plan(
        &runtime.handle(),
        registry.world_runtime_extension_plan().unwrap(),
    )
    .unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime.activate_module(PHYSICS_MODULE_NAME).unwrap();
    runtime
}

struct OwnedConfig {
    root: ResolvedProjectPathIdentity,
}

impl OwnedConfig {
    fn new() -> Self {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let pool = ProjectPaths::resolve_existing(POOL).expect("existing physical F evidence pool");
        let display = ProjectPaths::display_path(pool.operation_path());
        assert!(display
            .to_str()
            .unwrap()
            .replace('\\', "/")
            .eq_ignore_ascii_case(POOL));
        let pool = ProjectPaths::resolve_identity(pool.operation_path()).unwrap();
        let binary = std::env::current_exe().unwrap();
        assert!(ProjectPaths::resolve_identity(binary.parent().unwrap())
            .unwrap()
            .is_within(&pool));
        let tmp = ProjectPaths::resolve_existing(pool.operation_path().join("tmp")).unwrap();
        let tmp = ProjectPaths::resolve_identity(tmp.operation_path()).unwrap();
        assert!(tmp != pool && tmp.is_within(&pool));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(tmp.operation_path().join(format!(
            "fixed-step-revision-{}-{now}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        )))
        .unwrap();
        assert!(root != tmp && root.is_within(&tmp));
        std::fs::create_dir(root.operation_path()).unwrap();
        let owned = Self { root };
        assert_eq!(
            ProjectPaths::resolve_identity(owned.root.operation_path()).unwrap(),
            owned.root
        );
        owned
    }
}

impl Drop for OwnedConfig {
    fn drop(&mut self) {
        assert_eq!(
            ProjectPaths::resolve_identity(self.root.operation_path()).unwrap(),
            self.root
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}
