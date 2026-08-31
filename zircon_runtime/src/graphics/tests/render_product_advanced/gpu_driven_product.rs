use std::{path::PathBuf, sync::Arc};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ShaderAsset, ShaderSourceLanguage,
};
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderAmbientLightSnapshot,
    RenderCapabilitySummary, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderMeshStaticState, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    RendererCommon, ShaderAssetKind, ShaderFeatureBits, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Real, Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;
use crate::graphics::shader::standard_material_surface_source_for_features;

const GPU_DRIVEN_INSTANCE_COUNT: usize = 64;
const GPU_DRIVEN_VIEWPORT_SIZE: UVec2 = UVec2::new(320, 240);
const GPU_DRIVEN_MATERIAL_URI: &str = "res://materials/plan03-gpu-scene-multi-draw.zmaterial";
const GPU_DRIVEN_SHADER_URI: &str = "res://shaders/plan03-gpu-scene-multi-draw.zshader";
const GPU_DRIVEN_WGPU_PNG: &str = "plan03_gpu_scene_multi_draw_64_instances_wgpu_20260718.png";

#[test]
fn render_product_gpu_scene_multi_draw_64_instances_matches_cpu_fallback() {
    let (indirect_frame, indirect_stats) = render_gpu_driven_scene(Some(
        gpu_driven_product_capabilities("plan03-gpu-driven-indirect"),
    ));
    let (fallback_frame, fallback_stats) = render_gpu_driven_scene(Some(
        gpu_driven_product_fallback_capabilities("plan03-gpu-driven-fallback"),
    ));

    assert_gpu_driven_indirect_stats(&indirect_stats);
    assert_gpu_driven_fallback_stats(&fallback_stats);
    assert_visible_geometry(&indirect_frame);
    assert_eq!(
        captured_frame_hash(&indirect_frame),
        captured_frame_hash(&fallback_frame),
        "capability fallback must preserve the exact rendered product frame"
    );
}

#[test]
#[ignore = "writes Plan 03 WGPU framebuffer evidence under docs/tests/runtime/render"]
fn export_gpu_scene_multi_draw_64_instances_wgpu_png() {
    let (indirect_frame, indirect_stats) = render_gpu_driven_scene(None);
    assert!(
        indirect_stats
            .capabilities
            .gpu_driven_submission_supported(),
        "the real WGPU adapter must expose indirect draw, multi-draw indirect, and indirect first-instance; capabilities={:?}",
        indirect_stats.capabilities
    );
    assert_gpu_driven_indirect_stats(&indirect_stats);
    assert_visible_geometry(&indirect_frame);

    let mut fallback_capabilities = indirect_stats.capabilities.clone();
    fallback_capabilities.backend_name =
        format!("{}-plan03-cpu-fallback", fallback_capabilities.backend_name);
    fallback_capabilities.supports_multi_draw_indirect = false;
    let (fallback_frame, fallback_stats) = render_gpu_driven_scene(Some(fallback_capabilities));
    assert_gpu_driven_fallback_stats(&fallback_stats);
    assert_eq!(
        captured_frame_hash(&indirect_frame),
        captured_frame_hash(&fallback_frame),
        "real-adapter indirect submission and CPU fallback must produce identical pixels"
    );

    let output = repository_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
        .join(GPU_DRIVEN_WGPU_PNG);
    std::fs::create_dir_all(output.parent().expect("render evidence directory")).unwrap();
    image::save_buffer_with_format(
        &output,
        &indirect_frame.rgba,
        indirect_frame.width,
        indirect_frame.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
    assert!(
        output.is_file(),
        "missing visual evidence: {}",
        output.display()
    );
}

fn render_gpu_driven_scene(
    capabilities: Option<RenderCapabilitySummary>,
) -> (CapturedFrame, RenderStats) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material = register_gpu_driven_material(asset_manager.as_ref());
    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    if let Some(capabilities) = capabilities {
        framework.override_capabilities_for_tests(capabilities);
    }
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(GPU_DRIVEN_VIEWPORT_SIZE))
        .unwrap();
    framework
        .set_quality_profile(viewport, gpu_driven_quality_profile())
        .unwrap();
    framework
        .submit_frame_extract(viewport, gpu_driven_extract(material))
        .unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Plan 03 GPU-driven frame should be capturable");
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn gpu_driven_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("plan03-gpu-scene-multi-draw")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn gpu_driven_product_capabilities(backend_name: &str) -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: backend_name.to_string(),
        supports_offscreen: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        supports_buffer_readback: true,
        ..RenderCapabilitySummary::default()
    }
}

fn gpu_driven_product_fallback_capabilities(backend_name: &str) -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        supports_multi_draw_indirect: false,
        ..gpu_driven_product_capabilities(backend_name)
    }
}

fn gpu_driven_extract(material: ResourceHandle<MaterialMarker>) -> RenderFrameExtract {
    let meshes = (0..GPU_DRIVEN_INSTANCE_COUNT)
        .map(|index| gpu_driven_mesh(index, material))
        .collect();
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(3_003),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes,
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: vec![RenderAmbientLightSnapshot {
                    color: Vec3::new(0.78, 0.86, 1.0),
                    intensity: 1.4,
                    affects_lightmapped_meshes: true,
                    renderer_degraded: false,
                    degradation_reason: None,
                }],
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::new(0.006, 0.009, 0.016, 1.0),
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(GPU_DRIVEN_VIEWPORT_SIZE)
}

fn gpu_driven_mesh(index: usize, material: ResourceHandle<MaterialMarker>) -> RenderMeshSnapshot {
    let row = index / 8;
    let column = index % 8;
    let translation = Vec3::new(
        (column as Real - 3.5) * 0.62,
        (3.5 - row as Real) * 0.62,
        -6.0,
    );
    let node_id = 30_000 + index as u64;
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 1,
        transform: Transform {
            translation,
            scale: Vec3::splat(0.24),
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::new(
            0.12 + column as Real * 0.08,
            0.8,
            0.34 + row as Real * 0.06,
            1.0,
        ),
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::new(true, 1, 1),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    }
}

fn register_gpu_driven_material(
    asset_manager: &ProjectAssetManager,
) -> ResourceHandle<MaterialMarker> {
    let shader_uri = AssetUri::parse(GPU_DRIVEN_SHADER_URI).unwrap();
    let shader_id = ResourceId::from_locator(&shader_uri);
    let shader_source = gpu_driven_material_surface_source();
    let shader_hash = blake3::hash(shader_source.as_bytes()).to_hex().to_string();
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone())
                .with_source_hash(shader_hash.as_str()),
            ShaderAsset {
                uri: shader_uri.clone(),
                kind: ShaderAssetKind::Surface,
                source_language: ShaderSourceLanguage::Wgsl,
                source: shader_source,
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
                shading_model: Some("standard_pbr".to_string()),
                render_state: Default::default(),
                queue: None,
                disabled_passes: vec![
                    "depth_prepass".to_string(),
                    "shadow".to_string(),
                    "velocity".to_string(),
                    "taa_reactive_mask".to_string(),
                ],
                resources: Vec::new(),
                material_property_layout: Default::default(),
                material_option_table: Default::default(),
                generated_material_wgsl: String::new(),
                editor: Default::default(),
                pipeline_layout: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("Plan 03 GPU-driven shader insert");

    let material_uri = AssetUri::parse(GPU_DRIVEN_MATERIAL_URI).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri)
                .with_source_hash("plan03-gpu-scene-multi-draw-material-v1"),
            MaterialAsset {
                name: Some("Plan03GpuSceneMultiDraw".to_string()),
                shader: AssetReference::from_locator(shader_uri),
                parent: None,
                base_color: [0.18, 0.82, 0.42, 1.0],
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.05,
                roughness: 0.58,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.012, 0.028, 0.016],
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
                property_values: Default::default(),
                texture_slots: Default::default(),
                options: Default::default(),
                queue: None,
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("Plan 03 GPU-driven material insert");
    ResourceHandle::new(material_id)
}

fn gpu_driven_material_surface_source() -> String {
    let surface = standard_material_surface_source_for_features(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        0.0,
    );
    surface.source.replacen(
        &format!("fn {}(", surface.entry_point),
        "fn zr_material_surface(",
        1,
    )
}

fn assert_gpu_driven_indirect_stats(stats: &RenderStats) {
    assert!(stats.capabilities.gpu_driven_submission_supported());
    assert_eq!(
        stats.last_gpu_scene_instance_count,
        GPU_DRIVEN_INSTANCE_COUNT as u32
    );
    assert_eq!(stats.last_indirect_batch_count, 1);
    assert_eq!(
        stats.last_indirect_batched_draw_count,
        GPU_DRIVEN_INSTANCE_COUNT
    );
    assert_eq!(stats.last_indirect_args_count, GPU_DRIVEN_INSTANCE_COUNT);
    assert_eq!(stats.last_indirect_fallback_draw_count, 0);
}

fn assert_gpu_driven_fallback_stats(stats: &RenderStats) {
    assert!(!stats.capabilities.gpu_driven_submission_supported());
    assert_eq!(
        stats.last_gpu_scene_instance_count,
        GPU_DRIVEN_INSTANCE_COUNT as u32
    );
    assert_eq!(stats.last_indirect_batch_count, 0);
    assert_eq!(stats.last_indirect_batched_draw_count, 0);
    assert_eq!(stats.last_indirect_args_count, 0);
    assert_eq!(
        stats.last_indirect_fallback_draw_count,
        GPU_DRIVEN_INSTANCE_COUNT
    );
}

fn assert_visible_geometry(frame: &CapturedFrame) {
    let background = &frame.rgba[..3];
    let changed_pixel_count = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            pixel[..3]
                .iter()
                .zip(background)
                .any(|(channel, background)| channel.abs_diff(*background) > 12)
        })
        .count();
    assert!(
        changed_pixel_count > 2_000,
        "Plan 03 evidence must contain visible geometry, not only a clear; changed_pixels={changed_pixel_count}"
    );
}

fn captured_frame_hash(frame: &CapturedFrame) -> blake3::Hash {
    blake3::hash(&frame.rgba)
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should live below the repository root")
        .to_path_buf()
}
