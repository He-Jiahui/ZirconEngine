//! Registered world-point exports may author local T, not signed local R/S.
use super::*;
use crate::core::math::Mat4;
use crate::scene::components::{LocalTransform, WorldMatrix};
use crate::scene::{EntityId, Mobility};
use crate::script::VmError;

const SIGNED: Vec3 = Vec3::new(-2.0, -3.0, 0.5);
const ROOT_ORIGIN: Vec3 = Vec3::new(4.0, -3.0, 1.0);
const WORLD_DELTA: Vec3 = Vec3::new(2.0, 1.0, 4.0);

#[derive(Debug, PartialEq)]
struct Published {
    authored: Transform,
    matrix: Mat4,
    revision: u64,
}

fn level_with_world(world: World) -> LevelSystem {
    LevelSystem::new(
        WorldHandle::new(512),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    )
}

fn spawn(world: &mut World, translation: Vec3, rotation: Quat, scale: Vec3) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .update_transform(
            entity,
            Transform {
                translation,
                rotation,
                scale,
            },
        )
        .unwrap();
    entity
}

fn signed_root() -> (LevelSystem, EntityId) {
    let mut world = World::empty();
    let entity = spawn(&mut world, ROOT_ORIGIN, Quat::IDENTITY, SIGNED);
    (level_with_world(world), entity)
}

fn published(level: &LevelSystem, entity: EntityId) -> Published {
    level.with_world_mut(|world| {
        world.flush_pending_scene_systems();
        Published {
            authored: world.get::<LocalTransform>(entity).unwrap().transform,
            // Do not use world_matrix(), which can project a dirty chain on read.
            matrix: world.get::<WorldMatrix>(entity).unwrap().0,
            revision: world.world_generation(),
        }
    })
}

fn call(
    level: &LevelSystem,
    entity: EntityId,
    name: &str,
    extra: impl IntoIterator<Item = ScriptHostValue>,
) -> Result<ScriptHostValue, VmError> {
    let core = CoreRuntime::new();
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");
    let args = std::iter::once(ScriptHostValue::Int(entity as i64))
        .chain(extra)
        .collect();
    with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || exports.call_with_capabilities(GAMEPLAY_MODULE, name, args, &capabilities),
    )
}

fn xyz(point: Vec3) -> [ScriptHostValue; 3] {
    point
        .to_array()
        .map(|v| ScriptHostValue::Float(f64::from(v)))
}

fn assert_basis(actual: Mat4, expected: [Vec3; 3]) {
    assert_vec3_close(actual.x_axis.truncate(), expected[0]);
    assert_vec3_close(actual.y_axis.truncate(), expected[1]);
    assert_vec3_close(actual.z_axis.truncate(), expected[2]);
    assert_eq!(actual.row(3).to_array(), [0.0, 0.0, 0.0, 1.0]);
}

fn basis_of(matrix: Mat4) -> [Vec3; 3] {
    [
        matrix.x_axis.truncate(),
        matrix.y_axis.truncate(),
        matrix.z_axis.truncate(),
    ]
}

fn assert_authored_rs(actual: Transform, expected: Transform) {
    assert_eq!(
        actual.scale.to_array().map(f32::to_bits),
        expected.scale.to_array().map(f32::to_bits),
        "world point writes must retain authored signed local scale"
    );
    assert_eq!(
        actual.rotation.to_array().map(f32::to_bits),
        expected.rotation.to_array().map(f32::to_bits),
        "world point writes must retain the authored local quaternion"
    );
}

fn observe(label: &str, before: &Published, after: &Published, changed: &ScriptHostValue) {
    let columns = basis_of(before.matrix);
    let basis_delta = basis_of(after.matrix)
        .into_iter()
        .zip(columns)
        .map(|(old, new)| (old - new).abs().max_element())
        .fold(0.0_f32, f32::max);
    eprintln!(
        "SCRIPT_POINT label={label} changed={changed:?} rev_before={} rev_after={} local={:?} q={:?} scale={:?} origin={:?} basis_delta={basis_delta}",
        before.revision,
        after.revision,
        after.authored.translation.to_array(),
        after.authored.rotation.to_array(),
        after.authored.scale.to_array(),
        after.matrix.w_axis.truncate().to_array(),
    );
}

#[test]
fn signed_root_same_point_is_not_an_authored_write() {
    let (level, entity) = signed_root();
    let before = published(&level, entity);
    let root_basis = [-2.0 * Vec3::X, -3.0 * Vec3::Y, 0.5 * Vec3::Z];
    assert_eq!(before.authored.scale, SIGNED);
    assert_eq!(before.authored.rotation, Quat::IDENTITY);
    assert_basis(before.matrix, root_basis);
    let changed = call(&level, entity, "set_position", xyz(ROOT_ORIGIN)).unwrap();
    let after = published(&level, entity);
    observe("root_same", &before, &after, &changed);
    assert_eq!(after.matrix, before.matrix);
    assert_eq!(
        changed,
        ScriptHostValue::Bool(false),
        "same point is a no-op"
    );
    assert_eq!(after.revision, before.revision);
    assert_authored_rs(after.authored, before.authored);
}

#[test]
fn signed_root_changed_point_keeps_authored_basis() {
    let (level, entity) = signed_root();
    let before = published(&level, entity);
    let changed = call(
        &level,
        entity,
        "set_position_json",
        [ScriptHostValue::String("[-5,7,2]".into())],
    )
    .unwrap();
    let after = published(&level, entity);
    observe("root_move", &before, &after, &changed);
    let point = Vec3::new(-5.0, 7.0, 2.0);
    assert_eq!(changed, ScriptHostValue::Bool(true));
    assert!(after.revision > before.revision);
    assert_eq!(after.authored.translation, point);
    assert_vec3_close(after.matrix.w_axis.truncate(), point);
    assert_basis(
        after.matrix,
        [-2.0 * Vec3::X, -3.0 * Vec3::Y, 0.5 * Vec3::Z],
    );
    assert_authored_rs(after.authored, before.authored);
}

fn translate_in_sheared_parent(sign: f32, label: &str, local_delta: Vec3) {
    let mut world = World::empty();
    let grand = spawn(
        &mut world,
        Vec3::new(8.0, -6.0, 2.0),
        Quat::IDENTITY,
        Vec3::new(2.0 * sign, 1.0, 4.0),
    );
    let mid = spawn(
        &mut world,
        Vec3::new(1.0, 2.0, -1.0),
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        Vec3::ONE,
    );
    let child_local = Vec3::new(2.0, 1.0, 3.0);
    let child = spawn(&mut world, child_local, Quat::IDENTITY, SIGNED);
    assert!(world.set_parent_checked(mid, Some(grand)).unwrap());
    assert!(world.set_parent_checked(child, Some(mid)).unwrap());
    let level = level_with_world(world);
    let parent = published(&level, mid).matrix;
    let parent_gram = parent.x_axis.truncate().dot(parent.y_axis.truncate());
    assert!(
        (parent_gram + 1.5).abs() < 1.0e-5,
        "parent itself is sheared"
    );
    assert_eq!(parent.determinant().signum(), sign);

    // Independent column and point oracle from D=diag(±2,1,4), M=Rz(45°).
    let c = std::f32::consts::FRAC_1_SQRT_2;
    let expected_origin = Vec3::new(8.0 + 2.0 * sign * (1.0 + c), -4.0 + 3.0 * c, 10.0);
    let expected_basis = [
        Vec3::new(-4.0 * sign * c, -2.0 * c, 0.0),
        Vec3::new(6.0 * sign * c, -3.0 * c, 0.0),
        2.0 * Vec3::Z,
    ];
    let before = published(&level, child);
    assert_vec3_close(before.matrix.w_axis.truncate(), expected_origin);
    assert_basis(before.matrix, expected_basis);
    assert_eq!(before.authored.scale, SIGNED);
    let changed = call(&level, child, "translate", xyz(WORLD_DELTA)).unwrap();
    let after = published(&level, child);
    eprintln!("SCRIPT_POINT_PARENT label={label} gram_xy={parent_gram} det_sign={sign}");
    observe(label, &before, &after, &changed);
    assert_eq!(changed, ScriptHostValue::Bool(true));
    assert!(after.revision > before.revision);
    assert_vec3_close(after.authored.translation, child_local + local_delta);
    assert_vec3_close(
        after.matrix.w_axis.truncate(),
        expected_origin + WORLD_DELTA,
    );
    assert_basis(after.matrix, expected_basis);
    assert_authored_rs(after.authored, before.authored);
    assert_eq!(published(&level, mid).matrix, parent);
}

#[test]
fn child_point_keeps_basis_below_sheared_parent() {
    translate_in_sheared_parent(
        1.0,
        "shear_pos",
        Vec3::new(std::f32::consts::SQRT_2, 0.0, 1.0),
    );
}

#[test]
fn child_point_keeps_basis_below_reflected_sheared_parent() {
    translate_in_sheared_parent(
        -1.0,
        "shear_neg",
        Vec3::new(0.0, std::f32::consts::SQRT_2, 1.0),
    );
}

#[test]
fn ordinary_root_position_changes_once() {
    let mut world = World::empty();
    let entity = spawn(&mut world, ROOT_ORIGIN, Quat::IDENTITY, Vec3::ONE);
    let level = level_with_world(world);
    let set = || call(&level, entity, "set_position", xyz(WORLD_DELTA)).unwrap();
    let before = published(&level, entity);
    assert_eq!(set(), ScriptHostValue::Bool(true));
    let moved = published(&level, entity);
    assert!(moved.revision > before.revision);
    assert_eq!(moved.matrix.w_axis.truncate(), WORLD_DELTA);
    assert_authored_rs(moved.authored, before.authored);
    assert_eq!(set(), ScriptHostValue::Bool(false));
    assert_eq!(published(&level, entity), moved);
}

fn assert_refusal(level: &LevelSystem, entity: EntityId, point: [ScriptHostValue; 3], why: &str) {
    let before = published(level, entity);
    let error = call(level, entity, "set_position", point).unwrap_err();
    let VmError::Operation(message) = error else {
        panic!("expected a registered host operation error: {error:?}");
    };
    assert_eq!(
        message,
        format!("host export call failed: {GAMEPLAY_MODULE}.set_position: {why}")
    );
    assert_eq!(published(level, entity), before);
    eprintln!("SCRIPT_POINT_REFUSAL {message}");
}

#[test]
fn same_point_does_not_bypass_static_authority() {
    let (level, entity) = signed_root();
    level.with_world_mut(|world| assert!(world.set_mobility(entity, Mobility::Static).unwrap()));
    assert_refusal(
        &level,
        entity,
        xyz(ROOT_ORIGIN),
        &format!("static entity {entity} cannot update transform at runtime"),
    );
}

#[test]
fn singular_derived_parent_rejects_without_writing() {
    let mut world = World::empty();
    let thin = Vec3::new(1.0, 1.0, 1.0e-25);
    let grand = spawn(&mut world, Vec3::ZERO, Quat::IDENTITY, thin);
    let mid = spawn(&mut world, Vec3::ZERO, Quat::IDENTITY, thin);
    let child = spawn(&mut world, ROOT_ORIGIN, Quat::IDENTITY, SIGNED);
    assert!(world.set_parent_checked(mid, Some(grand)).unwrap());
    assert!(world.set_parent_checked(child, Some(mid)).unwrap());
    let level = level_with_world(world);
    let parent = published(&level, mid);
    assert_eq!(parent.authored.scale, thin);
    assert_eq!(parent.matrix.z_axis.truncate(), Vec3::ZERO);
    assert_eq!(parent.matrix.determinant(), 0.0);
    assert_refusal(
        &level,
        child,
        xyz(WORLD_DELTA),
        &format!("cannot convert world transform for entity {child}: parent {mid} has a non-invertible world transform"),
    );
}

#[test]
fn nonfinite_point_keeps_existing_boundary_diagnostics() {
    let outside_f32 = || {
        [
            ScriptHostValue::Float(f64::MAX),
            ScriptHostValue::Float(0.0),
            ScriptHostValue::Float(0.0),
        ]
    };
    let (level, entity) = signed_root();
    assert_refusal(
        &level,
        entity,
        outside_f32(),
        &format!("property `entities[{entity}].transform.translation` expected finite translation"),
    );

    let mut world = World::empty();
    let parent = spawn(&mut world, Vec3::ZERO, Quat::IDENTITY, Vec3::splat(2.0));
    let child = spawn(&mut world, ROOT_ORIGIN, Quat::IDENTITY, SIGNED);
    assert!(world.set_parent_checked(child, Some(parent)).unwrap());
    let level = level_with_world(world);
    assert_refusal(
        &level,
        child,
        outside_f32(),
        &format!("cannot represent world transform for entity {child} as local TRS under parent {parent}"),
    );
}
