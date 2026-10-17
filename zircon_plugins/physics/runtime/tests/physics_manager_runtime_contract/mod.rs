use std::sync::Arc;
use std::time::Duration;

use zircon_plugin_physics_runtime::{
    build_world_sync_state, integrate_builtin_physics_steps, module_descriptor,
    register_runtime_systems, DefaultPhysicsManager, PhysicsBodyCommand,
    DEFAULT_PHYSICS_MANAGER_NAME, PHYSICS_MODULE_NAME, PLUGIN_RUNTIME_MODULE_NAME,
};
use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBackendState, PhysicsColliderShape, PhysicsJointType, PhysicsManager,
        PhysicsQueryFilter, PhysicsQueryMode, PhysicsRayCastQuery, PhysicsSettings,
        PhysicsShapeCastQuery, PhysicsShapeOverlapQuery, PhysicsSimulationMode,
        PhysicsWorldStepPlan,
    },
    scene::physics::{
        PhysicsJointConstraintMetadata, PhysicsJointDrive, PhysicsSkeletonJointBinding,
    },
};
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::core::{CoreRuntime, TimePolicyTransaction};
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::plugin::RuntimeExtensionRegistry;
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, JointComponent, JointKind, NodeKind, RigidBodyComponent,
    RigidBodyType,
};
use zircon_runtime::scene::{
    create_default_level, install_world_runtime_extension_plan, LevelSystem,
};

const TEST_MAX_FIXED_STEPS: u32 = 4;
const TEST_FIXED_TIMESTEP_NANOS: u64 = 1_000_000_000 / 60;

fn configure_fixed_timestep(runtime: &CoreRuntime, timestep: Duration) {
    runtime
        .apply_time_policy(TimePolicyTransaction::new(
            runtime.time_policy().with_fixed_timestep(timestep),
        ))
        .expect("test physics fixed timestep should commit");
}

fn physics_manager(runtime: &CoreRuntime) -> Arc<dyn PhysicsManager> {
    let resolver = ManagerResolver::new(runtime.handle());
    resolver
        .resolve(resolver.physics_handle().expect("physics manager handle"))
        .expect("physics manager should resolve")
}

fn create_runtime_with_scene_and_physics() -> ScopedCoreRuntime {
    let fixture = ManagedConfigFixture::new("physics-contract");
    let runtime = CoreRuntime::new();
    configure_fixed_timestep(&runtime, test_fixed_timestep());
    let mut foundation = zircon_runtime::foundation::module_descriptor();
    zircon_runtime::foundation::bind_config_file_path(&mut foundation, fixture.config_path())
        .unwrap();
    runtime.register_module(foundation).unwrap();
    runtime
        .register_module(zircon_runtime::scene::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions.register_module(module_descriptor()).unwrap();
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(&mut extensions)
        .module(PLUGIN_RUNTIME_MODULE_NAME)
        .unwrap();
    register_runtime_systems(&mut module).unwrap();
    let plan = extensions.world_runtime_extension_plan().unwrap();
    install_world_runtime_extension_plan(&runtime.handle(), plan).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime.activate_module(PHYSICS_MODULE_NAME).unwrap();
    ScopedCoreRuntime {
        runtime,
        _fixture: fixture,
    }
}

fn tick_physics_level(runtime: &CoreRuntime, level: &LevelSystem) {
    let core = runtime.handle();
    let advance = runtime.advance_time_by(test_fixed_timestep(), TEST_MAX_FIXED_STEPS);
    level.tick(&core, advance).unwrap();
}

fn test_fixed_timestep() -> Duration {
    Duration::from_nanos(TEST_FIXED_TIMESTEP_NANOS)
}

#[test]
#[cfg(not(feature = "backend-jolt"))]
fn empty_jolt_feature_slot_reports_unavailable_not_ready() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "jolt".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let status = physics_manager(&runtime).backend_status();

    assert_eq!(status.requested_backend, "jolt");
    assert_eq!(status.active_backend, None);
    assert_eq!(status.state, PhysicsBackendState::Unavailable);
    assert_eq!(status.feature_gate.as_deref(), Some("backend-jolt"));
}

#[test]
#[cfg(feature = "backend-jolt")]
fn linked_jolt_backend_reports_ready() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "jolt".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let status = physics_manager(&runtime).backend_status();

    assert_eq!(status.requested_backend, "jolt");
    assert_eq!(status.active_backend.as_deref(), Some("jolt"));
    assert_eq!(status.state, PhysicsBackendState::Ready);
    assert_eq!(status.feature_gate.as_deref(), Some("backend-jolt"));
}

#[test]
fn unknown_backend_reports_unavailable_not_ready() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "experimental".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let status = physics_manager(&runtime).backend_status();

    assert_eq!(status.requested_backend, "experimental");
    assert_eq!(status.active_backend, None);
    assert_eq!(status.state, PhysicsBackendState::Unavailable);
}

#[test]
fn world_sync_preserves_constraint_and_skeletal_joint_metadata() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let (world_handle, skeleton, joint_entity) = level.with_world_mut(|world| {
        let skeleton = world.spawn_node(NodeKind::Mesh);
        let joint_entity = world.spawn_node(NodeKind::Cube);
        world
            .set_joint(
                joint_entity,
                Some(JointComponent {
                    joint_type: JointKind::Generic6Dof,
                    connected_entity: Some(skeleton),
                    anchor: Vec3::new(0.0, 1.0, 0.0),
                    axis: Vec3::Y,
                    limits: Some([-0.25, 0.25]),
                    collide_connected: true,
                    constraint: PhysicsJointConstraintMetadata {
                        linear_limits: [Some([-0.2, 0.2]), None, Some([0.0, 0.5])],
                        angular_limits: [Some([-0.5, 0.5]), Some([-0.25, 0.25]), None],
                        linear_drives: [
                            PhysicsJointDrive {
                                target_position: 0.1,
                                stiffness: 12.0,
                                damping: 2.0,
                                max_force: 30.0,
                                ..PhysicsJointDrive::default()
                            },
                            PhysicsJointDrive::default(),
                            PhysicsJointDrive::default(),
                        ],
                        break_force: Some(120.0),
                        break_torque: Some(40.0),
                        ..PhysicsJointConstraintMetadata::default()
                    },
                    skeleton_binding: Some(PhysicsSkeletonJointBinding {
                        skeleton_entity: skeleton,
                        bone_path: "Armature/Hips/Spine".to_string(),
                        parent_bone_path: Some("Armature/Hips".to_string()),
                    }),
                }),
            )
            .unwrap();
        (level.handle(), skeleton, joint_entity)
    });

    let sync = level.with_world(|world| build_world_sync_state(world_handle, world));
    let joint = sync
        .joints
        .iter()
        .find(|joint| joint.entity == joint_entity)
        .expect("joint metadata should be synced");

    assert_eq!(joint.kind, PhysicsJointType::Generic6Dof);
    assert_eq!(joint.connected_entity, Some(skeleton));
    assert!(joint.collide_connected);
    assert_eq!(joint.constraint.linear_limits[0], Some([-0.2, 0.2]));
    assert_eq!(joint.constraint.angular_limits[1], Some([-0.25, 0.25]));
    assert_eq!(joint.constraint.linear_drives[0].stiffness, 12.0);
    assert_eq!(joint.constraint.break_force, Some(120.0));
    assert_eq!(
        joint.skeleton_binding.as_ref().map(|binding| (
            binding.skeleton_entity,
            binding.bone_path.as_str(),
            binding.parent_bone_path.as_deref()
        )),
        Some((skeleton, "Armature/Hips/Spine", Some("Armature/Hips")))
    );
}

#[path = "contact.rs"]
mod contact;
#[path = "event.rs"]
mod event;
#[path = "query.rs"]
mod query;
#[path = "step.rs"]
mod step;

// Field order closes the runtime (and its final worker) before cleaning the unique fixture.
struct ScopedCoreRuntime {
    runtime: CoreRuntime,
    _fixture: ManagedConfigFixture,
}

impl std::ops::Deref for ScopedCoreRuntime {
    type Target = CoreRuntime;
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

struct ManagedConfigFixture {
    root: zircon_runtime::asset::project::ResolvedProjectPathIdentity,
}

impl ManagedConfigFixture {
    fn new(label: &str) -> Self {
        use zircon_runtime::asset::ProjectPaths;
        static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let approved = ProjectPaths::resolve_existing("E:/cargo-targets/zircon-local")
            .expect("the managed E drive fixture root must already exist");
        let display = ProjectPaths::display_path(approved.operation_path());
        assert!(
            display
                .to_str()
                .expect("managed root must be Unicode")
                .replace('\\', "/")
                .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
            "the approved root must resolve to the physical E drive root"
        );
        let approved = ProjectPaths::resolve_identity(approved.operation_path()).unwrap();
        let executable = std::env::current_exe().unwrap();
        let binary = ProjectPaths::resolve_identity(executable.parent().unwrap()).unwrap();
        assert!(
            binary.is_within(&approved),
            "test binary must physically live under the managed E root"
        );
        let id = NEXT_FIXTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(binary.operation_path().join(format!(
            "runtime55-{label}-{}-{nanos}-{id}",
            std::process::id()
        )))
        .unwrap();
        assert!(root.is_within(&binary));
        // Every authority check precedes mkdir/write. create_dir never adopts an existing root.
        std::fs::create_dir(root.operation_path()).unwrap();
        let fixture = Self { root };
        assert_eq!(
            ProjectPaths::resolve_identity(fixture.root.operation_path()).unwrap(),
            fixture.root
        );
        fixture
    }

    fn config_path(&self) -> std::path::PathBuf {
        self.root.operation_path().join("config.json")
    }
}

impl Drop for ManagedConfigFixture {
    fn drop(&mut self) {
        // The declaring scope/wrapper closes all runtimes, handles and worker owners first.
        let current =
            zircon_runtime::asset::ProjectPaths::resolve_identity(self.root.operation_path())
                .unwrap();
        assert_eq!(
            current, self.root,
            "cleanup must retain this fixture's physical identity"
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}
