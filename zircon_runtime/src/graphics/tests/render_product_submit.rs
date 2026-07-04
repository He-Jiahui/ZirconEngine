use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ShaderAsset, ShaderMaterialPropertyAsset,
    ShaderSourceLanguage,
};
use crate::core::framework::render::{
    AdvancedProviderStatus, AdvancedRenderFeature, CorePipelineKind, DisplayMode,
    FallbackSkyboxKind, GeometryExtract, MaterialPropertyKind, PreviewEnvironmentExtract,
    ProjectionMode, RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMaterialAlphaMode, RenderMeshSnapshot,
    RenderOverlayExtract, RenderPhase, RenderPipelineHandle, RenderPointLightSnapshot,
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderQualityProfile,
    RenderRectLightSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderSpotLightSnapshot, RenderSpriteAnchor, RenderSpriteImageMode, RenderSpriteSnapshot,
    RenderViewportDescriptor, RenderVirtualGeometryPayloadSource, RenderWorldSnapshotHandle,
    ShaderAssetKind, SolariRuntimeStatus, SpriteExtract, ViewportCameraSnapshot,
    DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
    TextureMarker,
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

mod profiles;

#[test]
fn render_product_submit_direct_extract_frame_does_not_use_legacy_scene_snapshot_authority() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(88),
        snapshot_with_projection(ProjectionMode::Orthographic),
    );

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

    assert_eq!(
        frame.effective_camera().projection_mode,
        ProjectionMode::Orthographic
    );
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
            renderer_degraded: false,
            degradation_reason: None,
        });
    extract.lighting.directional_lights.extend([
        RenderDirectionalLightSnapshot {
            node_id: 701,
            light_id: 701,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 4.0,
            shadow: None,
        },
        RenderDirectionalLightSnapshot {
            node_id: 702,
            light_id: 702,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            direction: Vec3::new(1.0, -1.0, 0.0),
            color: Vec3::new(0.6, 0.7, 1.0),
            intensity: 2.0,
            shadow: None,
        },
    ]);
    extract
        .lighting
        .point_lights
        .push(RenderPointLightSnapshot {
            node_id: 703,
            light_id: 703,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            position: Vec3::new(-1.0, 1.0, 0.5),
            color: Vec3::new(0.9, 0.8, 1.0),
            intensity: 6.0,
            range: 8.0,
            shadow: None,
        });
    extract.lighting.spot_lights.push(RenderSpotLightSnapshot {
        node_id: 704,
        light_id: 704,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(0.0, 3.0, 2.0),
        direction: Vec3::new(0.0, -1.0, -0.5),
        color: Vec3::new(1.0, 0.95, 0.75),
        intensity: 5.0,
        range: 10.0,
        inner_angle_radians: 0.35,
        outer_angle_radians: 0.75,
        shadow: None,
    });
    extract.lighting.rect_lights.push(RenderRectLightSnapshot {
        node_id: 700,
        light_id: 700,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 4.0,
        range: 12.0,
        size: Vec2::new(2.0, 0.5),
        shadow: None,
        renderer_degraded: true,
        degradation_reason: Some("rect light renderer path is deferred after M5A".to_string()),
    });

    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_material_count, 1);
    assert_eq!(stats.last_material_ready_count, 0);
    assert_eq!(stats.last_material_fallback_count, 1);
    assert_eq!(stats.last_material_validation_error_count, 1);
    assert_eq!(stats.last_directional_light_count, 2);
    assert_eq!(stats.last_directional_light_ready_count, 2);
    assert_eq!(stats.last_directional_light_degraded_count, 0);
    assert_eq!(stats.last_point_light_count, 1);
    assert_eq!(stats.last_point_light_ready_count, 1);
    assert_eq!(stats.last_point_light_degraded_count, 0);
    assert_eq!(stats.last_spot_light_count, 1);
    assert_eq!(stats.last_spot_light_ready_count, 1);
    assert_eq!(stats.last_spot_light_degraded_count, 0);
    assert_eq!(stats.last_ambient_light_count, 1);
    assert_eq!(stats.last_ambient_light_ready_count, 1);
    assert_eq!(stats.last_ambient_light_degraded_count, 0);
    assert_eq!(stats.last_rect_light_count, 1);
    assert_eq!(stats.last_rect_light_ready_count, 0);
    assert_eq!(stats.last_rect_light_degraded_count, 1);
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
}

#[test]
fn render_product_submit_material_stats_count_non_blocking_diagnostics() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/import-note.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_import_note(),
        )
        .expect("material insert");

    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(95),
        snapshot_with_projection(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![RenderMeshSnapshot {
            node_id: 601,
            stable_instance_key: 601 << 16,
            transform_revision: 0,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "builtin://cube",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(material_id),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        }],
    );

    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_material_count, 1);
    assert_eq!(stats.last_material_ready_count, 1);
    assert_eq!(stats.last_material_fallback_count, 0);
    assert_eq!(stats.last_material_validation_error_count, 0);
    assert_eq!(stats.last_material_diagnostic_count, 1);
}

#[test]
fn render_product_submit_material_stats_count_material_uniform_diagnostics() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let shader_uri = AssetUri::parse("res://shaders/uniform-string.zshader").unwrap();
    let shader_id = ResourceId::from_locator(&shader_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri),
            shader_with_string_property(),
        )
        .expect("shader insert");

    let material_uri = AssetUri::parse("res://materials/uniform-string.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_string_property(),
        )
        .expect("material insert");

    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(96),
        snapshot_with_projection(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![RenderMeshSnapshot {
            node_id: 602,
            stable_instance_key: 602 << 16,
            transform_revision: 0,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "builtin://cube",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(material_id),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        }],
    );

    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_material_count, 1);
    assert_eq!(stats.last_material_ready_count, 1);
    assert_eq!(stats.last_material_fallback_count, 0);
    assert_eq!(stats.last_material_validation_error_count, 0);
    assert_eq!(stats.last_material_diagnostic_count, 1);
}

pub(super) fn snapshot_with_projection_for_sprite_tests(
    projection_mode: ProjectionMode,
) -> RenderSceneSnapshot {
    snapshot_with_projection(projection_mode)
}

pub(super) fn snapshot_with_projection_for_mesh_cache_tests(
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
        environment: crate::core::framework::render::EnvironmentExtract::default(),
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
        stable_instance_key: 600 << 16,
        transform_revision: 0,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/not-registered",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

pub(super) fn material_with_import_note() -> MaterialAsset {
    MaterialAsset {
        name: Some("ImportNote".to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: vec![
            "glTF material imported with generated renderer defaults".to_string()
        ],
    }
}

fn shader_with_string_property() -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/uniform-string.zshader").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: vec![ShaderMaterialPropertyAsset {
            name: "debug_label".to_string(),
            kind: MaterialPropertyKind::Bool,
            required: false,
            default: None,
            editor: Default::default(),
        }],
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn material_with_string_property() -> MaterialAsset {
    let mut material = material_with_import_note();
    material.name = Some("UniformString".to_string());
    material.shader = AssetReference::from_locator(
        AssetUri::parse("res://shaders/uniform-string.zshader").unwrap(),
    );
    material.validation_diagnostics.clear();
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("paint".to_string()),
    );
    material
}
