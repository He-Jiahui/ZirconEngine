use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Quat, Real, Vec3};
use crate::core::resource::ResourceId;

const PROBE_DIRECTION_EPSILON: Real = 0.000001;

// TODO: [CR-ENVIRONMENT-0002] 核实直接构造或反序列化的形状是否必须复验；try_new 可接受未校验的半径与混合距离。下一步统一入口并覆盖渲染消费路径。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProbeInfluenceShape {
    Box {
        half_extents: Vec3,
        blend_distance: Real,
    },
    Sphere {
        radius: Real,
        blend_distance: Real,
    },
}

impl ProbeInfluenceShape {
    pub fn box_shape(
        half_extents: Vec3,
        blend_distance: Real,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !half_extents.is_finite() || half_extents.min_element() <= 0.0 {
            return Err(ReflectionProbeValidationError::InvalidBoxHalfExtents {
                half_extents: half_extents.to_array(),
            });
        }
        validate_blend_distance(blend_distance, half_extents.min_element())?;
        Ok(Self::Box {
            half_extents,
            blend_distance,
        })
    }

    pub fn sphere(
        radius: Real,
        blend_distance: Real,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(ReflectionProbeValidationError::InvalidSphereRadius { radius });
        }
        validate_blend_distance(blend_distance, radius)?;
        Ok(Self::Sphere {
            radius,
            blend_distance,
        })
    }

    pub const fn blend_distance(self) -> Real {
        match self {
            Self::Box { blend_distance, .. } | Self::Sphere { blend_distance, .. } => {
                blend_distance
            }
        }
    }
}

/// 场景提取中的局部反射探针快照，按资源 ID 引用已烘焙 cubemap。
/// 创建时校验位置、旋转与投影范围；形状应先经 ProbeInfluenceShape 构造器校验。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectionProbeData {
    probe_id: u64,
    position: Vec3,
    rotation: Quat,
    shape: ProbeInfluenceShape,
    box_projection: bool,
    projection_half_extents: Vec3,
    baked_cubemap: Option<ResourceId>,
    intensity: Real,
    priority: i32,
    layer_mask: RenderLayerSet,
}

impl ReflectionProbeData {
    pub fn try_new(
        probe_id: u64,
        position: Vec3,
        rotation: Quat,
        shape: ProbeInfluenceShape,
        projection_half_extents: Vec3,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !position.is_finite() {
            return Err(ReflectionProbeValidationError::InvalidPosition {
                position: position.to_array(),
            });
        }
        if !rotation.is_finite() || rotation.length_squared() <= PROBE_DIRECTION_EPSILON {
            return Err(ReflectionProbeValidationError::InvalidRotation {
                rotation: rotation.to_array(),
            });
        }
        if !projection_half_extents.is_finite() || projection_half_extents.min_element() <= 0.0 {
            return Err(
                ReflectionProbeValidationError::InvalidProjectionHalfExtents {
                    half_extents: projection_half_extents.to_array(),
                },
            );
        }
        Ok(Self {
            probe_id,
            position,
            rotation: rotation.normalize(),
            shape,
            box_projection: false,
            projection_half_extents,
            baked_cubemap: None,
            intensity: 1.0,
            priority: 0,
            layer_mask: RenderLayerSet::default(),
        })
    }

    pub const fn probe_id(&self) -> u64 {
        self.probe_id
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }

    pub const fn rotation(&self) -> Quat {
        self.rotation
    }

    pub const fn shape(&self) -> ProbeInfluenceShape {
        self.shape
    }

    pub const fn box_projection(&self) -> bool {
        self.box_projection
    }

    pub const fn projection_half_extents(&self) -> Vec3 {
        self.projection_half_extents
    }

    pub const fn baked_cubemap(&self) -> Option<ResourceId> {
        self.baked_cubemap
    }

    pub const fn intensity(&self) -> Real {
        self.intensity
    }

    pub const fn priority(&self) -> i32 {
        self.priority
    }

    pub const fn layer_mask(&self) -> &RenderLayerSet {
        &self.layer_mask
    }

    pub const fn with_box_projection(mut self, enabled: bool) -> Self {
        self.box_projection = enabled;
        self
    }

    pub const fn with_baked_cubemap(mut self, cubemap: Option<ResourceId>) -> Self {
        self.baked_cubemap = cubemap;
        self
    }

    pub fn try_with_intensity(
        mut self,
        intensity: Real,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !intensity.is_finite() || intensity < 0.0 {
            return Err(ReflectionProbeValidationError::InvalidIntensity { intensity });
        }
        self.intensity = intensity;
        Ok(self)
    }

    pub const fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_layer_mask(mut self, layer_mask: RenderLayerSet) -> Self {
        self.layer_mask = layer_mask;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionProbeBlendEntry {
    pub probe_index: usize,
    pub weight: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionProbeBlend {
    pub primary: Option<ReflectionProbeBlendEntry>,
    pub secondary: Option<ReflectionProbeBlendEntry>,
    pub skybox_weight: Real,
}

impl ReflectionProbeBlend {
    pub const fn skybox_only() -> Self {
        Self {
            primary: None,
            secondary: None,
            skybox_weight: 1.0,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ReflectionProbeValidationError {
    #[error("reflection probe position must be finite, got {position:?}")]
    InvalidPosition { position: [Real; 3] },
    #[error("reflection probe rotation must be finite and nonzero, got {rotation:?}")]
    InvalidRotation { rotation: [Real; 4] },
    #[error("reflection probe box half extents must be finite and positive, got {half_extents:?}")]
    InvalidBoxHalfExtents { half_extents: [Real; 3] },
    #[error("reflection probe sphere radius must be finite and positive, got {radius}")]
    InvalidSphereRadius { radius: Real },
    #[error(
        "reflection probe blend distance must be finite and within [0, {maximum}], got {blend_distance}"
    )]
    InvalidBlendDistance { blend_distance: Real, maximum: Real },
    #[error(
        "reflection probe projection half extents must be finite and positive, got {half_extents:?}"
    )]
    InvalidProjectionHalfExtents { half_extents: [Real; 3] },
    #[error("reflection probe intensity must be finite and nonnegative, got {intensity}")]
    InvalidIntensity { intensity: Real },
}

pub fn reflection_probe_influence_weight(
    probe: &ReflectionProbeData,
    world_position: Vec3,
) -> Real {
    if !world_position.is_finite() {
        return 0.0;
    }
    let position_delta = world_position - probe.position;
    match probe.shape {
        ProbeInfluenceShape::Box {
            half_extents,
            blend_distance,
        } => {
            let local_position = probe.rotation.conjugate() * position_delta;
            let edge_distance = (half_extents - local_position.abs()).min_element();
            boundary_weight(edge_distance, blend_distance)
        }
        ProbeInfluenceShape::Sphere {
            radius,
            blend_distance,
        } => {
            let distance_squared = position_delta.length_squared();
            if radius > 0.0 && distance_squared >= radius * radius {
                0.0
            } else {
                boundary_weight(radius - distance_squared.sqrt(), blend_distance)
            }
        }
    }
}

pub fn reflection_probe_box_project_direction(
    probe: &ReflectionProbeData,
    world_position: Vec3,
    reflection_direction: Vec3,
) -> Vec3 {
    let fallback = normalize_or_zero(reflection_direction);
    if !probe.box_projection || !world_position.is_finite() || fallback == Vec3::ZERO {
        return fallback;
    }

    let world_to_probe = probe.rotation.conjugate();
    let local_position = world_to_probe * (world_position - probe.position);
    let local_direction = world_to_probe * fallback;
    let extent = probe.projection_half_extents;
    let mut distance = Real::INFINITY;
    for axis in 0..3 {
        let direction = local_direction[axis];
        if direction.abs() <= PROBE_DIRECTION_EPSILON {
            continue;
        }
        let plane = if direction > 0.0 {
            extent[axis]
        } else {
            -extent[axis]
        };
        let axis_distance = (plane - local_position[axis]) / direction;
        if axis_distance >= 0.0 {
            distance = distance.min(axis_distance);
        }
    }
    if !distance.is_finite() {
        return fallback;
    }
    let local_hit = local_position + local_direction * distance;
    probe.rotation * local_hit
}

/// 为一个世界位置选出最多两个可用探针，并将剩余权重留给天空环境。
/// 等权候选再按优先级和稳定 ID 排序，使不同探针 ID 的选择不依赖输入排列。
pub fn select_reflection_probe_blend(
    probes: &[ReflectionProbeData],
    world_position: Vec3,
    render_layers: &RenderLayerSet,
) -> ReflectionProbeBlend {
    let mut primary = None;
    let mut secondary = None;
    for (probe_index, probe) in probes.iter().enumerate() {
        if probe.baked_cubemap.is_none()
            || probe.intensity <= 0.0
            || !probe.layer_mask.intersects(render_layers)
        {
            continue;
        }
        let weight = reflection_probe_influence_weight(probe, world_position);
        if weight <= 0.0 {
            continue;
        }
        let candidate = (probe_index, weight, probe.priority, probe.probe_id);
        if primary.is_none_or(|current| reflection_probe_candidate_precedes(candidate, current)) {
            secondary = primary;
            primary = Some(candidate);
        } else if secondary
            .is_none_or(|current| reflection_probe_candidate_precedes(candidate, current))
        {
            secondary = Some(candidate);
        }
    }

    let Some((primary_index, primary_weight, _, _)) = primary else {
        return ReflectionProbeBlend::skybox_only();
    };
    let primary_weight = primary_weight.clamp(0.0, 1.0);
    let primary = Some(ReflectionProbeBlendEntry {
        probe_index: primary_index,
        weight: primary_weight,
    });
    let secondary = secondary.map(|candidate| ReflectionProbeBlendEntry {
        probe_index: candidate.0,
        weight: candidate.1.clamp(0.0, 1.0 - primary_weight),
    });
    let secondary_weight = secondary.map_or(0.0, |entry| entry.weight);
    ReflectionProbeBlend {
        primary,
        secondary,
        skybox_weight: (1.0 - primary_weight - secondary_weight).max(0.0),
    }
}

fn reflection_probe_candidate_precedes(
    left: (usize, Real, i32, u64),
    right: (usize, Real, i32, u64),
) -> bool {
    right
        .1
        .total_cmp(&left.1)
        .then_with(|| right.2.cmp(&left.2))
        .then_with(|| left.3.cmp(&right.3))
        .is_lt()
}

fn validate_blend_distance(
    blend_distance: Real,
    maximum: Real,
) -> Result<(), ReflectionProbeValidationError> {
    if blend_distance.is_finite() && blend_distance >= 0.0 && blend_distance <= maximum {
        return Ok(());
    }
    Err(ReflectionProbeValidationError::InvalidBlendDistance {
        blend_distance,
        maximum,
    })
}

fn boundary_weight(edge_distance: Real, blend_distance: Real) -> Real {
    if edge_distance <= 0.0 {
        return 0.0;
    }
    if blend_distance <= PROBE_DIRECTION_EPSILON {
        return 1.0;
    }
    (edge_distance / blend_distance).clamp(0.0, 1.0)
}

fn normalize_or_zero(value: Vec3) -> Vec3 {
    if !value.is_finite() || value.length_squared() <= PROBE_DIRECTION_EPSILON {
        return Vec3::ZERO;
    }
    value.normalize()
}

#[cfg(test)]
#[path = "tests/reflection_probe.rs"]
mod tests;
