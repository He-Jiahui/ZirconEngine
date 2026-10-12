use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, SkeletalPoseTarget, SkeletalPoseTargets,
};
use zircon_runtime::core::framework::scene::physics::{
    PhysicsJointConstraintMetadata, PhysicsSkeletonJointBinding,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, JointComponent, JointKind, NodeKind, RigidBodyComponent,
};
use zircon_runtime::scene::world::{DetachedEntityBatchRemoveError, SceneError, World};
use zircon_runtime::scene::{EntityId, LifecycleTargetChange};

use super::runtime::{RagdollMode, RagdollRuntime};

const DEFAULT_RAGDOLL_BODY_MASS: Real = 1.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollProfile {
    pub id: String,
    #[serde(default)]
    pub bones: Vec<RagdollBoneProfile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollBoneProfile {
    pub bone_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_bone_path: Option<String>,
    pub shape: PhysicsColliderShape,
    #[serde(default = "default_body_mass")]
    pub mass: Real,
    #[serde(default)]
    pub body_offset: Transform,
    #[serde(default)]
    pub constraint: PhysicsJointConstraintMetadata,
    #[serde(default = "default_bone_weight")]
    pub blend_weight: Real,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RagdollProfileError {
    Parse(String),
    EmptyId,
    EmptyProfile,
    EmptyBonePath {
        index: usize,
    },
    DuplicateBonePath {
        bone_path: String,
    },
    MissingParent {
        bone_path: String,
        parent_bone_path: String,
    },
    ParentCycle {
        bone_path: String,
    },
    InvalidMass {
        bone_path: String,
    },
    InvalidBlendWeight {
        bone_path: String,
    },
    InvalidShape {
        bone_path: String,
    },
    InvalidTransform {
        bone_path: String,
    },
    MissingSkeletonTargets {
        skeleton: EntityId,
    },
    MissingBoneTarget {
        bone_path: String,
    },
}

/// A spawn failure keeps the primary scene error and any cancelled cleanup payload.
/// Profile validation remains available separately as a cloneable error.
pub enum RagdollSpawnError {
    Profile(RagdollProfileError),
    SceneMutation {
        source: SceneError,
        /// The exact cancelled removal owns any rows already detached by cleanup.
        /// Callers may retain it or consume its `into_parts` to dispose of the batch.
        cleanup_cancellation: Option<DetachedEntityBatchRemoveError>,
    },
}

impl RagdollSpawnError {
    pub fn lifecycle_target_change(&self) -> Option<LifecycleTargetChange> {
        match self {
            Self::Profile(_) => None,
            Self::SceneMutation {
                source,
                cleanup_cancellation,
            } => source.lifecycle_target_change().or_else(|| {
                cleanup_cancellation
                    .as_ref()
                    .and_then(|cleanup| cleanup.error().lifecycle_target_change())
            }),
        }
    }
}

impl fmt::Debug for RagdollSpawnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => formatter.debug_tuple("Profile").field(error).finish(),
            Self::SceneMutation {
                source,
                cleanup_cancellation,
            } => formatter
                .debug_struct("SceneMutation")
                .field("source", source)
                .field(
                    "cleanup_cancellation",
                    &cleanup_cancellation.as_ref().map(|cleanup| cleanup.error()),
                )
                .finish(),
        }
    }
}

impl fmt::Display for RagdollSpawnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => fmt::Display::fmt(error, formatter),
            Self::SceneMutation {
                source,
                cleanup_cancellation,
            } => {
                write!(formatter, "ragdoll spawn failed: {source}")?;
                if let Some(cleanup) = cleanup_cancellation {
                    write!(formatter, "; cleanup cancelled: {}", cleanup.error())?;
                }
                Ok(())
            }
        }
    }
}

impl Error for RagdollSpawnError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::SceneMutation { source, .. } => Some(source),
        }
    }
}

impl From<RagdollProfileError> for RagdollSpawnError {
    fn from(error: RagdollProfileError) -> Self {
        Self::Profile(error)
    }
}

impl From<SceneError> for RagdollSpawnError {
    fn from(source: SceneError) -> Self {
        Self::SceneMutation {
            source,
            cleanup_cancellation: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RagdollSpawn {
    pub skeleton: EntityId,
    pub bodies_by_bone: BTreeMap<String, EntityId>,
}

impl RagdollProfile {
    pub fn from_toml(source: &str) -> Result<Self, RagdollProfileError> {
        let profile = toml::from_str::<Self>(source)
            .map_err(|error| RagdollProfileError::Parse(error.to_string()))?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn validate(&self) -> Result<(), RagdollProfileError> {
        if self.id.trim().is_empty() {
            return Err(RagdollProfileError::EmptyId);
        }
        if self.bones.is_empty() {
            return Err(RagdollProfileError::EmptyProfile);
        }
        let mut bone_paths = BTreeSet::new();
        for (index, bone) in self.bones.iter().enumerate() {
            if bone.bone_path.trim().is_empty() {
                return Err(RagdollProfileError::EmptyBonePath { index });
            }
            if !bone_paths.insert(bone.bone_path.as_str()) {
                return Err(RagdollProfileError::DuplicateBonePath {
                    bone_path: bone.bone_path.clone(),
                });
            }
            if !bone.mass.is_finite() || bone.mass <= 0.0 {
                return Err(RagdollProfileError::InvalidMass {
                    bone_path: bone.bone_path.clone(),
                });
            }
            if !bone.blend_weight.is_finite() || !(0.0..=1.0).contains(&bone.blend_weight) {
                return Err(RagdollProfileError::InvalidBlendWeight {
                    bone_path: bone.bone_path.clone(),
                });
            }
            if !shape_is_valid(&bone.shape) {
                return Err(RagdollProfileError::InvalidShape {
                    bone_path: bone.bone_path.clone(),
                });
            }
            if !transform_is_finite(bone.body_offset) {
                return Err(RagdollProfileError::InvalidTransform {
                    bone_path: bone.bone_path.clone(),
                });
            }
        }
        for bone in &self.bones {
            if let Some(parent) = bone.parent_bone_path.as_ref() {
                if parent == &bone.bone_path || !bone_paths.contains(parent.as_str()) {
                    return Err(RagdollProfileError::MissingParent {
                        bone_path: bone.bone_path.clone(),
                        parent_bone_path: parent.clone(),
                    });
                }
            }
        }
        let parent_by_bone = self
            .bones
            .iter()
            .map(|bone| (bone.bone_path.as_str(), bone.parent_bone_path.as_deref()))
            .collect::<BTreeMap<_, _>>();
        for bone in &self.bones {
            let mut visited = BTreeSet::new();
            let mut cursor = Some(bone.bone_path.as_str());
            while let Some(path) = cursor {
                if !visited.insert(path) {
                    return Err(RagdollProfileError::ParentCycle {
                        bone_path: bone.bone_path.clone(),
                    });
                }
                cursor = parent_by_bone.get(path).copied().flatten();
            }
        }
        Ok(())
    }

    pub fn spawn(
        &self,
        world: &mut World,
        skeleton: EntityId,
    ) -> Result<RagdollSpawn, RagdollSpawnError> {
        self.validate()?;
        let targets = world
            .get_resource::<SkeletalPoseTargets>()
            .cloned()
            .ok_or(RagdollProfileError::MissingSkeletonTargets { skeleton })?;
        let rows = targets
            .targets(skeleton)
            .ok_or(RagdollProfileError::MissingSkeletonTargets { skeleton })?;
        let skeleton_world = world
            .world_transform(skeleton)
            .ok_or(RagdollProfileError::MissingSkeletonTargets { skeleton })?;
        let target_lookup = SkeletalTargetLookup::new(rows);
        let bones = topologically_ordered_bones(&self.bones);
        let mut bone_world_by_path = BTreeMap::<String, Transform>::new();
        let mut prepared = Vec::with_capacity(bones.len());
        for bone in bones {
            let local_bone = target_lookup
                .resolve(&bone.bone_path)
                .ok_or_else(|| RagdollProfileError::MissingBoneTarget {
                    bone_path: bone.bone_path.clone(),
                })?
                .local_transform;
            let parent_world = bone
                .parent_bone_path
                .as_ref()
                .and_then(|parent| bone_world_by_path.get(parent).copied())
                .unwrap_or(skeleton_world);
            let bone_world = combine_transforms(parent_world, local_bone);
            let body_world = combine_transforms(bone_world, bone.body_offset);
            if !transform_is_finite(body_world) {
                return Err(RagdollProfileError::InvalidTransform {
                    bone_path: bone.bone_path.clone(),
                }
                .into());
            }
            prepared.push((bone, body_world));
            bone_world_by_path.insert(bone.bone_path.clone(), bone_world);
        }

        let mut spawn = RagdollSpawn {
            skeleton,
            ..RagdollSpawn::default()
        };
        for (bone, body_world) in prepared {
            let body = world.spawn_node(NodeKind::Empty)?;
            if let Err(source) = configure_body(world, body, skeleton, bone, &spawn, body_world) {
                if source.lifecycle_target_change().is_some() {
                    return Err(source.into());
                }
                let cleanup_cancellation = match rollback_spawn(world, &spawn) {
                    Ok(()) => remove_spawn_body(world, body).err(),
                    Err(cleanup) => Some(cleanup),
                };
                return Err(RagdollSpawnError::SceneMutation {
                    source,
                    cleanup_cancellation,
                });
            }
            spawn.bodies_by_bone.insert(bone.bone_path.clone(), body);
        }
        Ok(spawn)
    }

    pub fn spawn_configured(
        &self,
        world: &mut World,
        skeleton: EntityId,
        mode: RagdollMode,
    ) -> Result<RagdollSpawn, RagdollSpawnError> {
        let spawn = self.spawn(world, skeleton)?;
        if world.get_resource::<RagdollRuntime>().is_none() {
            world.insert_resource(RagdollRuntime::default());
        }
        if let Some(runtime) = world.get_resource_mut::<RagdollRuntime>() {
            runtime.configure_profile(skeleton, mode, self);
        }
        Ok(spawn)
    }
}

impl fmt::Display for RagdollProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(formatter, "ragdoll profile parse failed: {error}"),
            Self::EmptyId => formatter.write_str("ragdoll profile has no id"),
            Self::EmptyProfile => formatter.write_str("ragdoll profile has no bones"),
            Self::EmptyBonePath { index } => write!(formatter, "ragdoll bone {index} has no path"),
            Self::DuplicateBonePath { bone_path } => {
                write!(formatter, "ragdoll bone path is duplicated: {bone_path}")
            }
            Self::MissingParent {
                bone_path,
                parent_bone_path,
            } => write!(
                formatter,
                "ragdoll bone {bone_path} references missing parent {parent_bone_path}"
            ),
            Self::ParentCycle { bone_path } => {
                write!(formatter, "ragdoll bone {bone_path} has a parent cycle")
            }
            Self::InvalidMass { bone_path } => {
                write!(formatter, "ragdoll bone {bone_path} has invalid mass")
            }
            Self::InvalidBlendWeight { bone_path } => {
                write!(
                    formatter,
                    "ragdoll bone {bone_path} has invalid blend weight"
                )
            }
            Self::InvalidShape { bone_path } => {
                write!(formatter, "ragdoll bone {bone_path} has invalid shape")
            }
            Self::InvalidTransform { bone_path } => {
                write!(
                    formatter,
                    "ragdoll bone {bone_path} has invalid body offset"
                )
            }
            Self::MissingSkeletonTargets { skeleton } => {
                write!(formatter, "skeleton {skeleton} has no target pose")
            }
            Self::MissingBoneTarget { bone_path } => {
                write!(
                    formatter,
                    "ragdoll bone {bone_path} has no animation target"
                )
            }
        }
    }
}

impl Error for RagdollProfileError {}

fn configure_body(
    world: &mut World,
    body: EntityId,
    skeleton: EntityId,
    bone: &RagdollBoneProfile,
    spawn: &RagdollSpawn,
    body_world: Transform,
) -> Result<(), SceneError> {
    world
        .update_transform(body, body_world)
        .and_then(|_| {
            world.set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    mass: bone.mass,
                    ..RigidBodyComponent::default()
                }),
            )
        })
        .and_then(|_| {
            world.set_collider(
                body,
                Some(ColliderComponent {
                    shape: scene_shape(&bone.shape),
                    ..ColliderComponent::default()
                }),
            )
        })
        .and_then(|_| {
            let connected_entity = bone
                .parent_bone_path
                .as_ref()
                .and_then(|parent| spawn.bodies_by_bone.get(parent).copied())
                .or(Some(skeleton));
            world.set_joint(
                body,
                Some(JointComponent {
                    joint_type: JointKind::Generic6Dof,
                    connected_entity,
                    constraint: bone.constraint.clone(),
                    skeleton_binding: Some(PhysicsSkeletonJointBinding {
                        skeleton_entity: skeleton,
                        bone_path: bone.bone_path.clone(),
                        parent_bone_path: bone.parent_bone_path.clone(),
                    }),
                    ..JointComponent::default()
                }),
            )
        })
        .map(|_| ())
}

fn remove_spawn_body(
    world: &mut World,
    body: EntityId,
) -> Result<(), DetachedEntityBatchRemoveError> {
    match world.remove_entity_recursive(body) {
        Ok(_) | Err(DetachedEntityBatchRemoveError::Rejected(_)) => Ok(()),
        Err(error @ DetachedEntityBatchRemoveError::Cancelled { .. }) => Err(error),
    }
}

fn rollback_spawn(
    world: &mut World,
    spawn: &RagdollSpawn,
) -> Result<(), DetachedEntityBatchRemoveError> {
    for body in spawn.bodies_by_bone.values().rev() {
        remove_spawn_body(world, *body)?;
    }
    Ok(())
}

struct SkeletalTargetLookup<'a> {
    by_name: HashMap<&'a str, Option<&'a SkeletalPoseTarget>>,
}

impl<'a> SkeletalTargetLookup<'a> {
    fn new(rows: &'a [SkeletalPoseTarget]) -> Self {
        let mut by_name = HashMap::with_capacity(rows.len());
        for target in rows {
            by_name
                .entry(target.bone_name.as_str())
                .and_modify(|target| *target = None)
                .or_insert(Some(target));
        }
        Self { by_name }
    }

    fn resolve(&self, bone_path: &str) -> Option<&'a SkeletalPoseTarget> {
        let leaf = bone_leaf(bone_path);
        let full_path = self.by_name.get(bone_path).copied().flatten();
        if leaf == bone_path {
            return full_path;
        }
        match (full_path, self.by_name.get(leaf).copied().flatten()) {
            (Some(_), Some(_)) => None,
            (Some(target), None) | (None, Some(target)) => Some(target),
            (None, None) => None,
        }
    }
}

fn scene_shape(shape: &PhysicsColliderShape) -> ColliderShape {
    match shape {
        PhysicsColliderShape::Box { half_extents } => ColliderShape::Box {
            half_extents: Vec3::from_array(*half_extents),
        },
        PhysicsColliderShape::Sphere { radius } => ColliderShape::Sphere { radius: *radius },
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => ColliderShape::Capsule {
            radius: *radius,
            half_height: *half_height,
        },
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => ColliderShape::Cylinder {
            radius: *radius,
            half_height: *half_height,
        },
        PhysicsColliderShape::ConvexHull { points } => ColliderShape::ConvexHull {
            points: points.iter().copied().map(Vec3::from_array).collect(),
        },
        PhysicsColliderShape::TriangleMesh { mesh } => {
            ColliderShape::TriangleMesh { mesh: mesh.clone() }
        }
        PhysicsColliderShape::HeightField {
            resolution,
            heights,
        } => ColliderShape::HeightField {
            resolution: *resolution,
            heights: heights.clone(),
        },
        PhysicsColliderShape::Compound { children } => ColliderShape::Compound {
            children: children
                .iter()
                .map(|(transform, child)| (*transform, Box::new(scene_shape(child))))
                .collect(),
        },
    }
}

fn shape_is_valid(shape: &PhysicsColliderShape) -> bool {
    match shape {
        PhysicsColliderShape::Box { half_extents } => half_extents
            .iter()
            .all(|value| value.is_finite() && *value > 0.0),
        PhysicsColliderShape::Sphere { radius } => radius.is_finite() && *radius > 0.0,
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height >= 0.0,
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height > 0.0,
        PhysicsColliderShape::ConvexHull { points } => {
            points.len() >= 4
                && points
                    .iter()
                    .flatten()
                    .all(|coordinate| coordinate.is_finite())
        }
        PhysicsColliderShape::TriangleMesh { .. } => true,
        PhysicsColliderShape::HeightField { resolution, .. } => {
            resolution[0] >= 2 && resolution[1] >= 2
        }
        PhysicsColliderShape::Compound { children } => {
            !children.is_empty() && children.iter().all(|(_, child)| shape_is_valid(child))
        }
    }
}

fn default_body_mass() -> Real {
    DEFAULT_RAGDOLL_BODY_MASS
}

fn default_bone_weight() -> Real {
    1.0
}

fn bone_leaf(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn bone_depth(path: &str) -> usize {
    path.bytes().filter(|byte| *byte == b'/').count()
}

fn topologically_ordered_bones(bones: &[RagdollBoneProfile]) -> Vec<&RagdollBoneProfile> {
    let all_paths = bones
        .iter()
        .map(|bone| bone.bone_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut pending = bones.iter().collect::<Vec<_>>();
    pending.sort_by(|left, right| {
        bone_depth(&left.bone_path)
            .cmp(&bone_depth(&right.bone_path))
            .then_with(|| left.bone_path.cmp(&right.bone_path))
    });
    let mut emitted = BTreeSet::new();
    let mut ordered = Vec::with_capacity(pending.len());
    while !pending.is_empty() {
        let Some(index) = pending.iter().position(|bone| {
            bone.parent_bone_path.as_ref().is_none_or(|parent| {
                !all_paths.contains(parent.as_str()) || emitted.contains(parent.as_str())
            })
        }) else {
            break;
        };
        let bone = pending.remove(index);
        emitted.insert(bone.bone_path.as_str());
        ordered.push(bone);
    }
    ordered
}

fn combine_transforms(parent: Transform, local: Transform) -> Transform {
    Transform {
        translation: parent.translation + parent.rotation * (parent.scale * local.translation),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale * local.scale,
    }
}

fn transform_is_finite(transform: Transform) -> bool {
    transform.translation.is_finite()
        && transform.rotation.is_finite()
        && transform.scale.is_finite()
}
