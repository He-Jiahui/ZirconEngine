use std::collections::{BTreeMap, HashMap};

use crate::core::framework::render::{
    GpuLightData, LightShadowSettings, LightingExtract, RenderDirectionalLightSnapshot,
    SHADOW_SLOT_NONE, ViewportCameraSnapshot,
};
use crate::core::math::Mat4;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::VisibilityViewKey;

use super::atlas::{
    SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT, ShadowAtlasAllocator, ShadowAtlasRect,
    ShadowAtlasResourceConfig, ShadowSlotAllocation, ShadowSlotKey, ShadowSlotRequest,
};
use super::cascade::{CascadeSplitConfig, compute_cascade_ranges};
use super::shadow_cache::{
    ShadowCacheInput, shadow_light_params_hash, static_shadow_caster_revision_from_meshes,
};
use super::slot::{
    GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE, GPU_SHADOW_SLOT_FLAG_POINT_FACE,
    GPU_SHADOW_SLOT_FLAG_SPOT, GpuShadowGlobals, GpuShadowSlot,
};
use super::view_projection::{
    directional_cascade_view_projection, point_light_face_view_projection,
    spot_light_view_projection,
};

const POINT_LIGHT_SHADOW_FACE_COUNT: u32 = 6;
const SPOT_LIGHT_SHADOW_SLOT_COUNT: u32 = 1;
const SHADOW_PLAN_NEAR_PLANE: f32 = 0.1;
const SHADOW_REQUEST_PREALLOCATION_SAMPLE_LIGHTS_PER_KIND: usize = 64;
const SHADOW_REQUEST_PREALLOCATION_MINIMUM_SAMPLED_REQUESTS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShadowLightSlotAssignment {
    pub(crate) first_slot: u32,
    pub(crate) slot_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShadowLightSlotAssignments {
    assignments: BTreeMap<u64, ShadowLightSlotAssignment>,
}

impl ShadowLightSlotAssignments {
    pub(crate) fn insert(
        &mut self,
        light_id: u64,
        first_slot: u32,
        slot_count: u32,
    ) -> Option<ShadowLightSlotAssignment> {
        self.assignments.insert(
            light_id,
            ShadowLightSlotAssignment {
                first_slot,
                slot_count,
            },
        )
    }

    pub(crate) fn get(&self, light_id: u64) -> Option<ShadowLightSlotAssignment> {
        self.assignments.get(&light_id).copied()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    pub(crate) fn apply_to_packed_lights(
        &self,
        lighting: &LightingExtract,
        lights: &mut [GpuLightData],
    ) {
        let mut light_index = 0usize;
        for light in &lighting.directional_lights {
            apply_assignment_to_light(self.get(light.light_id), lights.get_mut(light_index));
            light_index += 1;
        }
        for light in &lighting.point_lights {
            apply_assignment_to_light(self.get(light.light_id), lights.get_mut(light_index));
            light_index += 1;
        }
        for light in &lighting.spot_lights {
            apply_assignment_to_light(self.get(light.light_id), lights.get_mut(light_index));
            light_index += 1;
        }
        for light in &lighting.rect_lights {
            apply_assignment_to_light(self.get(light.light_id), lights.get_mut(light_index));
            light_index += 1;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShadowFramePlan {
    slots: Vec<GpuShadowSlot>,
    atlas_passes: Vec<ShadowAtlasSlotPass>,
    globals: GpuShadowGlobals,
    light_slots: ShadowLightSlotAssignments,
    static_caster_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowAtlasSlotPass {
    pub(crate) slot_index: u32,
    pub(crate) slot_key: ShadowSlotKey,
    pub(crate) atlas_slot_generation: u64,
    pub(crate) rect: ShadowAtlasRect,
    pub(crate) view_proj: Mat4,
    pub(crate) view_key: Option<VisibilityViewKey>,
}

impl ShadowAtlasSlotPass {
    fn new(
        slot_index: u32,
        slot_key: ShadowSlotKey,
        atlas_slot_generation: u64,
        rect: ShadowAtlasRect,
        view_proj: Mat4,
        view_key: Option<VisibilityViewKey>,
    ) -> Self {
        Self {
            slot_index,
            slot_key,
            atlas_slot_generation,
            rect,
            view_proj,
            view_key,
        }
    }
}

impl ShadowFramePlan {
    pub(crate) fn disabled(atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            slots: Vec::new(),
            atlas_passes: Vec::new(),
            globals: GpuShadowGlobals::disabled(atlas_width, atlas_height),
            light_slots: ShadowLightSlotAssignments::default(),
            static_caster_revision: None,
        }
    }

    pub(crate) fn slots(&self) -> &[GpuShadowSlot] {
        &self.slots
    }

    pub(crate) fn atlas_passes(&self) -> &[ShadowAtlasSlotPass] {
        &self.atlas_passes
    }

    pub(crate) fn globals(&self) -> GpuShadowGlobals {
        self.globals
    }

    pub(crate) fn light_slots(&self) -> &ShadowLightSlotAssignments {
        &self.light_slots
    }

    /// Returns cache inputs only when every static caster has authoritative revisions.
    pub(crate) fn static_shadow_cache_inputs(&self) -> Vec<ShadowCacheInput> {
        let Some(static_caster_revision) = self.static_caster_revision else {
            return Vec::new();
        };
        // The plan producer appends each pass immediately after its slot. Require that complete
        // correspondence before cache identity is exposed, so malformed plans redraw instead of
        // silently reusing only a subset of atlas depth.
        if self.atlas_passes.len() != self.slots.len()
            || self
                .atlas_passes
                .iter()
                .enumerate()
                .any(|(index, pass)| u32::try_from(index).ok() != Some(pass.slot_index))
        {
            return Vec::new();
        }
        self.atlas_passes
            .iter()
            .zip(&self.slots)
            .map(|(pass, slot)| {
                ShadowCacheInput::new(
                    pass.slot_key,
                    shadow_light_params_hash(slot),
                    static_caster_revision,
                    pass.atlas_slot_generation,
                )
            })
            .collect()
    }
}

pub(crate) fn build_shadow_frame_plan(
    allocator: &mut ShadowAtlasAllocator,
    frame: &ViewportRenderFrame,
    resource_config: ShadowAtlasResourceConfig,
) -> ShadowFramePlan {
    let static_caster_revision =
        static_shadow_caster_revision_from_meshes(&frame.extract.geometry.meshes);
    build_shadow_frame_plan_with_static_caster_revision(
        allocator,
        frame,
        resource_config,
        static_caster_revision,
    )
}

pub(crate) fn build_shadow_frame_plan_with_static_caster_revision(
    allocator: &mut ShadowAtlasAllocator,
    frame: &ViewportRenderFrame,
    resource_config: ShadowAtlasResourceConfig,
    static_caster_revision: Option<u64>,
) -> ShadowFramePlan {
    let resource_config = resource_config.normalized();
    if !frame.preview().lighting_enabled {
        allocator.allocate_frame(&[]);
        return ShadowFramePlan::disabled(resource_config.width, resource_config.height);
    }

    let lighting = &frame.extract.lighting;
    let directional = first_shadow_casting_directional(&lighting.directional_lights);
    let additional_requests = shadow_slot_requests_for_additional_lights(lighting);
    let atlas_allocation = allocator.allocate_frame(&additional_requests);
    let allocations_by_key = atlas_allocation
        .allocations
        .iter()
        .copied()
        .filter_map(|allocation| {
            atlas_allocation
                .slot_generation_for(allocation.key)
                .map(|generation| (allocation.key, (allocation, generation)))
        })
        .collect::<HashMap<_, _>>();

    let mut slots = Vec::new();
    let mut atlas_passes = Vec::new();
    let mut light_slots = ShadowLightSlotAssignments::default();
    let camera = frame.effective_camera();
    let globals = if let Some(light) = directional {
        let remaining_slots = slots_remaining(resource_config, slots.len());
        append_directional_cascades(
            &mut slots,
            &mut atlas_passes,
            &mut light_slots,
            light,
            &camera,
            resource_config,
            remaining_slots,
        )
    } else {
        GpuShadowGlobals::disabled(resource_config.width, resource_config.height)
    };

    append_point_light_slots(
        &mut slots,
        &mut atlas_passes,
        &mut light_slots,
        lighting,
        &allocations_by_key,
        resource_config,
    );
    append_spot_light_slots(
        &mut slots,
        &mut atlas_passes,
        &mut light_slots,
        lighting,
        &allocations_by_key,
        resource_config,
    );

    ShadowFramePlan {
        slots,
        atlas_passes,
        globals,
        light_slots,
        static_caster_revision,
    }
}

fn apply_assignment_to_light(
    assignment: Option<ShadowLightSlotAssignment>,
    light: Option<&mut GpuLightData>,
) {
    let Some(light) = light else {
        return;
    };
    let Some(assignment) = assignment else {
        light.shadow_slot_layer[0] = SHADOW_SLOT_NONE;
        light.shadow_params[3] = 0.0;
        return;
    };
    light.shadow_slot_layer[0] = assignment.first_slot;
    light.shadow_params[3] = assignment.slot_count as f32;
}

fn first_shadow_casting_directional(
    lights: &[RenderDirectionalLightSnapshot],
) -> Option<&RenderDirectionalLightSnapshot> {
    lights
        .iter()
        .find(|light| shadow_enabled(light.shadow).is_some())
}

fn shadow_slot_request_capacity_if_dense(lighting: &LightingExtract) -> Option<usize> {
    let sampled_point_requests = lighting
        .point_lights
        .iter()
        .take(SHADOW_REQUEST_PREALLOCATION_SAMPLE_LIGHTS_PER_KIND)
        .filter(|light| shadow_enabled(light.shadow).is_some())
        .count()
        .saturating_mul(POINT_LIGHT_SHADOW_FACE_COUNT as usize);
    let sampled_spot_requests = lighting
        .spot_lights
        .iter()
        .take(SHADOW_REQUEST_PREALLOCATION_SAMPLE_LIGHTS_PER_KIND)
        .filter(|light| shadow_enabled(light.shadow).is_some())
        .count()
        .saturating_mul(SPOT_LIGHT_SHADOW_SLOT_COUNT as usize);
    if sampled_point_requests.saturating_add(sampled_spot_requests)
        < SHADOW_REQUEST_PREALLOCATION_MINIMUM_SAMPLED_REQUESTS
    {
        return None;
    }

    let point_requests = lighting
        .point_lights
        .iter()
        .filter(|light| shadow_enabled(light.shadow).is_some())
        .count()
        .saturating_mul(POINT_LIGHT_SHADOW_FACE_COUNT as usize);
    let spot_requests = lighting
        .spot_lights
        .iter()
        .filter(|light| shadow_enabled(light.shadow).is_some())
        .count()
        .saturating_mul(SPOT_LIGHT_SHADOW_SLOT_COUNT as usize);
    Some(point_requests.saturating_add(spot_requests))
}

fn shadow_slot_requests_for_additional_lights(
    lighting: &LightingExtract,
) -> Vec<ShadowSlotRequest> {
    let mut requests =
        shadow_slot_request_capacity_if_dense(lighting).map_or_else(Vec::new, Vec::with_capacity);
    append_shadow_slot_requests(&mut requests, lighting);
    requests
}

fn append_shadow_slot_requests(requests: &mut Vec<ShadowSlotRequest>, lighting: &LightingExtract) {
    for light in &lighting.point_lights {
        let Some(shadow) = shadow_enabled(light.shadow) else {
            continue;
        };
        for face in 0..POINT_LIGHT_SHADOW_FACE_COUNT {
            requests.push(
                ShadowSlotRequest::new(
                    ShadowSlotKey::new(light.light_id, face as u8),
                    shadow.resolution_preference,
                    punctual_priority(light.intensity, light.range),
                )
                .with_minimum_tier(shadow.resolution_preference.minimum_with_global_floor()),
            );
        }
    }
    for light in &lighting.spot_lights {
        let Some(shadow) = shadow_enabled(light.shadow) else {
            continue;
        };
        requests.push(
            ShadowSlotRequest::new(
                ShadowSlotKey::new(light.light_id, 0),
                shadow.resolution_preference,
                punctual_priority(light.intensity, light.range),
            )
            .with_minimum_tier(shadow.resolution_preference.minimum_with_global_floor()),
        );
    }
}

fn append_directional_cascades(
    slots: &mut Vec<GpuShadowSlot>,
    atlas_passes: &mut Vec<ShadowAtlasSlotPass>,
    light_slots: &mut ShadowLightSlotAssignments,
    light: &RenderDirectionalLightSnapshot,
    camera: &ViewportCameraSnapshot,
    resource_config: ShadowAtlasResourceConfig,
    remaining_slots: u32,
) -> GpuShadowGlobals {
    let Some(shadow) = shadow_enabled(light.shadow) else {
        return GpuShadowGlobals::disabled(resource_config.width, resource_config.height);
    };
    let cascade_config = CascadeSplitConfig::default();
    let ranges = compute_cascade_ranges(&cascade_config, SHADOW_PLAN_NEAR_PLANE);
    let cascade_count = ranges.len().min(remaining_slots as usize);
    if cascade_count == 0 {
        return GpuShadowGlobals::disabled(resource_config.width, resource_config.height);
    }
    let cascade_tier =
        directional_cascade_tier(shadow.resolution_preference, resource_config, cascade_count);

    let first_slot = slots.len() as u32;
    for cascade_index in 0..cascade_count {
        let allocation =
            directional_cascade_allocation(light, shadow, cascade_tier, cascade_index as u32);
        let view_proj = directional_cascade_view_projection(
            light,
            camera,
            cascade_tier.size_px(),
            ranges[cascade_index],
        );
        let slot_index = slots.len() as u32;
        atlas_passes.push(ShadowAtlasSlotPass::new(
            slot_index,
            allocation.key,
            u64::from(cascade_index as u32) + 1,
            allocation.rect,
            view_proj,
            Some(VisibilityViewKey::ShadowCascade {
                light: light.node_id,
                cascade: cascade_index as u8,
            }),
        ));
        slots.push(GpuShadowSlot::from_allocation(
            allocation,
            view_proj,
            resource_config.width,
            resource_config.height,
            shadow.depth_bias,
            shadow.normal_bias,
            shadow.pcf_quality,
            GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE,
        ));
    }
    light_slots.insert(light.light_id, first_slot, cascade_count as u32);
    GpuShadowGlobals::from_cascade_ranges(&ranges, resource_config.width, resource_config.height)
}

fn append_point_light_slots(
    slots: &mut Vec<GpuShadowSlot>,
    atlas_passes: &mut Vec<ShadowAtlasSlotPass>,
    light_slots: &mut ShadowLightSlotAssignments,
    lighting: &LightingExtract,
    allocations_by_key: &HashMap<ShadowSlotKey, (ShadowSlotAllocation, u64)>,
    resource_config: ShadowAtlasResourceConfig,
) {
    for light in &lighting.point_lights {
        let Some(shadow) = shadow_enabled(light.shadow) else {
            continue;
        };
        if slots_remaining(resource_config, slots.len()) < POINT_LIGHT_SHADOW_FACE_COUNT {
            return;
        }
        let allocations: [Option<(ShadowSlotAllocation, u64)>;
            POINT_LIGHT_SHADOW_FACE_COUNT as usize] = std::array::from_fn(|face| {
            allocations_by_key
                .get(&ShadowSlotKey::new(light.light_id, face as u8))
                .copied()
        });
        if allocations.iter().any(Option::is_none) {
            continue;
        }
        let first_slot = slots.len() as u32;
        for (allocation, generation) in allocations.into_iter().flatten() {
            let view_proj = point_light_face_view_projection(light, allocation.key.face_index);
            let slot_index = slots.len() as u32;
            atlas_passes.push(ShadowAtlasSlotPass::new(
                slot_index,
                allocation.key,
                generation,
                allocation.rect,
                view_proj,
                Some(VisibilityViewKey::ShadowPointFace {
                    light: light.node_id,
                    face: allocation.key.face_index,
                }),
            ));
            slots.push(GpuShadowSlot::from_allocation(
                allocation,
                view_proj,
                resource_config.width,
                resource_config.height,
                shadow.depth_bias,
                shadow.normal_bias,
                shadow.pcf_quality,
                GPU_SHADOW_SLOT_FLAG_POINT_FACE,
            ));
        }
        light_slots.insert(light.light_id, first_slot, POINT_LIGHT_SHADOW_FACE_COUNT);
    }
}

fn append_spot_light_slots(
    slots: &mut Vec<GpuShadowSlot>,
    atlas_passes: &mut Vec<ShadowAtlasSlotPass>,
    light_slots: &mut ShadowLightSlotAssignments,
    lighting: &LightingExtract,
    allocations_by_key: &HashMap<ShadowSlotKey, (ShadowSlotAllocation, u64)>,
    resource_config: ShadowAtlasResourceConfig,
) {
    for light in &lighting.spot_lights {
        let Some(shadow) = shadow_enabled(light.shadow) else {
            continue;
        };
        if slots_remaining(resource_config, slots.len()) < SPOT_LIGHT_SHADOW_SLOT_COUNT {
            return;
        }
        let Some((allocation, generation)) = allocations_by_key
            .get(&ShadowSlotKey::new(light.light_id, 0))
            .copied()
        else {
            continue;
        };
        let first_slot = slots.len() as u32;
        let view_proj = spot_light_view_projection(light);
        atlas_passes.push(ShadowAtlasSlotPass::new(
            first_slot,
            allocation.key,
            generation,
            allocation.rect,
            view_proj,
            Some(VisibilityViewKey::ShadowSpot {
                light: light.node_id,
            }),
        ));
        slots.push(GpuShadowSlot::from_allocation(
            allocation,
            view_proj,
            resource_config.width,
            resource_config.height,
            shadow.depth_bias,
            shadow.normal_bias,
            shadow.pcf_quality,
            GPU_SHADOW_SLOT_FLAG_SPOT,
        ));
        light_slots.insert(light.light_id, first_slot, SPOT_LIGHT_SHADOW_SLOT_COUNT);
    }
}

fn directional_cascade_allocation(
    light: &RenderDirectionalLightSnapshot,
    shadow: LightShadowSettings,
    allocated_tier: crate::core::framework::render::ShadowResolutionTier,
    cascade_index: u32,
) -> ShadowSlotAllocation {
    let size = allocated_tier.size_px();
    ShadowSlotAllocation {
        key: ShadowSlotKey::new(light.light_id, cascade_index as u8),
        rect: ShadowAtlasRect::new(cascade_index * size, 0, size, size),
        requested_tier: shadow.resolution_preference,
        allocated_tier,
        priority: directional_priority(light.intensity),
        reused_previous: false,
    }
}

fn directional_cascade_tier(
    preferred: crate::core::framework::render::ShadowResolutionTier,
    resource_config: ShadowAtlasResourceConfig,
    cascade_count: usize,
) -> crate::core::framework::render::ShadowResolutionTier {
    let max_size = resource_config
        .width
        .checked_div(cascade_count.max(1) as u32)
        .unwrap_or(1)
        .min(
            resource_config
                .height
                .min(SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT),
        )
        .max(1);
    let mut tier = preferred;
    while tier.size_px() > max_size {
        let Some(lower) = tier.next_lower() else {
            return tier;
        };
        tier = lower;
    }
    tier
}

fn shadow_enabled(shadow: Option<LightShadowSettings>) -> Option<LightShadowSettings> {
    shadow.filter(|settings| settings.casts_shadow)
}

fn directional_priority(intensity: f32) -> f32 {
    sanitize_priority(intensity) + 1_000_000.0
}

fn punctual_priority(intensity: f32, range: f32) -> f32 {
    sanitize_priority(intensity) * sanitize_priority(range).max(1.0)
}

fn sanitize_priority(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn slots_remaining(resource_config: ShadowAtlasResourceConfig, current_len: usize) -> u32 {
    resource_config
        .slot_capacity
        .saturating_sub(current_len as u32)
}

trait ShadowResolutionTierMinimum {
    fn minimum_with_global_floor(self) -> crate::core::framework::render::ShadowResolutionTier;
}

impl ShadowResolutionTierMinimum for crate::core::framework::render::ShadowResolutionTier {
    fn minimum_with_global_floor(self) -> crate::core::framework::render::ShadowResolutionTier {
        crate::core::framework::render::ShadowResolutionTier::MIN
    }
}

#[cfg(test)]
mod tests;
