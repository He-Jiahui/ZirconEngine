use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsWorldSyncState, SimulatedPoseFeed, SkeletalPoseTarget,
    SkeletalPoseTargets,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::components::RigidBodyType;
use zircon_runtime::scene::ecs::Resource;
use zircon_runtime::scene::world::World;
use zircon_runtime::scene::EntityId;

use super::profile::RagdollProfile;

const MIN_RAGDOLL_SCALE_COMPONENT: Real = 1.0e-6;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RagdollMode {
    #[default]
    Animated,
    Simulated,
    Blended {
        weight: Real,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RagdollState {
    mode: RagdollMode,
    transition_from: Option<RagdollMode>,
    bone_weights: BTreeMap<String, Real>,
    body_offsets: BTreeMap<String, Transform>,
    last_animated_body_world: BTreeMap<String, Transform>,
    release_linear_velocity: BTreeMap<String, Vec3>,
    release_angular_velocity: BTreeMap<String, Vec3>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RagdollRuntime {
    states: Arc<BTreeMap<EntityId, RagdollState>>,
}

impl RagdollRuntime {
    pub fn configure(&mut self, skeleton: EntityId, mode: RagdollMode) {
        let state = Arc::make_mut(&mut self.states).entry(skeleton).or_default();
        state.mode = sanitize_mode(mode);
        state.transition_from = None;
    }

    pub fn configure_profile(
        &mut self,
        skeleton: EntityId,
        mode: RagdollMode,
        profile: &RagdollProfile,
    ) {
        self.configure(skeleton, mode);
        for bone in &profile.bones {
            self.set_bone_weight(skeleton, bone.bone_path.clone(), bone.blend_weight);
            Arc::make_mut(&mut self.states)
                .entry(skeleton)
                .or_default()
                .body_offsets
                .insert(bone.bone_path.clone(), bone.body_offset);
        }
    }

    pub fn set_mode(&mut self, skeleton: EntityId, mode: RagdollMode) {
        let next_mode = sanitize_mode(mode);
        let state = Arc::make_mut(&mut self.states).entry(skeleton).or_default();
        if state.mode != next_mode {
            state.transition_from = Some(state.mode);
            state.mode = next_mode;
        }
    }

    pub fn mode(&self, skeleton: EntityId) -> Option<RagdollMode> {
        self.states.get(&skeleton).map(|state| state.mode)
    }

    pub fn set_bone_weight(
        &mut self,
        skeleton: EntityId,
        bone_path: impl Into<String>,
        weight: Real,
    ) {
        Arc::make_mut(&mut self.states)
            .entry(skeleton)
            .or_default()
            .bone_weights
            .insert(bone_path.into(), sanitize_weight(weight));
    }

    pub fn remove(&mut self, skeleton: EntityId) {
        Arc::make_mut(&mut self.states).remove(&skeleton);
    }

    fn state(&self, skeleton: EntityId) -> Option<&RagdollState> {
        self.states.get(&skeleton)
    }
}

impl Resource for RagdollRuntime {}

pub(crate) fn drive_ragdoll_bodies_from_animation(
    world: &mut World,
    ragdolls: &mut RagdollRuntime,
    delta_seconds: Real,
) {
    let Some(targets) = world.get_resource::<SkeletalPoseTargets>().cloned() else {
        return;
    };
    let bindings = topologically_ordered_bindings(collect_bound_bodies(world));
    let mut driven_bone_world = BTreeMap::<(EntityId, String), Transform>::new();
    let mut visited_skeletons = std::collections::BTreeSet::new();

    for binding in bindings {
        let Some(state) = Arc::make_mut(&mut ragdolls.states).get_mut(&binding.skeleton) else {
            continue;
        };
        visited_skeletons.insert(binding.skeleton);
        let Some(mut body) = world.rigid_body(binding.body).cloned() else {
            continue;
        };

        match state.mode {
            RagdollMode::Animated => {
                body.body_type = RigidBodyType::Kinematic;
                body.linear_velocity = Vec3::ZERO;
                body.angular_velocity = Vec3::ZERO;
            }
            RagdollMode::Simulated | RagdollMode::Blended { .. } => {
                body.body_type = RigidBodyType::Dynamic;
                if state.transition_from == Some(RagdollMode::Animated) {
                    body.linear_velocity = state
                        .release_linear_velocity
                        .get(&binding.bone_path)
                        .copied()
                        .unwrap_or(Vec3::ZERO);
                    body.angular_velocity = state
                        .release_angular_velocity
                        .get(&binding.bone_path)
                        .copied()
                        .unwrap_or(Vec3::ZERO);
                }
            }
        }

        if state.mode == RagdollMode::Animated {
            if let Some(target) =
                resolve_unique_target(&targets, binding.skeleton, &binding.bone_path)
            {
                let parent_world = binding
                    .parent_bone_path
                    .as_deref()
                    .and_then(|path| {
                        driven_bone_world
                            .get(&(binding.skeleton, path.to_string()))
                            .copied()
                    })
                    .or_else(|| world.world_transform(binding.skeleton));
                if let Some(parent_world) = parent_world {
                    let bone_world = combine_transforms(parent_world, target.local_transform);
                    let body_world = state
                        .body_offsets
                        .get(&binding.bone_path)
                        .copied()
                        .map(|offset| combine_transforms(bone_world, offset))
                        .unwrap_or(bone_world);
                    if transform_is_finite(body_world) {
                        let (linear_velocity, angular_velocity) = state
                            .last_animated_body_world
                            .get(&binding.bone_path)
                            .copied()
                            .map(|previous| {
                                animated_release_velocity(previous, body_world, delta_seconds)
                            })
                            .unwrap_or((Vec3::ZERO, Vec3::ZERO));
                        state
                            .release_linear_velocity
                            .insert(binding.bone_path.clone(), linear_velocity);
                        state
                            .release_angular_velocity
                            .insert(binding.bone_path.clone(), angular_velocity);
                        state
                            .last_animated_body_world
                            .insert(binding.bone_path.clone(), body_world);
                        let _ = world.update_transform(binding.body, body_world);
                        driven_bone_world
                            .insert((binding.skeleton, binding.bone_path.clone()), bone_world);
                    }
                }
            }
        }
        let _ = world.set_rigid_body(binding.body, Some(body));
    }

    let states = Arc::make_mut(&mut ragdolls.states);
    for skeleton in visited_skeletons {
        if let Some(state) = states.get_mut(&skeleton) {
            state.transition_from = None;
        }
    }
}

pub(crate) fn write_simulated_pose_feed(
    world: &World,
    sync: &PhysicsWorldSyncState,
    ragdolls: &RagdollRuntime,
    interpolation_alpha: Real,
    feed: &mut SimulatedPoseFeed,
) {
    feed.clear();
    let interpolation_alpha = sanitize_weight(interpolation_alpha);
    let bodies_by_entity = index_synced_bodies(sync);
    let bone_world_by_path = collect_synced_bone_world_by_path(sync, ragdolls, &bodies_by_entity);
    let mut rows_by_skeleton =
        BTreeMap::<EntityId, BTreeMap<String, Option<SkeletalPoseTarget>>>::new();

    for joint in &sync.joints {
        let Some(binding) = joint.skeleton_binding.as_ref() else {
            continue;
        };
        let Some(state) = ragdolls.state(binding.skeleton_entity) else {
            continue;
        };
        let Some(body) =
            resolve_joint_body(&bodies_by_entity, joint.entity, joint.connected_entity)
        else {
            continue;
        };
        let Some(skeleton_world) = world.world_transform(binding.skeleton_entity) else {
            continue;
        };
        let parent_world = binding
            .parent_bone_path
            .as_ref()
            .and_then(|path| {
                bone_world_by_path
                    .get(&(binding.skeleton_entity, path.clone()))
                    .copied()
            })
            .unwrap_or(skeleton_world);
        let normalized_weight = mode_weight(state.mode)
            * state
                .bone_weights
                .get(&binding.bone_path)
                .copied()
                .map(sanitize_weight)
                .unwrap_or(1.0)
            * interpolation_alpha;
        if normalized_weight <= 0.0 {
            continue;
        }

        let bone_world = state
            .body_offsets
            .get(&binding.bone_path)
            .copied()
            .map(|offset| combine_transforms(body.transform, inverse_transform(offset)))
            .unwrap_or(body.transform);
        let local_transform = relative_transform(parent_world, bone_world);
        if !transform_is_finite(local_transform) {
            continue;
        }
        let bone_name = bone_leaf(&binding.bone_path).to_string();
        let rows = rows_by_skeleton.entry(binding.skeleton_entity).or_default();
        match rows.entry(bone_name.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Some(SkeletalPoseTarget {
                    bone_name,
                    local_transform,
                    normalized_weight,
                }));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                entry.insert(None);
            }
        }
    }

    for (skeleton, rows) in rows_by_skeleton {
        let rows = rows.into_values().flatten().collect::<Vec<_>>();
        if !rows.is_empty() {
            feed.replace(skeleton, Arc::from(rows));
        }
    }
}

#[derive(Clone, Debug)]
struct BoundBody {
    body: EntityId,
    skeleton: EntityId,
    bone_path: String,
    parent_bone_path: Option<String>,
}

fn collect_bound_bodies(world: &World) -> Vec<BoundBody> {
    world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let binding = node.joint.as_ref()?.skeleton_binding.as_ref()?;
            node.rigid_body.as_ref()?;
            Some(BoundBody {
                body: node.id,
                skeleton: binding.skeleton_entity,
                bone_path: binding.bone_path.clone(),
                parent_bone_path: binding.parent_bone_path.clone(),
            })
        })
        .collect()
}

fn topologically_ordered_bindings(mut bindings: Vec<BoundBody>) -> Vec<BoundBody> {
    let all_paths = bindings
        .iter()
        .map(|binding| (binding.skeleton, binding.bone_path.clone()))
        .collect::<std::collections::BTreeSet<_>>();
    bindings.sort_by(|left, right| {
        bone_depth(&left.bone_path)
            .cmp(&bone_depth(&right.bone_path))
            .then_with(|| left.bone_path.cmp(&right.bone_path))
    });
    let mut emitted = std::collections::BTreeSet::new();
    let mut ordered = Vec::with_capacity(bindings.len());
    while !bindings.is_empty() {
        let Some(index) = bindings.iter().position(|binding| {
            binding.parent_bone_path.as_ref().is_none_or(|parent| {
                !all_paths.contains(&(binding.skeleton, parent.clone()))
                    || emitted.contains(&(binding.skeleton, parent.clone()))
            })
        }) else {
            break;
        };
        let binding = bindings.remove(index);
        emitted.insert((binding.skeleton, binding.bone_path.clone()));
        ordered.push(binding);
    }
    ordered
}

fn resolve_unique_target<'a>(
    targets: &'a SkeletalPoseTargets,
    skeleton: EntityId,
    bone_path: &str,
) -> Option<&'a SkeletalPoseTarget> {
    let rows = targets.targets(skeleton)?;
    let expected = bone_leaf(bone_path);
    let mut matches = rows
        .iter()
        .filter(|target| target.bone_name == bone_path || target.bone_name == expected);
    let target = matches.next()?;
    matches.next().is_none().then_some(target)
}

fn resolve_joint_body<'a>(
    bodies_by_entity: &HashMap<EntityId, Option<&'a PhysicsBodySyncState>>,
    joint_entity: EntityId,
    connected_entity: Option<EntityId>,
) -> Option<&'a PhysicsBodySyncState> {
    bodies_by_entity
        .get(&joint_entity)
        .copied()
        .flatten()
        .or_else(|| {
            connected_entity.and_then(|entity| bodies_by_entity.get(&entity).copied().flatten())
        })
}

fn index_synced_bodies<'a>(
    sync: &'a PhysicsWorldSyncState,
) -> HashMap<EntityId, Option<&'a PhysicsBodySyncState>> {
    let requested_body_count = sync
        .joints
        .iter()
        .filter(|joint| joint.skeleton_binding.is_some())
        .map(|joint| 1 + usize::from(joint.connected_entity.is_some()))
        .sum();
    if requested_body_count == 0 {
        return HashMap::new();
    }

    let mut bodies_by_entity = HashMap::with_capacity(requested_body_count);
    for joint in &sync.joints {
        if joint.skeleton_binding.is_none() {
            continue;
        }
        bodies_by_entity.entry(joint.entity).or_insert(None);
        if let Some(entity) = joint.connected_entity {
            bodies_by_entity.entry(entity).or_insert(None);
        }
    }
    for body in &sync.bodies {
        if let Some(slot) = bodies_by_entity.get_mut(&body.entity) {
            slot.get_or_insert(body);
        }
    }
    bodies_by_entity
}

fn collect_synced_bone_world_by_path(
    sync: &PhysicsWorldSyncState,
    ragdolls: &RagdollRuntime,
    bodies_by_entity: &HashMap<EntityId, Option<&PhysicsBodySyncState>>,
) -> BTreeMap<(EntityId, String), Transform> {
    sync.joints
        .iter()
        .filter_map(|joint| {
            let binding = joint.skeleton_binding.as_ref()?;
            let body = resolve_joint_body(bodies_by_entity, joint.entity, joint.connected_entity)?;
            let state = ragdolls.state(binding.skeleton_entity)?;
            let bone_world = state
                .body_offsets
                .get(&binding.bone_path)
                .copied()
                .map(|offset| combine_transforms(body.transform, inverse_transform(offset)))
                .unwrap_or(body.transform);
            Some((
                (binding.skeleton_entity, binding.bone_path.clone()),
                bone_world,
            ))
        })
        .collect()
}

fn sanitize_mode(mode: RagdollMode) -> RagdollMode {
    match mode {
        RagdollMode::Blended { weight } => RagdollMode::Blended {
            weight: sanitize_weight(weight),
        },
        mode => mode,
    }
}

fn sanitize_weight(weight: Real) -> Real {
    if weight.is_finite() {
        weight.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn mode_weight(mode: RagdollMode) -> Real {
    match sanitize_mode(mode) {
        RagdollMode::Animated => 0.0,
        RagdollMode::Simulated => 1.0,
        RagdollMode::Blended { weight } => weight,
    }
}

fn bone_leaf(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn bone_depth(path: &str) -> usize {
    path.bytes().filter(|byte| *byte == b'/').count()
}

fn combine_transforms(parent: Transform, local: Transform) -> Transform {
    Transform {
        translation: parent.translation + parent.rotation * (parent.scale * local.translation),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale * local.scale,
    }
}

fn relative_transform(parent: Transform, child: Transform) -> Transform {
    let inverse_rotation = parent.rotation.inverse();
    Transform {
        translation: divide_by_scale(
            inverse_rotation * (child.translation - parent.translation),
            parent.scale,
        ),
        rotation: inverse_rotation * child.rotation,
        scale: divide_by_scale(child.scale, parent.scale),
    }
}

fn inverse_transform(transform: Transform) -> Transform {
    let inverse_rotation = transform.rotation.inverse();
    let inverse_scale = divide_by_scale(Vec3::ONE, transform.scale);
    Transform {
        translation: divide_by_scale(inverse_rotation * -transform.translation, transform.scale),
        rotation: inverse_rotation,
        scale: inverse_scale,
    }
}

fn divide_by_scale(value: Vec3, scale: Vec3) -> Vec3 {
    Vec3::new(
        divide_by_component(value.x, scale.x),
        divide_by_component(value.y, scale.y),
        divide_by_component(value.z, scale.z),
    )
}

fn divide_by_component(value: Real, scale: Real) -> Real {
    if scale.abs() > MIN_RAGDOLL_SCALE_COMPONENT {
        value / scale
    } else {
        0.0
    }
}

fn transform_is_finite(transform: Transform) -> bool {
    transform.translation.is_finite()
        && transform.rotation.is_finite()
        && transform.scale.is_finite()
}

fn animated_release_velocity(
    previous: Transform,
    current: Transform,
    delta_seconds: Real,
) -> (Vec3, Vec3) {
    if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
        return (Vec3::ZERO, Vec3::ZERO);
    }
    let inverse_delta = delta_seconds.recip();
    let linear = (current.translation - previous.translation) * inverse_delta;
    let angular = (current.rotation * previous.rotation.inverse()).to_scaled_axis() * inverse_delta;
    (
        finite_velocity_or_zero(linear),
        finite_velocity_or_zero(angular),
    )
}

fn finite_velocity_or_zero(velocity: Vec3) -> Vec3 {
    if velocity.is_finite() {
        velocity
    } else {
        Vec3::ZERO
    }
}
