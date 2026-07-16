use std::path::Path;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, ShaderAsset, ShaderSourceLanguage};
use crate::core::framework::render::{
    AntiAliasSettings, CameraRenderDescriptor, CapturedFrame, DisplayMode, GeometryExtract,
    LightShadowSettings, ProjectionMode, RenderCameraClear, RenderDirectionalLightSnapshot,
    RenderFrameExtract, RenderFramework, RenderLayerSet, RenderMaterialLightingModel,
    RenderMeshSnapshot, RenderQualityProfile, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ShaderAssetKind, ShadingModelDescriptor, ShadowPcfQuality,
    ShadowResolutionTier, ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
    ResourceState,
};
use crate::graphics::shader::ShaderVariantCacheDisk;
use crate::graphics::WgpuRenderFramework;

use super::super::super::render_product_submit::{
    material_with_import_note, snapshot_with_projection_for_mesh_cache_tests,
};
use super::super::register_material_asset_revision;
use super::case::RegistryShaderCase;
use super::manifest::raw_wgsl_hash;
use super::pipeline::registry_material_pass_product_pipeline;

pub(super) fn submit_registry_material_passes_with_staged_cache(
    case: RegistryShaderCase,
    shader_source: &str,
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
) -> RegistryMaterialPassLaunchStats {
    submit_registry_material_passes_with_shading_model(
        case,
        shader_source,
        world,
        runtime_root,
        staged_root,
        None,
        false,
    )
}

pub(super) fn submit_registry_material_passes_with_staged_cache_capture(
    case: RegistryShaderCase,
    shader_source: &str,
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
) -> RegistryMaterialPassLaunchStats {
    submit_registry_material_passes_with_shading_model(
        case,
        shader_source,
        world,
        runtime_root,
        staged_root,
        None,
        true,
    )
}

pub(super) fn submit_registry_material_passes_with_plugin_shading_model(
    case: RegistryShaderCase,
    shader_source: &str,
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
    plugin_shading_model: RegistryMaterialPassPluginShadingModel,
) -> RegistryMaterialPassLaunchStats {
    submit_registry_material_passes_with_shading_model(
        case,
        shader_source,
        world,
        runtime_root,
        staged_root,
        Some(plugin_shading_model),
        false,
    )
}

pub(super) fn submit_registry_material_passes_with_plugin_shading_model_capture(
    case: RegistryShaderCase,
    shader_source: &str,
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
    plugin_shading_model: RegistryMaterialPassPluginShadingModel,
) -> RegistryMaterialPassLaunchStats {
    submit_registry_material_passes_with_shading_model(
        case,
        shader_source,
        world,
        runtime_root,
        staged_root,
        Some(plugin_shading_model),
        true,
    )
}

fn submit_registry_material_passes_with_shading_model(
    case: RegistryShaderCase,
    shader_source: &str,
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
    plugin_shading_model: Option<RegistryMaterialPassPluginShadingModel>,
    capture_frames: bool,
) -> RegistryMaterialPassLaunchStats {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    register_registry_shader(
        &asset_manager,
        case,
        shader_source,
        plugin_shading_model
            .as_ref()
            .map(|model| model.descriptor.token.as_str()),
    );
    if let Some(plugin_shading_model) = &plugin_shading_model {
        (plugin_shading_model.register_shader_includes)(&asset_manager);
    }
    register_registry_taa_reactive_material(
        &asset_manager,
        case,
        plugin_shading_model
            .as_ref()
            .map(|model| model.material_lighting_model.clone()),
    );

    let framework = match plugin_shading_model {
        Some(plugin_shading_model) => {
            WgpuRenderFramework::new_for_test_with_plugin_render_extensions_and_shading_models(
                asset_manager,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                [plugin_shading_model.descriptor],
                Vec::new(),
                Vec::new(),
            )
        }
        None => WgpuRenderFramework::new_for_test(asset_manager),
    }
    .expect("WGPU framework");
    framework.replace_shader_variant_disk_cache_for_tests(
        ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root]),
    );
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(
            registry_material_pass_viewport_size(),
        ))
        .expect("viewport");
    let pipeline = framework
        .register_pipeline_asset(registry_material_pass_product_pipeline())
        .expect("project/plugin registry material-pass product pipeline");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("set registry material-pass product pipeline");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("project-plugin-registry-material-pass-staged-cache")
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(true)
                .with_temporal_history(true)
                .with_anti_alias(true),
        )
        .expect("quality profile");

    let first_extract = registry_material_pass_extract(case.material_id(), world, 0.0);
    framework
        .submit_frame_extract(viewport, first_extract)
        .expect("submit registry material-pass first frame");
    let first_frame = framework.query_stats().expect("first frame stats");
    let first_capture = if capture_frames {
        Some(
            framework
                .capture_frame(viewport)
                .expect("capture registry material-pass first frame")
                .expect("captured registry material-pass first frame"),
        )
    } else {
        None
    };

    let velocity_extract =
        registry_material_pass_extract(case.material_id(), world + 10_000, 0.125);
    framework
        .submit_frame_extract(viewport, velocity_extract)
        .expect("submit registry material-pass velocity frame");
    let velocity_frame = framework.query_stats().expect("velocity frame stats");
    let velocity_capture = if capture_frames {
        Some(
            framework
                .capture_frame(viewport)
                .expect("capture registry material-pass velocity frame")
                .expect("captured registry material-pass velocity frame"),
        )
    } else {
        None
    };

    RegistryMaterialPassLaunchStats {
        first_frame,
        velocity_frame,
        first_capture,
        velocity_capture,
    }
}

pub(super) struct RegistryMaterialPassPluginShadingModel {
    pub(super) descriptor: ShadingModelDescriptor,
    pub(super) material_lighting_model: RenderMaterialLightingModel,
    pub(super) register_shader_includes: fn(&ProjectAssetManager),
}

fn register_registry_shader(
    asset_manager: &ProjectAssetManager,
    case: RegistryShaderCase,
    source: &str,
    shading_model_token: Option<&str>,
) {
    let shader_uri = case.shader_uri();
    let shader_id = ResourceId::from_locator(&shader_uri);
    let source_hash = raw_wgsl_hash(source);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone())
                .with_source_hash(source_hash.as_str())
                .with_importer_id("zircon.plan08.registry-material-pass")
                .with_importer_version(1)
                .with_config_hash(source_hash.as_str()),
            ShaderAsset {
                uri: shader_uri.clone(),
                kind: ShaderAssetKind::Surface,
                source_language: ShaderSourceLanguage::Wgsl,
                source: source.to_string(),
                wgsl_source: String::new(),
                import_path: None,
                entry_points: Vec::new(),
                dependencies: Vec::new(),
                source_files: Vec::new(),
                imports: Vec::new(),
                shader_defs: Vec::new(),
                property_schema: Vec::new(),
                options: Vec::new(),
                texture_slots: Vec::new(),
                shading_model: Some(shading_model_token.unwrap_or("standard_pbr").to_string()),
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
            },
        )
        .expect("registry shader insert");

    let mut exported_record = ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri)
        .with_source_hash(source_hash.as_str())
        .with_importer_id("zircon.plan08.registry-material-pass")
        .with_importer_version(1)
        .with_config_hash(source_hash);
    exported_record.revision = case.revision;
    exported_record.state = ResourceState::Ready;
    asset_manager
        .resource_manager()
        .register_record(exported_record);
}

fn register_registry_taa_reactive_material(
    asset_manager: &ProjectAssetManager,
    case: RegistryShaderCase,
    lighting_model: Option<RenderMaterialLightingModel>,
) {
    let mut material = material_with_import_note();
    material.name = Some(format!("Plan08RegistryTaaMaterial{}", case.revision));
    material.shader = AssetReference::from_locator(case.shader_uri());
    if let Some(lighting_model) = lighting_model {
        material.property_values.insert(
            "lighting_model".to_string(),
            toml::Value::String(lighting_model.to_string()),
        );
    }
    material.validation_diagnostics.clear();
    material.property_values.insert(
        "taa_reactive_mask_strength".to_string(),
        toml::Value::Float(1.0),
    );
    register_material_asset_revision(
        asset_manager,
        case.material_id(),
        case.material_uri(),
        "project-plugin-registry-material-pass-v1",
        material,
    );
}

fn registry_material_pass_extract(
    material_id: ResourceId,
    world: u64,
    x_offset: f32,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    select_visible_registry_material_pass_camera(&mut extract);
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![registry_material_pass_mesh(material_id, x_offset)],
    );
    extract.lighting.directional_lights = vec![registry_material_pass_shadow_light()];
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &extract.view.anti_alias);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    extract
}

fn select_visible_registry_material_pass_camera(extract: &mut RenderFrameExtract) {
    let mut camera = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Perspective,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(registry_material_pass_viewport_size());
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(4_200), camera);
    descriptor.clear = RenderCameraClear::Color(Vec4::ZERO);
    extract.view.select_camera_descriptor(descriptor);
}

fn registry_material_pass_viewport_size() -> UVec2 {
    UVec2::new(320, 240)
}

fn registry_material_pass_mesh(material_id: ResourceId, x_offset: f32) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 4_203,
        stable_instance_key: 4_203 << 16,
        transform_revision: 1,
        transform: Transform {
            translation: Vec3::new(x_offset, 0.0, 0.0),
            scale: Vec3::splat(0.8),
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
    }
}

fn registry_material_pass_shadow_light() -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: 4_290,
        light_id: 4_290,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        direction: Vec3::new(0.35, -0.25, -1.0).normalize(),
        color: Vec3::ONE,
        intensity: 1.0,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: Some(LightShadowSettings {
            casts_shadow: true,
            depth_bias: 0.0,
            normal_bias: 0.0,
            strength: 1.0,
            resolution_preference: ShadowResolutionTier::T512,
            pcf_quality: ShadowPcfQuality::Medium,
        }),
    }
}

pub(super) struct RegistryMaterialPassLaunchStats {
    pub(super) first_frame: RenderStats,
    pub(super) velocity_frame: RenderStats,
    pub(super) first_capture: Option<CapturedFrame>,
    pub(super) velocity_capture: Option<CapturedFrame>,
}
