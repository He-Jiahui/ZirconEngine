use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::assets::{AlphaMode, MaterialAsset, ModelAsset, ModelPrimitiveAsset};
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, AssetUri, MeshVertex};
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, DisplayMode, EnvironmentExtract, GBufferChannelMask,
    PreviewEnvironmentExtract, ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState,
    RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle, ShadingModelDescriptor,
    ShadingModelId, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::{
    subsurface_render_feature_descriptor, subsurface_render_pass_executor_registrations,
    WgpuRenderFramework, SSS_RECOMBINE_EXECUTOR_ID, SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID,
};

const PRODUCT_SIZE: UVec2 = UVec2::new(640, 360);
const DEFERRED_PIPELINE_HANDLE: RenderPipelineHandle = RenderPipelineHandle::new(2);
const PRODUCT_IMAGE_NAME: &str = "plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.png";
const PRODUCT_REPORT_NAME: &str = "plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.txt";
const SSS_SHADING_MODEL_ID: ShadingModelId = ShadingModelId::new(16);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct FrameDifference {
    changed_pixel_count: u64,
    brightened_pixel_count: u64,
    darkened_pixel_count: u64,
    red_dominant_gain_count: u64,
    mean_absolute_rgb_error: f64,
    max_rgb_error: u8,
}

struct ProductRender {
    frame: CapturedFrame,
    stats: RenderStats,
    submit_cpu_micros: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProductSssMode {
    FeatureOff,
    RegisteredWithoutProfile,
    Enabled,
}

#[test]
fn render_product_sss_registered_but_without_profiles_is_exact_deferred_baseline() {
    let baseline = render_skin_sphere(ProductSssMode::FeatureOff);
    let registered = render_skin_sphere(ProductSssMode::RegisteredWithoutProfile);

    assert_eq!(baseline.frame.rgba, registered.frame.rgba);
    assert!(!registered
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| {
            matches!(
                pass.as_str(),
                SSS_SETUP_EXECUTOR_ID | SSS_SCATTER_EXECUTOR_ID | SSS_RECOMBINE_EXECUTOR_ID
            )
        }));
}

#[test]
fn render_product_sss_skin_sphere_executes_burley_indirect_pipeline() {
    let baseline = render_skin_sphere(ProductSssMode::FeatureOff);
    let subsurface = render_skin_sphere(ProductSssMode::Enabled);
    let difference = frame_difference(&baseline.frame, &subsurface.frame);

    for pass in [
        SSS_SETUP_EXECUTOR_ID,
        SSS_SCATTER_EXECUTOR_ID,
        SSS_RECOMBINE_EXECUTOR_ID,
    ] {
        assert!(
            subsurface
                .stats
                .last_graph_executed_passes
                .iter()
                .any(|executed| executed == pass),
            "SSS product graph did not execute {pass}: {:?}",
            subsurface.stats.last_graph_executed_passes
        );
    }
    assert!(
        difference.changed_pixel_count > 2_000,
        "skin sphere should expose broad SSS coverage: {difference:?}"
    );
    assert!(
        difference.mean_absolute_rgb_error > 0.04,
        "Burley scattering should be visibly distinct from deferred PBR: {difference:?}"
    );
    assert!(
        difference.brightened_pixel_count > 500
            && difference.darkened_pixel_count > 500,
        "SSS must redistribute diffuse light instead of applying a uniform brightness change: {difference:?}"
    );
    assert!(
        difference.red_dominant_gain_count > 200,
        "skin profile must produce measurable long-radius red-channel bleed: {difference:?}"
    );
    assert_eq!(
        subsurface.stats.last_graph_compute_workload_mismatch_count, 0,
        "SSS setup and GPU-indirect scatter must both satisfy workload audit"
    );
}

#[test]
#[ignore = "exports the plan-18 SSS skin-sphere WGPU product comparison and timing evidence"]
fn export_render_product_sss_skin_sphere_png() {
    let baseline = render_skin_sphere(ProductSssMode::FeatureOff);
    let subsurface = render_skin_sphere(ProductSssMode::Enabled);
    let difference = frame_difference(&baseline.frame, &subsurface.frame);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");

    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_side_by_side_png(&image_path, &baseline.frame, &subsurface.frame);
    let report_path = output_dir.join(PRODUCT_REPORT_NAME);
    fs::write(
        &report_path,
        product_report(&baseline, &subsurface, difference, &image_path),
    )
    .expect("SSS product report should be writable");

    assert!(image_path.is_file());
    assert!(report_path.is_file());
}

fn render_skin_sphere(mode: ProductSssMode) -> ProductRender {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = register_skin_sphere_model(&asset_manager);
    let registered_sss = mode != ProductSssMode::FeatureOff;
    let material = register_skin_material(
        &asset_manager,
        registered_sss,
        mode == ProductSssMode::Enabled,
    );
    let render_features = registered_sss
        .then(subsurface_render_feature_descriptor)
        .into_iter()
        .collect::<Vec<_>>();
    let executors = if registered_sss {
        subsurface_render_pass_executor_registrations()
    } else {
        Vec::new()
    };
    let framework =
        WgpuRenderFramework::new_for_test_with_plugin_render_extensions_and_shading_models(
            asset_manager,
            render_features,
            executors,
            Vec::new(),
            Vec::new(),
            registered_sss
                .then(sss_shading_model_descriptor)
                .into_iter(),
            Vec::new(),
            Vec::new(),
        )
        .expect("WGPU framework should initialize for the SSS product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_SIZE))
        .expect("SSS product viewport should be created");
    framework
        .set_pipeline_asset(viewport, DEFERRED_PIPELINE_HANDLE)
        .expect("SSS product must use the deferred pipeline");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("SSS product quality profile should be accepted");

    let start = Instant::now();
    framework
        .submit_frame_extract(viewport, skin_sphere_extract(model, material))
        .expect("SSS product frame should submit");
    let submit_cpu_micros = start.elapsed().as_micros();
    let frame = framework
        .capture_frame(viewport)
        .expect("SSS product capture should succeed")
        .expect("SSS product viewport should expose a frame");
    let stats = framework
        .query_stats()
        .expect("SSS product stats should exist");
    framework
        .destroy_viewport(viewport)
        .expect("SSS product viewport should be destroyed");

    ProductRender {
        frame,
        stats,
        submit_cpu_micros,
    }
}

fn skin_sphere_extract(
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let environment = EnvironmentExtract::procedural_default();
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.25, 4.6), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 48.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 40.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(PRODUCT_SIZE);
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![RenderMeshSnapshot {
                node_id: 18_600,
                stable_instance_key: 18_600 << 16,
                transform_revision: 1,
                transform: Transform {
                    scale: Vec3::splat(1.35),
                    ..Transform::default()
                },
                model,
                mesh: None,
                material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: RenderMeshStaticState::default(),
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            }],
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 18_601,
                light_id: 18_601,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                direction: Vec3::new(-0.65, -0.35, -0.68).normalize_or_zero(),
                color: Vec3::new(1.0, 0.86, 0.76),
                intensity: 3.2,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: None,
            }],
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: crate::core::framework::render::RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..Default::default()
        },
        environment: environment.clone(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            true,
            Vec4::new(0.018, 0.022, 0.03, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(18_602), snapshot)
        .with_viewport_size(PRODUCT_SIZE)
}

fn register_skin_material(
    asset_manager: &ProjectAssetManager,
    subsurface: bool,
    with_embedded_profile: bool,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse("res://materials/plan18-skin-sss.zmaterial").unwrap();
    let id = ResourceId::from_locator(&uri);
    let mut material = MaterialAsset {
        name: Some("Plan18 Skin SSS".to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.95, 0.42, 0.31, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.58,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0; 3],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    if subsurface {
        material.property_values.insert(
            "lighting_model".to_string(),
            toml::Value::String("custom:subsurface".to_string()),
        );
        material
            .property_values
            .insert("subsurface_profile".to_string(), toml::Value::Integer(7));
    }
    if with_embedded_profile {
        material.property_values.insert(
            "subsurface_scatter_radius".to_string(),
            toml::Value::Array(
                vec![8.0, 3.4, 1.5]
                    .into_iter()
                    .map(toml::Value::Float)
                    .collect(),
            ),
        );
        material.property_values.insert(
            "subsurface_falloff".to_string(),
            toml::Value::Array(
                vec![1.0, 0.48, 0.3]
                    .into_iter()
                    .map(toml::Value::Float)
                    .collect(),
            ),
        );
        material.property_values.insert(
            "subsurface_world_unit_scale".to_string(),
            toml::Value::Float(0.04),
        );
    }
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri)
                .with_source_hash("plan18-skin-sss-v1"),
            material,
        )
        .expect("SSS skin material should register");
    ResourceHandle::new(id)
}

fn register_skin_sphere_model(asset_manager: &ProjectAssetManager) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse("res://models/plan18-skin-sphere.zmodel").unwrap();
    let id = ResourceId::from_locator(&uri);
    let (vertices, indices) = uv_sphere_geometry(48, 72);
    let model = ModelAsset {
        uri: uri.clone(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            virtual_geometry: None,
        }],
    };
    asset_manager
        .assets::<ModelAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Model, uri)
                .with_source_hash("plan18-skin-sphere-v1"),
            model,
        )
        .expect("SSS skin sphere model should register");
    ResourceHandle::new(id)
}

fn uv_sphere_geometry(rings: usize, segments: usize) -> (Vec<MeshVertex>, Vec<u32>) {
    let rings = rings.max(3);
    let segments = segments.max(6);
    let mut vertices = Vec::with_capacity((rings + 1) * (segments + 1));
    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = theta.cos();
        let radius = theta.sin();
        for segment in 0..=segments {
            let phi = std::f32::consts::TAU * segment as f32 / segments as f32;
            let normal = Vec3::new(radius * phi.cos(), y, radius * phi.sin());
            vertices.push(MeshVertex::new(
                normal,
                normal,
                Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32),
            ));
        }
    }
    let mut indices = Vec::with_capacity(rings * segments * 6);
    for ring in 0..rings {
        for segment in 0..segments {
            let a = (ring * (segments + 1) + segment) as u32;
            let b = a + 1;
            let c = a + segments as u32 + 1;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    (vertices, indices)
}

fn product_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("advanced-lighting-sss-product")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
}

fn sss_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        SSS_SHADING_MODEL_ID,
        "custom:subsurface",
        "zr_shading_standard_pbr.wgsl",
        "zr_gbuffer_encode_subsurface.wgsl",
        "zr_shade_deferred_subsurface.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

fn frame_difference(baseline: &CapturedFrame, subsurface: &CapturedFrame) -> FrameDifference {
    assert_eq!(baseline.width, subsurface.width);
    assert_eq!(baseline.height, subsurface.height);
    let mut difference = FrameDifference::default();
    let mut absolute_error_sum = 0_u64;
    for (baseline_pixel, sss_pixel) in baseline
        .rgba
        .chunks_exact(4)
        .zip(subsurface.rgba.chunks_exact(4))
    {
        let mut changed = false;
        for channel in 0..3 {
            let error = baseline_pixel[channel].abs_diff(sss_pixel[channel]);
            changed |= error != 0;
            absolute_error_sum = absolute_error_sum.saturating_add(u64::from(error));
            difference.max_rgb_error = difference.max_rgb_error.max(error);
        }
        if changed {
            difference.changed_pixel_count += 1;
        }
        let baseline_rgb = u32::from(baseline_pixel[0])
            + u32::from(baseline_pixel[1])
            + u32::from(baseline_pixel[2]);
        let sss_rgb = u32::from(sss_pixel[0]) + u32::from(sss_pixel[1]) + u32::from(sss_pixel[2]);
        if sss_rgb > baseline_rgb + 2 {
            difference.brightened_pixel_count += 1;
        } else if baseline_rgb > sss_rgb + 2 {
            difference.darkened_pixel_count += 1;
        }
        let red_gain = i16::from(sss_pixel[0]) - i16::from(baseline_pixel[0]);
        let blue_gain = i16::from(sss_pixel[2]) - i16::from(baseline_pixel[2]);
        if red_gain > 1 && red_gain > blue_gain + 1 {
            difference.red_dominant_gain_count += 1;
        }
    }
    let sample_count = u64::from(baseline.width) * u64::from(baseline.height) * 3;
    difference.mean_absolute_rgb_error = absolute_error_sum as f64 / sample_count.max(1) as f64;
    difference
}

fn write_side_by_side_png(path: &Path, baseline: &CapturedFrame, subsurface: &CapturedFrame) {
    const SEPARATOR_WIDTH: u32 = 6;
    let output_width = baseline.width * 2 + SEPARATOR_WIDTH;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(output_width, baseline.height, |x, y| {
        if x < baseline.width {
            captured_pixel(baseline, x, y)
        } else if x < baseline.width + SEPARATOR_WIDTH {
            Rgba([235, 239, 245, 255])
        } else {
            captured_pixel(subsurface, x - baseline.width - SEPARATOR_WIDTH, y)
        }
    });
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("SSS side-by-side PNG should be writable");
}

fn captured_pixel(frame: &CapturedFrame, x: u32, y: u32) -> Rgba<u8> {
    let index = ((y * frame.width + x) * 4) as usize;
    Rgba([
        frame.rgba[index],
        frame.rgba[index + 1],
        frame.rgba[index + 2],
        frame.rgba[index + 3],
    ])
}

fn product_report(
    baseline: &ProductRender,
    subsurface: &ProductRender,
    difference: FrameDifference,
    image_path: &Path,
) -> String {
    format!(
        concat!(
            "Plan 18 AF-M4 SSS WGPU product evidence\n",
            "image={}\n",
            "viewport={}x{}\n",
            "shading_model_id={}\n",
            "burley_sample_count=64\n",
            "changed_pixel_count={}\n",
            "brightened_pixel_count={}\n",
            "darkened_pixel_count={}\n",
            "red_dominant_gain_count={}\n",
            "mean_absolute_rgb_error={:.4}\n",
            "max_rgb_error={}\n",
            "baseline_submit_cpu_micros={}\n",
            "sss_submit_cpu_micros={}\n",
            "sss_executed_passes={}\n",
        ),
        image_path.display(),
        PRODUCT_SIZE.x,
        PRODUCT_SIZE.y,
        SSS_SHADING_MODEL_ID.value(),
        difference.changed_pixel_count,
        difference.brightened_pixel_count,
        difference.darkened_pixel_count,
        difference.red_dominant_gain_count,
        difference.mean_absolute_rgb_error,
        difference.max_rgb_error,
        baseline.submit_cpu_micros,
        subsurface.submit_cpu_micros,
        subsurface.stats.last_graph_executed_passes.join(","),
    )
}

fn render_product_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should have repository parent")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
