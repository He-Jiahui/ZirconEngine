use std::error::Error;
use std::fmt;

use std::sync::Arc;

use crate::core::framework::render::{
    MaterialPropertyOverrideBlock, RenderMaterialAlphaMode, RenderMaterialPropertyValue,
    RenderMeshBounds, RendererCommon,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{Mat4, Real, Vec4};

use super::change_journal::RenderScenePrimitiveDirtyFlags;
use super::deformation::{RenderSceneSkeletalPose, RenderSceneSkeletalPoseIssue};
use super::mesh_source::{RenderSceneMeshSource, RenderSceneMeshSourceIssue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderScenePrimitiveField {
    WorldFromLocal,
    Tint,
    MorphWeight,
    SkeletalPoseTranslation,
    SkeletalPoseRotation,
    SkeletalPoseScale,
    MaterialAlphaCutoff,
    MaterialPropertyOverride,
    DepthBias,
    LodMinDistance,
    LodMinDistanceOrder,
    LodLocalBoundsCount,
    LocalBoundsMin,
    LocalBoundsMax,
    LocalBoundsOrder,
    WorldBoundsMin,
    WorldBoundsMax,
    WorldBoundsOrder,
}

impl fmt::Display for RenderScenePrimitiveField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WorldFromLocal => "world_from_local",
            Self::Tint => "tint",
            Self::MorphWeight => "morph_weights",
            Self::SkeletalPoseTranslation => "skeletal_pose.bones.local_transform.translation",
            Self::SkeletalPoseRotation => "skeletal_pose.bones.local_transform.rotation",
            Self::SkeletalPoseScale => "skeletal_pose.bones.local_transform.scale",
            Self::MaterialAlphaCutoff => "material_alpha_mode.cutoff",
            Self::MaterialPropertyOverride => "material_property_overrides",
            Self::DepthBias => "depth_bias",
            Self::LodMinDistance => "mesh_source.lods.min_distance",
            Self::LodMinDistanceOrder => "mesh_source.lods.min_distance_order",
            Self::LodLocalBoundsCount => "local_bounds.lod_count",
            Self::LocalBoundsMin => "local_bounds.min",
            Self::LocalBoundsMax => "local_bounds.max",
            Self::LocalBoundsOrder => "local_bounds.min_max_order",
            Self::WorldBoundsMin => "world_bounds.min",
            Self::WorldBoundsMax => "world_bounds.max",
            Self::WorldBoundsOrder => "world_bounds.min_max_order",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderScenePrimitiveInputError {
    stable_instance_key: u64,
    field: RenderScenePrimitiveField,
}

impl RenderScenePrimitiveInputError {
    pub(crate) const fn stable_instance_key(self) -> u64 {
        self.stable_instance_key
    }

    pub(crate) const fn field(self) -> RenderScenePrimitiveField {
        self.field
    }
}

impl fmt::Display for RenderScenePrimitiveInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "render-scene primitive {} has invalid {}",
            self.stable_instance_key, self.field
        )
    }
}

impl Error for RenderScenePrimitiveInputError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderScenePrimitiveDescriptor {
    pub(crate) node_id: EntityId,
    pub(crate) stable_instance_key: u64,
    pub(crate) world_from_local: Mat4,
    pub(crate) mesh_source: RenderSceneMeshSource,
    pub(crate) morph_weights: Arc<[Real]>,
    pub(crate) skeletal_pose: Option<RenderSceneSkeletalPose>,
    pub(crate) tint: Vec4,
    pub(crate) material_property_overrides: MaterialPropertyOverrideBlock,
    pub(crate) material_alpha_mode: RenderMaterialAlphaMode,
    pub(crate) render_queue: i32,
    pub(crate) material_queue: i32,
    pub(crate) order_in_layer: i32,
    pub(crate) depth_bias: Real,
    pub(crate) mobility: Mobility,
    pub(crate) transform_static: bool,
    pub(crate) common: RendererCommon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderScenePrimitiveRevisions {
    pub(crate) transform: u64,
    pub(crate) geometry: u64,
    pub(crate) material: u64,
    pub(crate) bounds: u64,
    pub(crate) deformation: u64,
}

impl RenderScenePrimitiveRevisions {
    pub(crate) const fn new(
        transform: u64,
        geometry: u64,
        material: u64,
        bounds: u64,
        deformation: u64,
    ) -> Self {
        Self {
            transform,
            geometry,
            material,
            bounds,
            deformation,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderScenePrimitiveLocalBounds {
    base: RenderMeshBounds,
    lods: Arc<[RenderMeshBounds]>,
}

impl RenderScenePrimitiveLocalBounds {
    pub(crate) fn new(base: RenderMeshBounds, lods: Vec<RenderMeshBounds>) -> Self {
        Self {
            base,
            lods: lods.into(),
        }
    }

    pub(crate) fn base_only(base: RenderMeshBounds) -> Self {
        Self::new(base, Vec::new())
    }

    pub(crate) const fn base(&self) -> RenderMeshBounds {
        self.base
    }

    pub(crate) fn lods(&self) -> &[RenderMeshBounds] {
        &self.lods
    }

    fn canonicalize(
        self,
        stable_instance_key: u64,
        expected_lod_count: usize,
        canonical_lod_order: &[usize],
    ) -> Result<(Self, RenderMeshBounds), RenderScenePrimitiveInputError> {
        if self.lods.len() != expected_lod_count || canonical_lod_order.len() != expected_lod_count
        {
            return Err(RenderScenePrimitiveInputError {
                stable_instance_key,
                field: RenderScenePrimitiveField::LodLocalBoundsCount,
            });
        }

        let base = canonical_local_bounds(stable_instance_key, self.base)?;
        let mut min = base.min;
        let mut max = base.max;
        let mut lods = Vec::with_capacity(expected_lod_count);
        for source_index in canonical_lod_order.iter().copied() {
            let bounds =
                self.lods
                    .get(source_index)
                    .copied()
                    .ok_or(RenderScenePrimitiveInputError {
                        stable_instance_key,
                        field: RenderScenePrimitiveField::LodLocalBoundsCount,
                    })?;
            let bounds = canonical_local_bounds(stable_instance_key, bounds)?;
            for axis in 0..3 {
                min[axis] = min[axis].min(bounds.min[axis]);
                max[axis] = max[axis].max(bounds.max[axis]);
            }
            lods.push(bounds);
        }
        Ok((
            Self::new(base, lods),
            RenderMeshBounds::from_min_max(min, max),
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderScenePrimitive {
    descriptor: RenderScenePrimitiveDescriptor,
    local_bounds_source: RenderScenePrimitiveLocalBounds,
    local_bounds: RenderMeshBounds,
    world_bounds: RenderMeshBounds,
    revisions: RenderScenePrimitiveRevisions,
}

impl RenderScenePrimitive {
    pub(crate) fn new(
        mut descriptor: RenderScenePrimitiveDescriptor,
        local_bounds: RenderScenePrimitiveLocalBounds,
        revisions: RenderScenePrimitiveRevisions,
    ) -> Result<Self, RenderScenePrimitiveInputError> {
        let canonical_lod_order = canonicalize_mesh_source(&mut descriptor)?;
        validate_descriptor(&descriptor)?;
        let (local_bounds_source, local_bounds) = local_bounds.canonicalize(
            descriptor.stable_instance_key,
            descriptor.mesh_source.lods().len(),
            &canonical_lod_order,
        )?;
        let world_bounds = canonical_world_bounds(
            descriptor.stable_instance_key,
            local_bounds.transformed_by_affine(descriptor.world_from_local),
        )?;
        Ok(Self {
            descriptor,
            local_bounds_source,
            local_bounds,
            world_bounds,
            revisions,
        })
    }

    pub(crate) const fn stable_instance_key(&self) -> u64 {
        self.descriptor.stable_instance_key
    }

    pub(crate) const fn descriptor(&self) -> &RenderScenePrimitiveDescriptor {
        &self.descriptor
    }

    pub(crate) const fn local_bounds(&self) -> RenderMeshBounds {
        self.local_bounds
    }

    pub(crate) const fn world_bounds(&self) -> RenderMeshBounds {
        self.world_bounds
    }

    pub(crate) const fn local_bounds_source(&self) -> &RenderScenePrimitiveLocalBounds {
        &self.local_bounds_source
    }

    pub(crate) const fn revisions(&self) -> RenderScenePrimitiveRevisions {
        self.revisions
    }

    pub(super) fn dirty_from(&self, previous: &Self) -> RenderScenePrimitiveDirtyFlags {
        let mut dirty = RenderScenePrimitiveDirtyFlags::NONE;
        let current = &self.descriptor;
        let old = &previous.descriptor;

        if self.revisions.transform != previous.revisions.transform
            || current.world_from_local != old.world_from_local
        {
            dirty |= RenderScenePrimitiveDirtyFlags::TRANSFORM;
            dirty |= RenderScenePrimitiveDirtyFlags::BOUNDS;
        }

        if !current.mesh_source.geometry_eq(&old.mesh_source)
            || self.revisions.geometry != previous.revisions.geometry
        {
            dirty |= RenderScenePrimitiveDirtyFlags::GEOMETRY;
            dirty |= RenderScenePrimitiveDirtyFlags::VISIBILITY;
        }

        if !current.mesh_source.lod_policy_eq(&old.mesh_source) {
            dirty |= RenderScenePrimitiveDirtyFlags::VISIBILITY;
        }

        if self.local_bounds_source != previous.local_bounds_source
            || self.local_bounds != previous.local_bounds
            || self.revisions.bounds != previous.revisions.bounds
        {
            dirty |= RenderScenePrimitiveDirtyFlags::BOUNDS;
            dirty |= RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS;
        }

        let alpha_phase_changed =
            !same_material_alpha_phase(current.material_alpha_mode, old.material_alpha_mode);

        if !current.mesh_source.materials_eq(&old.mesh_source)
            || current.tint != old.tint
            || self.revisions.material != previous.revisions.material
            || current.material_property_overrides != old.material_property_overrides
            || current.material_alpha_mode != old.material_alpha_mode
            || current.common.material_overrides != old.common.material_overrides
        {
            dirty |= RenderScenePrimitiveDirtyFlags::MATERIAL;
        }

        if current.morph_weights != old.morph_weights
            || current.skeletal_pose != old.skeletal_pose
            || self.revisions.deformation != previous.revisions.deformation
        {
            dirty |= RenderScenePrimitiveDirtyFlags::DEFORMATION;
            dirty |= RenderScenePrimitiveDirtyFlags::BOUNDS;
        }

        if current.mobility != old.mobility
            || current.transform_static != old.transform_static
            || current.common.queue_override != old.common.queue_override
            || current.common.cast_shadows != old.common.cast_shadows
            || current.common.receive_shadows != old.common.receive_shadows
            || current.common.motion_vectors != old.common.motion_vectors
            || current.common.is_static != old.common.is_static
            || current.render_queue != old.render_queue
            || current.material_queue != old.material_queue
            || current.order_in_layer != old.order_in_layer
            || current.depth_bias != old.depth_bias
            || alpha_phase_changed
        {
            dirty |= RenderScenePrimitiveDirtyFlags::RENDER_STATE;
        }

        if current.common.enabled != old.common.enabled
            || current.common.layer_mask != old.common.layer_mask
            || current.common.lod_group != old.common.lod_group
            || current.mobility != old.mobility
            || current.common.cast_shadows != old.common.cast_shadows
            || alpha_phase_changed
        {
            dirty |= RenderScenePrimitiveDirtyFlags::VISIBILITY;
        }

        dirty
    }
}

const fn same_material_alpha_phase(
    current: RenderMaterialAlphaMode,
    previous: RenderMaterialAlphaMode,
) -> bool {
    matches!(
        (current, previous),
        (
            RenderMaterialAlphaMode::Opaque,
            RenderMaterialAlphaMode::Opaque
        ) | (
            RenderMaterialAlphaMode::Mask { .. },
            RenderMaterialAlphaMode::Mask { .. }
        ) | (
            RenderMaterialAlphaMode::Blend,
            RenderMaterialAlphaMode::Blend
        )
    )
}

fn canonicalize_mesh_source(
    descriptor: &mut RenderScenePrimitiveDescriptor,
) -> Result<Vec<usize>, RenderScenePrimitiveInputError> {
    descriptor
        .mesh_source
        .canonicalize_lods()
        .map_err(|issue| RenderScenePrimitiveInputError {
            stable_instance_key: descriptor.stable_instance_key,
            field: match issue {
                RenderSceneMeshSourceIssue::NonFiniteLodDistance
                | RenderSceneMeshSourceIssue::NonPositiveLodDistance => {
                    RenderScenePrimitiveField::LodMinDistance
                }
                RenderSceneMeshSourceIssue::DuplicateLodDistance => {
                    RenderScenePrimitiveField::LodMinDistanceOrder
                }
            },
        })
}

fn validate_descriptor(
    descriptor: &RenderScenePrimitiveDescriptor,
) -> Result<(), RenderScenePrimitiveInputError> {
    let key = descriptor.stable_instance_key;
    let world_from_local = descriptor.world_from_local.to_cols_array();
    validate_finite(
        key,
        RenderScenePrimitiveField::WorldFromLocal,
        world_from_local,
    )?;
    if [
        world_from_local[3],
        world_from_local[7],
        world_from_local[11],
        world_from_local[15],
    ] != [0.0, 0.0, 0.0, 1.0]
    {
        return Err(RenderScenePrimitiveInputError {
            stable_instance_key: key,
            field: RenderScenePrimitiveField::WorldFromLocal,
        });
    }
    validate_finite(
        key,
        RenderScenePrimitiveField::Tint,
        descriptor.tint.to_array(),
    )?;
    if descriptor
        .morph_weights
        .iter()
        .any(|weight| !weight.is_finite())
    {
        return Err(RenderScenePrimitiveInputError {
            stable_instance_key: key,
            field: RenderScenePrimitiveField::MorphWeight,
        });
    }
    if let Some(issue) = descriptor
        .skeletal_pose
        .as_ref()
        .and_then(|pose| pose.validate().err())
    {
        return Err(RenderScenePrimitiveInputError {
            stable_instance_key: key,
            field: match issue {
                RenderSceneSkeletalPoseIssue::NonFiniteTranslation => {
                    RenderScenePrimitiveField::SkeletalPoseTranslation
                }
                RenderSceneSkeletalPoseIssue::NonFiniteRotation => {
                    RenderScenePrimitiveField::SkeletalPoseRotation
                }
                RenderSceneSkeletalPoseIssue::NonFiniteScale => {
                    RenderScenePrimitiveField::SkeletalPoseScale
                }
            },
        });
    }
    if let RenderMaterialAlphaMode::Mask { cutoff } = descriptor.material_alpha_mode {
        if !cutoff.is_finite() || !(0.0..=1.0).contains(&cutoff) {
            return Err(RenderScenePrimitiveInputError {
                stable_instance_key: key,
                field: RenderScenePrimitiveField::MaterialAlphaCutoff,
            });
        }
    }
    if descriptor
        .material_property_overrides
        .values()
        .values()
        .any(|value| !material_property_value_is_finite(value))
    {
        return Err(RenderScenePrimitiveInputError {
            stable_instance_key: key,
            field: RenderScenePrimitiveField::MaterialPropertyOverride,
        });
    }
    validate_finite(
        key,
        RenderScenePrimitiveField::DepthBias,
        [descriptor.depth_bias],
    )?;
    Ok(())
}

fn material_property_value_is_finite(value: &RenderMaterialPropertyValue) -> bool {
    match value {
        RenderMaterialPropertyValue::Bool { .. }
        | RenderMaterialPropertyValue::Int { .. }
        | RenderMaterialPropertyValue::UInt { .. }
        | RenderMaterialPropertyValue::String { .. } => true,
        RenderMaterialPropertyValue::Float { value } => value.is_finite(),
        RenderMaterialPropertyValue::Vec2 { value } => value.iter().all(|value| value.is_finite()),
        RenderMaterialPropertyValue::Vec3 { value } => value.iter().all(|value| value.is_finite()),
        RenderMaterialPropertyValue::Vec4 { value } => value.iter().all(|value| value.is_finite()),
    }
}

fn canonical_local_bounds(
    stable_instance_key: u64,
    bounds: RenderMeshBounds,
) -> Result<RenderMeshBounds, RenderScenePrimitiveInputError> {
    canonical_bounds(
        stable_instance_key,
        bounds,
        RenderScenePrimitiveField::LocalBoundsMin,
        RenderScenePrimitiveField::LocalBoundsMax,
        RenderScenePrimitiveField::LocalBoundsOrder,
    )
}

fn canonical_world_bounds(
    stable_instance_key: u64,
    bounds: RenderMeshBounds,
) -> Result<RenderMeshBounds, RenderScenePrimitiveInputError> {
    canonical_bounds(
        stable_instance_key,
        bounds,
        RenderScenePrimitiveField::WorldBoundsMin,
        RenderScenePrimitiveField::WorldBoundsMax,
        RenderScenePrimitiveField::WorldBoundsOrder,
    )
}

fn canonical_bounds(
    stable_instance_key: u64,
    bounds: RenderMeshBounds,
    min_field: RenderScenePrimitiveField,
    max_field: RenderScenePrimitiveField,
    order_field: RenderScenePrimitiveField,
) -> Result<RenderMeshBounds, RenderScenePrimitiveInputError> {
    validate_finite(stable_instance_key, min_field, bounds.min)?;
    validate_finite(stable_instance_key, max_field, bounds.max)?;
    if bounds
        .min
        .into_iter()
        .zip(bounds.max)
        .any(|(min, max)| min > max)
    {
        return Err(RenderScenePrimitiveInputError {
            stable_instance_key,
            field: order_field,
        });
    }
    Ok(RenderMeshBounds::from_min_max(bounds.min, bounds.max))
}

fn validate_finite<const N: usize>(
    stable_instance_key: u64,
    field: RenderScenePrimitiveField,
    values: [f32; N],
) -> Result<(), RenderScenePrimitiveInputError> {
    if values.into_iter().any(|value| !value.is_finite()) {
        Err(RenderScenePrimitiveInputError {
            stable_instance_key,
            field,
        })
    } else {
        Ok(())
    }
}
