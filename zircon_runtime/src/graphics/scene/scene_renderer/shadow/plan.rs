use std::collections::{BTreeMap, HashMap};

use crate::core::framework::render::{
    GpuLightData, LightShadowSettings, LightingExtract, RenderDirectionalLightSnapshot,
    ViewportCameraSnapshot, SHADOW_SLOT_NONE,
};
use crate::core::math::Mat4;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::VisibilityViewKey;

use super::atlas::{
    ShadowAtlasAllocator, ShadowAtlasRect, ShadowAtlasResourceConfig, ShadowSlotAllocation,
    ShadowSlotKey, ShadowSlotRequest, SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT,
};
use super::cascade::{compute_cascade_ranges, CascadeSplitConfig};
use super::slot::{
    GpuShadowGlobals, GpuShadowSlot, GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE,
    GPU_SHADOW_SLOT_FLAG_POINT_FACE, GPU_SHADOW_SLOT_FLAG_SPOT,
};
use super::view_projection::{
    directional_cascade_view_projection, point_light_face_view_projection,
    spot_light_view_projection,
};

const POINT_LIGHT_SHADOW_FACE_COUNT: u32 = 6;
const SPOT_LIGHT_SHADOW_SLOT_COUNT: u32 = 1;
const SHADOW_PLAN_NEAR_PLANE: f32 = 0.1;

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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowAtlasSlotPass {
    pub(crate) slot_index: u32,
    pub(crate) rect: ShadowAtlasRect,
    pub(crate) view_proj: Mat4,
    pub(crate) view_key: Option<VisibilityViewKey>,
}

impl ShadowAtlasSlotPass {
    fn new(
        slot_index: u32,
        rect: ShadowAtlasRect,
        view_proj: Mat4,
        view_key: Option<VisibilityViewKey>,
    ) -> Self {
        Self {
            slot_index,
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
}

pub(crate) fn build_shadow_frame_plan(
    allocator: &mut ShadowAtlasAllocator,
    frame: &ViewportRenderFrame,
    resource_config: ShadowAtlasResourceConfig,
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
        .map(|allocation| (allocation.key, allocation))
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

fn shadow_slot_requests_for_additional_lights(
    lighting: &LightingExtract,
) -> Vec<ShadowSlotRequest> {
    let mut requests = Vec::new();
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
    requests
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
    allocations_by_key: &HashMap<ShadowSlotKey, ShadowSlotAllocation>,
    resource_config: ShadowAtlasResourceConfig,
) {
    for light in &lighting.point_lights {
        let Some(shadow) = shadow_enabled(light.shadow) else {
            continue;
        };
        if slots_remaining(resource_config, slots.len()) < POINT_LIGHT_SHADOW_FACE_COUNT {
            return;
        }
        let allocations = (0..POINT_LIGHT_SHADOW_FACE_COUNT)
            .map(|face| {
                allocations_by_key
                    .get(&ShadowSlotKey::new(light.light_id, face as u8))
                    .copied()
            })
            .collect::<Option<Vec<_>>>();
        let Some(allocations) = allocations else {
            continue;
        };
        let first_slot = slots.len() as u32;
        for allocation in allocations {
            let view_proj = point_light_face_view_projection(light, allocation.key.face_index);
            let slot_index = slots.len() as u32;
            atlas_passes.push(ShadowAtlasSlotPass::new(
                slot_index,
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
    allocations_by_key: &HashMap<ShadowSlotKey, ShadowSlotAllocation>,
    resource_config: ShadowAtlasResourceConfig,
) {
    for light in &lighting.spot_lights {
        let Some(shadow) = shadow_enabled(light.shadow) else {
            continue;
        };
        if slots_remaining(resource_config, slots.len()) < SPOT_LIGHT_SHADOW_SLOT_COUNT {
            return;
        }
        let Some(allocation) = allocations_by_key
            .get(&ShadowSlotKey::new(light.light_id, 0))
            .copied()
        else {
            continue;
        };
        let first_slot = slots.len() as u32;
        let view_proj = spot_light_view_projection(light);
        atlas_passes.push(ShadowAtlasSlotPass::new(
            first_slot,
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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        FallbackSkyboxKind, LightShadowSettings, PreviewEnvironmentExtract, RenderFrameExtract,
        RenderLayerSet, RenderOverlayExtract, RenderPointLightSnapshot, RenderSceneGeometryExtract,
        RenderSceneSnapshot, RenderSpotLightSnapshot, RenderWorldSnapshotHandle, ShadowPcfQuality,
        ShadowResolutionTier, ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
    };
    use crate::core::math::{Transform, UVec2, Vec3, Vec4};
    use crate::graphics::scene::scene_renderer::shadow::slot::{
        GPU_SHADOW_SLOT_PCF_QUALITY_HIGH, GPU_SHADOW_SLOT_PCF_QUALITY_LOW,
        GPU_SHADOW_SLOT_PCF_QUALITY_MASK, GPU_SHADOW_SLOT_PCF_QUALITY_MEDIUM,
    };

    fn shadow_frame(lighting: LightingExtract) -> ViewportRenderFrame {
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, 0.0, 12.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: lighting.directional_lights,
                point_lights: lighting.point_lights,
                spot_lights: lighting.spot_lights,
                ambient_lights: lighting.ambient_lights,
                rect_lights: lighting.rect_lights,
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(42), snapshot);
        ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240))
    }

    fn shadow(tier: ShadowResolutionTier) -> LightShadowSettings {
        shadow_with_quality(tier, ShadowPcfQuality::High)
    }

    fn default_light_layer_mask() -> RenderLayerSet {
        RenderLayerSet::from_legacy_mask(DEFAULT_RENDER_LAYER_MASK)
    }

    fn shadow_with_quality(
        tier: ShadowResolutionTier,
        pcf_quality: ShadowPcfQuality,
    ) -> LightShadowSettings {
        LightShadowSettings {
            casts_shadow: true,
            depth_bias: 0.25,
            normal_bias: 0.5,
            strength: 0.75,
            resolution_preference: tier,
            pcf_quality,
        }
    }

    #[test]
    fn render_shadow_frame_plan_assigns_first_directional_cascade_slots() {
        let mut allocator = ShadowAtlasAllocator::default();
        let lighting = LightingExtract {
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 1,
                light_id: 101,
                layer_mask: default_light_layer_mask(),
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::ONE,
                intensity: 2.0,
                shadow: Some(shadow(ShadowResolutionTier::T1024)),
            }],
            ..LightingExtract::default()
        };

        let frame = shadow_frame(lighting);
        let plan = build_shadow_frame_plan(
            &mut allocator,
            &frame,
            ShadowAtlasResourceConfig::new(4096, 4096, 16),
        );

        assert_eq!(plan.slots().len(), 4);
        assert_eq!(plan.atlas_passes().len(), 4);
        assert_eq!(
            plan.light_slots().get(101),
            Some(ShadowLightSlotAssignment {
                first_slot: 0,
                slot_count: 4
            })
        );
        assert!(plan.globals().cascade_splits[0] > 0.0);
        assert_ne!(
            plan.slots()[0].flags_bits() & GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE,
            0
        );
        assert_eq!(
            plan.slots()[0].flags_bits() & GPU_SHADOW_SLOT_PCF_QUALITY_MASK,
            GPU_SHADOW_SLOT_PCF_QUALITY_HIGH
        );
        assert_ne!(plan.slots()[0].view_proj, Mat4::IDENTITY.to_cols_array_2d());
        assert_eq!(plan.atlas_passes()[0].slot_index, 0);
        assert_eq!(plan.atlas_passes()[0].rect.width, 1024);
        assert_eq!(
            plan.atlas_passes()[0].view_key,
            Some(VisibilityViewKey::ShadowCascade {
                light: 1,
                cascade: 0
            })
        );
        assert_ne!(plan.atlas_passes()[0].view_proj, Mat4::IDENTITY);
    }

    #[test]
    fn render_shadow_frame_plan_builds_distinct_directional_cascade_matrices() {
        let mut allocator = ShadowAtlasAllocator::default();
        let lighting = LightingExtract {
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 1,
                light_id: 101,
                layer_mask: default_light_layer_mask(),
                direction: Vec3::new(0.0, -1.0, -1.0),
                color: Vec3::ONE,
                intensity: 2.0,
                shadow: Some(shadow(ShadowResolutionTier::T1024)),
            }],
            ..LightingExtract::default()
        };

        let frame = shadow_frame(lighting);
        let plan = build_shadow_frame_plan(
            &mut allocator,
            &frame,
            ShadowAtlasResourceConfig::new(4096, 4096, 16),
        );

        assert_eq!(plan.atlas_passes().len(), 4);
        assert_ne!(
            plan.atlas_passes()[0].view_proj,
            plan.atlas_passes()[3].view_proj
        );
        assert_ne!(plan.slots()[0].view_proj, plan.slots()[3].view_proj);
    }

    #[test]
    fn render_shadow_frame_plan_caps_directional_cascade_tier_to_atlas_row() {
        let mut allocator = ShadowAtlasAllocator::default();
        let lighting = LightingExtract {
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 1,
                light_id: 404,
                layer_mask: default_light_layer_mask(),
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::ONE,
                intensity: 2.0,
                shadow: Some(shadow(ShadowResolutionTier::T2048)),
            }],
            ..LightingExtract::default()
        };

        let frame = shadow_frame(lighting);
        let plan = build_shadow_frame_plan(
            &mut allocator,
            &frame,
            ShadowAtlasResourceConfig::new(2048, 2048, 16),
        );

        assert_eq!(plan.slots().len(), 4);
        assert_eq!(plan.atlas_passes().len(), 4);
        assert_eq!(plan.slots()[0].atlas_scale_bias[0], 0.25);
        assert_eq!(plan.slots()[0].params[2], 1.0 / 512.0);
    }

    #[test]
    fn render_shadow_frame_plan_assigns_point_light_contiguous_face_slots() {
        let mut allocator =
            ShadowAtlasAllocator::new(super::super::atlas::ShadowAtlasConfig::new_square(1024));
        let lighting = LightingExtract {
            point_lights: vec![RenderPointLightSnapshot {
                node_id: 2,
                light_id: 202,
                layer_mask: default_light_layer_mask(),
                position: Vec3::ZERO,
                color: Vec3::ONE,
                intensity: 4.0,
                range: 8.0,
                shadow: Some(shadow(ShadowResolutionTier::T256)),
            }],
            ..LightingExtract::default()
        };

        let frame = shadow_frame(lighting);
        let plan = build_shadow_frame_plan(
            &mut allocator,
            &frame,
            ShadowAtlasResourceConfig::new(1024, 1024, 16),
        );

        assert_eq!(plan.slots().len(), 6);
        assert_eq!(plan.atlas_passes().len(), 6);
        assert_eq!(
            plan.light_slots().get(202),
            Some(ShadowLightSlotAssignment {
                first_slot: 0,
                slot_count: POINT_LIGHT_SHADOW_FACE_COUNT
            })
        );
        assert!(plan
            .slots()
            .iter()
            .all(|slot| slot.flags_bits() & GPU_SHADOW_SLOT_FLAG_POINT_FACE != 0));
        assert!(plan
            .slots()
            .iter()
            .all(|slot| slot.view_proj != Mat4::IDENTITY.to_cols_array_2d()));
        assert!(plan
            .atlas_passes()
            .iter()
            .all(|slot_pass| slot_pass.view_proj != Mat4::IDENTITY));
        assert_eq!(
            plan.atlas_passes()
                .iter()
                .map(|slot_pass| slot_pass.view_key)
                .collect::<Vec<_>>(),
            (0..POINT_LIGHT_SHADOW_FACE_COUNT)
                .map(|face| Some(VisibilityViewKey::ShadowPointFace {
                    light: 2,
                    face: face as u8
                }))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn render_shadow_frame_plan_assigns_spot_light_slot_view_key() {
        let mut allocator =
            ShadowAtlasAllocator::new(super::super::atlas::ShadowAtlasConfig::new_square(1024));
        let lighting = LightingExtract {
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 3,
                light_id: 303,
                layer_mask: default_light_layer_mask(),
                position: Vec3::ZERO,
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::ONE,
                intensity: 3.0,
                range: 6.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
                shadow: Some(shadow(ShadowResolutionTier::T512)),
            }],
            ..LightingExtract::default()
        };

        let frame = shadow_frame(lighting);
        let plan = build_shadow_frame_plan(
            &mut allocator,
            &frame,
            ShadowAtlasResourceConfig::new(1024, 1024, 16),
        );

        assert_eq!(plan.atlas_passes().len(), 1);
        assert_eq!(
            plan.atlas_passes()[0].view_key,
            Some(VisibilityViewKey::ShadowSpot { light: 3 })
        );
    }

    #[test]
    fn render_shadow_frame_plan_encodes_per_light_pcf_quality() {
        let mut allocator =
            ShadowAtlasAllocator::new(super::super::atlas::ShadowAtlasConfig::new_square(1024));
        let lighting = LightingExtract {
            point_lights: vec![RenderPointLightSnapshot {
                node_id: 2,
                light_id: 202,
                layer_mask: default_light_layer_mask(),
                position: Vec3::ZERO,
                color: Vec3::ONE,
                intensity: 4.0,
                range: 8.0,
                shadow: Some(shadow_with_quality(
                    ShadowResolutionTier::T256,
                    ShadowPcfQuality::Low,
                )),
            }],
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 3,
                light_id: 303,
                layer_mask: default_light_layer_mask(),
                position: Vec3::ZERO,
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::ONE,
                intensity: 3.0,
                range: 6.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
                shadow: Some(shadow_with_quality(
                    ShadowResolutionTier::T512,
                    ShadowPcfQuality::Medium,
                )),
            }],
            ..LightingExtract::default()
        };

        let frame = shadow_frame(lighting);
        let plan = build_shadow_frame_plan(
            &mut allocator,
            &frame,
            ShadowAtlasResourceConfig::new(1024, 1024, 16),
        );

        assert_eq!(plan.slots().len(), 7);
        assert!(plan.slots()[..6].iter().all(|slot| {
            slot.flags_bits() & GPU_SHADOW_SLOT_PCF_QUALITY_MASK == GPU_SHADOW_SLOT_PCF_QUALITY_LOW
        }));
        assert_eq!(
            plan.slots()[6].flags_bits() & GPU_SHADOW_SLOT_PCF_QUALITY_MASK,
            GPU_SHADOW_SLOT_PCF_QUALITY_MEDIUM
        );
    }

    #[test]
    fn render_shadow_light_slot_assignments_patch_packed_light_contract() {
        let mut assignments = ShadowLightSlotAssignments::default();
        assignments.insert(303, 7, 1);
        let lighting = LightingExtract {
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 3,
                light_id: 303,
                layer_mask: default_light_layer_mask(),
                position: Vec3::ZERO,
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::ONE,
                intensity: 3.0,
                range: 6.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
                shadow: Some(shadow(ShadowResolutionTier::T512)),
            }],
            ..LightingExtract::default()
        };
        let mut lights = vec![GpuLightData {
            shadow_slot_layer: [SHADOW_SLOT_NONE, DEFAULT_RENDER_LAYER_MASK, 303, 1],
            shadow_params: [0.75, 0.25, 0.5, 0.0],
            ..GpuLightData::default()
        }];

        assignments.apply_to_packed_lights(&lighting, &mut lights);

        assert_eq!(lights[0].shadow_slot_layer[0], 7);
        assert_eq!(lights[0].shadow_params[3], 1.0);
    }
}
