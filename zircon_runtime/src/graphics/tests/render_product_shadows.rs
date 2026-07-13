use crate::core::framework::render::{
    FallbackSkyboxKind, LightShadowSettings, LightingExtract, PostProcessGraphResourceNames,
    PreviewEnvironmentExtract, RenderDirectionalLightSnapshot, RenderFrameExtract, RenderLayerSet,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpotLightSnapshot,
    RenderWorldSnapshotHandle, ShadowPcfQuality, ShadowResolutionTier, ViewportCameraSnapshot,
    DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::graphics::scene::{
    build_shadow_frame_plan, ShadowAtlasAllocator, ShadowAtlasResourceConfig,
    ShadowLightSlotAssignment,
};
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::VisibilityViewKey;
use crate::graphics::RenderPipelineAsset;
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

mod many_point_lights;

#[test]
fn shadow_atlas_pass_stays_live_as_depth_only_graph_contract() {
    for compiled in [
        RenderPipelineAsset::default_forward_plus()
            .compile(&test_extract())
            .unwrap(),
        RenderPipelineAsset::default_deferred()
            .compile(&test_extract())
            .unwrap(),
    ] {
        let shadow_pass = compiled
            .graph
            .passes()
            .iter()
            .find(|pass| pass.name == "shadow-atlas")
            .expect("default Core3d pipelines should compile a shadow-atlas pass");

        assert!(
            !shadow_pass.culled,
            "shadow-atlas is a side-effectful renderer slot and must stay visible until concrete shadow consumers sample it"
        );
        assert!(shadow_pass.flags.has_side_effects);
        assert_eq!(shadow_pass.executor_id.as_deref(), Some("shadow.atlas"));
        assert_eq!(shadow_pass.queue, QueueLane::Graphics);

        let atlas_write = pass_resource_access(
            &compiled,
            "shadow-atlas",
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            RenderGraphResourceAccessKind::Write,
        );
        assert_eq!(
            atlas_write.kind,
            RenderGraphResourceKind::External,
            "shadow-atlas pass should publish the persistent atlas as a graph-visible external resource"
        );
        assert_eq!(
            atlas_write.attachment_ops,
            Some(RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Clear,
                store: RenderGraphAttachmentStoreOp::Store,
            }),
            "shadow-atlas pass should clear/store the persistent atlas depth target"
        );
        assert!(
            compiled
                .graph
                .passes()
                .iter()
                .flat_map(|pass| pass.resources.iter())
                .all(|resource| resource.name != "shadow-map"),
            "default graph should not retain the legacy shadow-map receiver resource"
        );
    }
}

#[test]
fn deferred_lighting_reads_shadow_atlas_for_receiver_sampling() {
    let compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();

    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::SHADOW_ATLAS,
        RenderGraphResourceAccessKind::Read,
    );
}

#[test]
fn forward_mesh_passes_read_shadow_atlas_for_receiver_sampling() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    for pass_name in ["opaque-mesh", "alpha-mask-mesh", "transparent-mesh"] {
        pass_resource_access(
            &compiled,
            pass_name,
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            RenderGraphResourceAccessKind::Read,
        );
    }
}

#[test]
fn deferred_transparent_mesh_reads_shadow_atlas_for_receiver_sampling() {
    let compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();

    pass_resource_access(
        &compiled,
        "transparent-mesh",
        PostProcessGraphResourceNames::SHADOW_ATLAS,
        RenderGraphResourceAccessKind::Read,
    );
}

#[test]
fn render_product_csm_directional() {
    let lighting = LightingExtract {
        directional_lights: vec![RenderDirectionalLightSnapshot {
            node_id: 901,
            light_id: 9_001,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            direction: Vec3::new(0.0, -1.0, -1.0),
            color: Vec3::ONE,
            intensity: 2.0,
            mobility: crate::core::framework::scene::Mobility::Dynamic,
            shadow: Some(shadow_settings(ShadowResolutionTier::T1024)),
        }],
        ..LightingExtract::default()
    };
    let mut allocator = ShadowAtlasAllocator::default();
    let plan = build_shadow_frame_plan(
        &mut allocator,
        &shadow_frame(lighting),
        ShadowAtlasResourceConfig::new(4096, 4096, 16),
    );

    assert_eq!(plan.slots().len(), 4);
    assert_eq!(plan.atlas_passes().len(), 4);
    assert_eq!(
        plan.light_slots().get(9_001),
        Some(ShadowLightSlotAssignment {
            first_slot: 0,
            slot_count: 4,
        })
    );
    assert!(plan.globals().cascade_splits[0] > 0.0);
    assert_ne!(
        plan.atlas_passes()[0].view_proj,
        plan.atlas_passes()[3].view_proj
    );
    assert_eq!(
        plan.atlas_passes()
            .iter()
            .map(|slot_pass| slot_pass.view_key)
            .collect::<Vec<_>>(),
        (0..4)
            .map(|cascade| Some(VisibilityViewKey::ShadowCascade {
                light: 901,
                cascade,
            }))
            .collect::<Vec<_>>()
    );
}

#[test]
fn render_product_multi_spot_shadows() {
    let lighting = LightingExtract {
        spot_lights: (0..3).map(product_spot_light).collect(),
        ..LightingExtract::default()
    };
    let mut allocator = ShadowAtlasAllocator::default();
    let plan = build_shadow_frame_plan(
        &mut allocator,
        &shadow_frame(lighting),
        ShadowAtlasResourceConfig::new(4096, 4096, 16),
    );

    assert_eq!(plan.slots().len(), 3);
    assert_eq!(plan.atlas_passes().len(), 3);
    for index in 0..3 {
        let light_id = 10_000 + index as u64;
        assert_eq!(
            plan.light_slots().get(light_id),
            Some(ShadowLightSlotAssignment {
                first_slot: index,
                slot_count: 1,
            })
        );
        assert_eq!(
            plan.atlas_passes()[index as usize].view_key,
            Some(VisibilityViewKey::ShadowSpot {
                light: 1_000 + index as u64,
            })
        );
    }
    for left in 0..plan.atlas_passes().len() {
        for right in (left + 1)..plan.atlas_passes().len() {
            assert!(
                !plan.atlas_passes()[left]
                    .rect
                    .intersects(plan.atlas_passes()[right].rect),
                "spot shadow atlas slots should not overlap"
            );
        }
    }
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn shadow_frame(lighting: LightingExtract) -> ViewportRenderFrame {
    ViewportRenderFrame::from_extract(
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(9_000),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot {
                        transform: Transform::looking_at(
                            Vec3::new(0.0, 1.0, 10.0),
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
                overlays: Default::default(),
                environment: crate::core::framework::render::EnvironmentExtract::default(),
                preview: PreviewEnvironmentExtract {
                    lighting_enabled: true,
                    skybox_enabled: false,
                    fallback_skybox: FallbackSkyboxKind::None,
                    clear_color: Vec4::ZERO,
                },
                virtual_geometry_debug: None,
            },
        ),
        UVec2::new(320, 240),
    )
}

fn shadow_settings(resolution_preference: ShadowResolutionTier) -> LightShadowSettings {
    LightShadowSettings {
        casts_shadow: true,
        depth_bias: 0.25,
        normal_bias: 0.5,
        strength: 0.75,
        resolution_preference,
        pcf_quality: ShadowPcfQuality::High,
    }
}

fn product_spot_light(index: u64) -> RenderSpotLightSnapshot {
    RenderSpotLightSnapshot {
        node_id: 1_000 + index,
        light_id: 10_000 + index,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(index as f32 * 2.0 - 2.0, 4.0, 1.0),
        direction: Vec3::new(0.0, -1.0, -0.25),
        color: Vec3::ONE,
        intensity: 3.0,
        range: 12.0,
        inner_angle_radians: 0.25,
        outer_angle_radians: 0.5,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: Some(shadow_settings(ShadowResolutionTier::T512)),
    }
}

fn pass_resource_access<'a>(
    compiled: &'a crate::graphics::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> &'a crate::render_graph::RenderGraphPassResourceAccess {
    compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .and_then(|pass| {
            pass.resources
                .iter()
                .find(|resource| resource.name == resource_name && resource.access == access)
        })
        .unwrap_or_else(|| panic!("pass `{pass_name}` should {access:?} `{resource_name}`"))
}
