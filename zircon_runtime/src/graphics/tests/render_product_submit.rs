use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AdvancedProviderStatus, AdvancedRenderFeature, CorePipelineKind, DisplayMode,
    FallbackSkyboxKind, GeometryExtract, PreviewEnvironmentExtract, ProjectionMode,
    RenderAmbientLightSnapshot, RenderFrameExtract, RenderFramework, RenderMaterialAlphaMode,
    RenderMeshSnapshot, RenderOverlayExtract, RenderPhase, RenderPipelineHandle,
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderQualityProfile,
    RenderRectLightSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpriteAnchor,
    RenderSpriteSnapshot, RenderViewportDescriptor, RenderVirtualGeometryPayloadSource,
    RenderWorldSnapshotHandle, SolariRuntimeStatus, SpriteExtract, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, TextureMarker,
};
use crate::graphics::{ViewportRenderFrame, WgpuRenderFramework};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap, UiVisualAssetRef,
};

use super::plugin_render_feature_fixtures::{
    pluginized_wgpu_render_framework_with_advanced_providers,
    pluginized_wgpu_render_framework_with_solari_provider,
};

#[test]
fn render_product_submit_direct_extract_frame_does_not_use_legacy_scene_snapshot_authority() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(88),
        snapshot_with_projection(ProjectionMode::Orthographic),
    );

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

    assert_eq!(frame.camera().projection_mode, ProjectionMode::Orthographic);
    assert_eq!(
        frame.scene.scene.camera.projection_mode,
        ProjectionMode::Perspective,
        "legacy scene snapshot must not be the product submit draw authority"
    );
    assert_eq!(frame.overlays().display_mode, DisplayMode::WireOnly);
    assert_eq!(frame.scene.overlays.display_mode, DisplayMode::Shaded);
}

#[test]
fn render_product_submit_unknown_viewport_returns_error_without_panic() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework.destroy_viewport(viewport).unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(89),
                snapshot_with_projection(ProjectionMode::Perspective),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        crate::core::framework::render::RenderFrameworkError::UnknownViewport { viewport: 1 }
    ));
}

#[test]
fn render_product_submit_selects_default_pipeline_from_extract_core_pipeline() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(90),
                snapshot_with_projection(ProjectionMode::Orthographic),
            ),
        )
        .unwrap();
    assert_eq!(
        framework.query_stats().unwrap().last_pipeline,
        Some(RenderPipelineHandle::new(3))
    );

    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(91),
                snapshot_with_projection(ProjectionMode::Perspective),
            ),
        )
        .unwrap();
    assert_eq!(
        framework.query_stats().unwrap().last_pipeline,
        Some(RenderPipelineHandle::new(1))
    );
}

#[test]
fn render_product_submit_preserves_quality_profile_pipeline_override() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("explicit-forward")
                .with_pipeline_asset(RenderPipelineHandle::new(1)),
        )
        .unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(92),
                snapshot_with_projection(ProjectionMode::Orthographic),
            ),
        )
        .unwrap_err();

    assert!(
        matches!(error, crate::core::framework::render::RenderFrameworkError::Backend(ref message) if message.contains("core pipeline mismatch")),
        "unexpected error: {error:?}"
    );
}

#[test]
fn render_product_pbr_submit_reports_material_fallback_and_light_stats() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("pbr-material-light-stats")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(93),
        snapshot_with_projection(ProjectionMode::Perspective),
    );
    extract.geometry = crate::core::framework::render::GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![pbr_mesh_with_missing_material()],
    );
    extract
        .lighting
        .ambient_lights
        .push(RenderAmbientLightSnapshot {
            color: Vec3::new(0.04, 0.05, 0.06),
            intensity: 0.25,
            renderer_degraded: true,
            degradation_reason: Some(
                "ambient light renderer path is deferred after M5A".to_string(),
            ),
        });
    extract.lighting.rect_lights.push(RenderRectLightSnapshot {
        node_id: 700,
        position: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 4.0,
        size: Vec2::new(2.0, 0.5),
        renderer_degraded: true,
        degradation_reason: Some("rect light renderer path is deferred after M5A".to_string()),
    });

    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_material_count, 1);
    assert_eq!(stats.last_material_ready_count, 0);
    assert_eq!(stats.last_material_fallback_count, 1);
    assert_eq!(stats.last_material_validation_error_count, 1);
    assert_eq!(stats.last_ambient_light_count, 1);
    assert_eq!(stats.last_rect_light_count, 1);
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
}

#[test]
fn render_product_submit_default_profile_accepts_default_3d_ui_and_2d_sprite_paths() {
    let bundle = RenderProfileBundle::default_render();
    bundle.validate().unwrap();
    assert!(bundle.enables(RenderProductProfile::DefaultRender));
    assert!(bundle.has_feature(RenderProductFeature::Mesh));
    assert!(bundle.has_feature(RenderProductFeature::Material));
    assert!(bundle.has_feature(RenderProductFeature::Sprite));
    assert!(bundle.enables(RenderProductProfile::Ui));
    assert!(bundle.has_feature(RenderProductFeature::UiRender));
    assert!(bundle.has_feature(RenderProductFeature::PostProcess));
    assert!(bundle.has_feature(RenderProductFeature::AntiAlias));
    assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
    assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    assert!(!bundle.has_feature(RenderProductFeature::Solari));

    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    framework
        .submit_frame_extract_with_ui(
            viewport,
            default_core3d_acceptance_extract(),
            Some(runtime_ui_acceptance_extract()),
        )
        .unwrap();
    let core3d_stats = framework.query_stats().unwrap();
    assert!(core3d_stats.last_material_count > 0);
    assert!(
        core3d_stats.last_directional_light_count
            + core3d_stats.last_point_light_count
            + core3d_stats.last_spot_light_count
            > 0
    );
    assert!(core3d_stats.last_post_process_graph_node_count > 0);
    assert_eq!(core3d_stats.last_anti_alias_graph_executed_pass_count, 1);
    assert_eq!(core3d_stats.last_ui_graph_executed_pass_count, 1);
    assert_eq!(
        core3d_stats.last_virtual_geometry_graph_executed_pass_count,
        0
    );
    assert_eq!(core3d_stats.last_hybrid_gi_graph_executed_pass_count, 0);
    assert_eq!(
        core3d_stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::NotRequested
    );

    framework
        .submit_frame_extract(viewport, default_core2d_sprite_acceptance_extract())
        .unwrap();
    let core2d_stats = framework.query_stats().unwrap();
    assert_eq!(core2d_stats.last_sprite_count, 1);
    assert_eq!(core2d_stats.last_sprite_texture_fallback_count, 1);
    assert_eq!(core2d_stats.last_sprite_graph_executed_pass_count, 3);
    assert_eq!(
        core2d_stats.last_virtual_geometry_graph_executed_pass_count,
        0
    );
    assert_eq!(core2d_stats.last_hybrid_gi_graph_executed_pass_count, 0);
}

#[test]
fn render_product_submit_headless_profile_has_no_render_product_activation() {
    let bundle = RenderProfileBundle::headless();

    bundle.validate().unwrap();
    bundle
        .validate_capabilities(&Default::default())
        .expect("headless render profile should not require backend rendering caps");
    assert_eq!(bundle.profile(), RenderProductProfile::Headless);
    assert!(bundle.features().is_empty());
    assert!(!bundle.enables(RenderProductProfile::DefaultRender));
    assert!(!bundle.has_feature(RenderProductFeature::Mesh));
    assert!(!bundle.has_feature(RenderProductFeature::UiRender));
    assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
    assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    assert!(!bundle.has_feature(RenderProductFeature::Solari));
}

#[test]
fn render_product_submit_advanced_profile_accepts_provider_backed_vg_hgi_path() {
    let bundle = RenderProfileBundle::advanced_render();
    assert!(bundle.enables(RenderProductProfile::AdvancedRender));
    assert!(bundle.has_feature(RenderProductFeature::VirtualGeometry));
    assert!(bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    assert!(!bundle.has_feature(RenderProductFeature::Solari));

    let framework = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            super::render_product_advanced::advanced_quality_profile("m10a-advanced-acceptance"),
        )
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            super::render_product_advanced::advanced_product_extract(),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 5);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert_eq!(
        super::render_product_advanced::advanced_provider_report(
            &stats,
            AdvancedRenderFeature::VirtualGeometry,
        )
        .status,
        AdvancedProviderStatus::Ready
    );
    assert_eq!(
        super::render_product_advanced::advanced_provider_report(
            &stats,
            AdvancedRenderFeature::HybridGlobalIllumination,
        )
        .status,
        AdvancedProviderStatus::Ready
    );
}

#[test]
fn render_product_submit_solari_experimental_reports_gated_provider_status() {
    let bundle = RenderProfileBundle::solari_experimental();
    assert!(bundle.enables(RenderProductProfile::SolariExperimental));
    assert!(bundle.has_feature(RenderProductFeature::Solari));

    let framework =
        pluginized_wgpu_render_framework_with_solari_provider(SolariRuntimeStatus::Unavailable);
    framework.override_capabilities_for_tests(super::render_product_solari::solari_capabilities());
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            super::render_product_solari::solari_quality_profile("m10a-solari-acceptance", true)
                .with_virtual_geometry(false)
                .with_hybrid_global_illumination(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, default_core3d_acceptance_extract())
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(
        stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::Unavailable
    );
    assert_eq!(
        stats.last_solari_runtime_report.provider_id.as_deref(),
        Some("test.solari")
    );
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
}

pub(super) fn snapshot_with_projection_for_sprite_tests(
    projection_mode: ProjectionMode,
) -> RenderSceneSnapshot {
    snapshot_with_projection(projection_mode)
}

fn snapshot_with_projection(projection_mode: ProjectionMode) -> RenderSceneSnapshot {
    let camera = ViewportCameraSnapshot {
        projection_mode,
        ..ViewportCameraSnapshot::default()
    };
    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::WireOnly,
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

fn pbr_mesh_with_missing_material() -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 600,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/not-registered",
        )),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        render_layer_mask: u32::MAX,
    }
}

fn default_core3d_acceptance_extract() -> RenderFrameExtract {
    let mut extract = crate::scene::world::World::new().to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract
}

fn default_core2d_sprite_acceptance_extract() -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(94),
        snapshot_with_projection(ProjectionMode::Orthographic),
    );
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract.geometry = GeometryExtract::from_meshes(CorePipelineKind::Core2d, Vec::new());
    extract.sprites = SpriteExtract::from_sprites(
        CorePipelineKind::Core2d,
        vec![RenderSpriteSnapshot {
            entity: 501,
            transform: Transform::default(),
            image: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
                "res://textures/m10a-default-sprite.png",
            )),
            material: None,
            atlas_region: None,
            rect: None,
            flip_x: false,
            flip_y: false,
            anchor: RenderSpriteAnchor::CENTER,
            custom_size: Some(Vec2::new(1.0, 1.0)),
            color: Vec4::ONE,
            z_order: 0,
            render_layer_mask: u32::MAX,
            material_alpha_mode: RenderMaterialAlphaMode::Blend,
        }],
    );
    assert_eq!(
        extract
            .sprites
            .phase_queue
            .items_for_phase(RenderPhase::Transparent2d)
            .count(),
        1
    );
    extract
}

fn runtime_ui_acceptance_extract() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.m10a.acceptance"),
        list: UiRenderList {
            commands: vec![
                UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 8.0, 180.0, 28.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#111827cc".to_string()),
                        foreground_color: Some("#f9fafb".to_string()),
                        font_size: 14.0,
                        line_height: 18.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::None,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Default HUD".to_string()),
                    image: None,
                    opacity: 1.0,
                },
                UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Image,
                    frame: UiFrame::new(20.0, 48.0, 32.0, 32.0),
                    clip_frame: Some(UiFrame::new(16.0, 44.0, 40.0, 40.0)),
                    z_index: 1,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: Some(UiVisualAssetRef::Image(
                        "res://ui/runtime/m10a-hud-icon.png".to_string(),
                    )),
                    opacity: 1.0,
                },
            ],
        },
    }
}
