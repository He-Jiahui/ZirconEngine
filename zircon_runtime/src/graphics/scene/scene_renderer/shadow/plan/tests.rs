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
                transform: Transform::looking_at(Vec3::new(0.0, 0.0, 12.0), Vec3::ZERO, Vec3::Y),
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
        environment: crate::core::framework::render::EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let extract = RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(42), snapshot);
    ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240))
}

fn shadow(tier: ShadowResolutionTier) -> LightShadowSettings {
    shadow_with_quality(tier, ShadowPcfQuality::High)
}

fn default_light_layer_mask() -> RenderLayerSet {
    RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK)
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
