use std::{fs, path::Path, sync::Arc, time::Instant};

use crate::asset::assets::{AlphaMode, MaterialAsset, ModelAsset, ModelPrimitiveAsset};
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, AssetUri, MeshVertex};
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, DisplayMode, EnvironmentExtract, PreviewEnvironmentExtract,
    ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Quat, Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;

use super::{render_product_output_dir, write_side_by_side_png, ProductRender};

pub(super) const PRODUCT_SIZE: UVec2 = UVec2::new(640, 360);
const ADVANCED_PBR_OPAQUE_PASS_NAME: &str = "advanced-pbr-opaque";
const TRANSMISSION_SCENE_COPY_PASS_NAME: &str = "transmission.scene_copy";
const TRANSMISSION_MESH_PASS_NAME: &str = "transmission-mesh.0";
const PRODUCT_IMAGE_NAME: &str =
    "plan18_advanced_pbr_clearcoat_anisotropy_glass_three_spheres_wgpu_20260714.png";
const PRODUCT_REPORT_NAME: &str =
    "plan18_advanced_pbr_clearcoat_anisotropy_glass_three_spheres_wgpu_20260714.txt";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RegionDifference {
    changed_pixel_count: u64,
    mean_absolute_rgb_error: f64,
    max_rgb_error: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProductMaterialMode {
    Baseline,
    Advanced,
}

#[test]
fn render_product_advanced_pbr_three_spheres_execute_owned_passes() {
    let baseline = render_three_spheres(ProductMaterialMode::Baseline);
    let advanced = render_three_spheres(ProductMaterialMode::Advanced);

    for pass in [
        ADVANCED_PBR_OPAQUE_PASS_NAME,
        TRANSMISSION_SCENE_COPY_PASS_NAME,
        TRANSMISSION_MESH_PASS_NAME,
    ] {
        assert!(
            !baseline
                .stats
                .last_graph_executed_passes
                .iter()
                .any(|executed| executed == pass),
            "baseline unexpectedly executed {pass}: {:?}",
            baseline.stats.last_graph_executed_passes
        );
        assert!(
            advanced
                .stats
                .last_graph_executed_passes
                .iter()
                .any(|executed| executed == pass),
            "advanced PBR product graph did not execute {pass}: {:?}",
            advanced.stats.last_graph_executed_passes
        );
    }

    let advanced_passes = &advanced.stats.last_graph_executed_passes;
    let advanced_opaque_index = pass_index(advanced_passes, ADVANCED_PBR_OPAQUE_PASS_NAME);
    let scene_copy_index = pass_index(advanced_passes, TRANSMISSION_SCENE_COPY_PASS_NAME);
    let transmission_index = pass_index(advanced_passes, TRANSMISSION_MESH_PASS_NAME);
    assert!(
        advanced_opaque_index < scene_copy_index && scene_copy_index < transmission_index,
        "late-forward opaque, scene copy, and transmission must execute in order: {advanced_passes:?}"
    );

    let left = region_difference(&baseline.frame, &advanced.frame, 24, 92, 220, 306);
    let center = region_difference(&baseline.frame, &advanced.frame, 220, 92, 420, 306);
    let right = region_difference(&baseline.frame, &advanced.frame, 340, 76, 548, 322);
    assert_region_changed("clearcoat", left, 500, 0.04);
    assert_region_changed("anisotropy", center, 500, 0.04);
    assert_region_changed("glass transmission", right, 800, 0.08);
}

#[test]
#[ignore = "exports the plan-18 AF-M1 clearcoat, anisotropy, and glass WGPU product evidence"]
fn export_render_product_advanced_pbr_three_spheres_png() {
    let baseline = render_three_spheres(ProductMaterialMode::Baseline);
    let advanced = render_three_spheres(ProductMaterialMode::Advanced);
    let left = region_difference(&baseline.frame, &advanced.frame, 24, 92, 220, 306);
    let center = region_difference(&baseline.frame, &advanced.frame, 220, 92, 420, 306);
    let right = region_difference(&baseline.frame, &advanced.frame, 340, 76, 548, 322);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");

    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_side_by_side_png(&image_path, &baseline.frame, &advanced.frame);
    let report_path = output_dir.join(PRODUCT_REPORT_NAME);
    fs::write(
        &report_path,
        product_report(&baseline, &advanced, left, center, right, &image_path),
    )
    .expect("advanced PBR product report should be writable");

    assert!(
        image_path.is_file(),
        "advanced PBR product PNG was not exported"
    );
    assert!(
        report_path.is_file(),
        "advanced PBR product report was not exported"
    );
}

#[test]
#[ignore = "captures the corrected advanced-only DX12 frame through RenderDoc"]
fn capture_render_product_advanced_pbr_three_spheres_renderdoc() {
    let advanced = render_three_spheres(ProductMaterialMode::Advanced);

    for pass in [
        ADVANCED_PBR_OPAQUE_PASS_NAME,
        TRANSMISSION_SCENE_COPY_PASS_NAME,
        TRANSMISSION_MESH_PASS_NAME,
    ] {
        assert!(
            advanced
                .stats
                .last_graph_executed_passes
                .iter()
                .any(|executed| executed == pass),
            "advanced-only RenderDoc frame did not execute {pass}: {:?}",
            advanced.stats.last_graph_executed_passes
        );
    }
}

fn render_three_spheres(mode: ProductMaterialMode) -> ProductRender {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = register_sphere_model(&asset_manager);
    let materials = ProductMaterials::register(&asset_manager, mode);
    let framework = WgpuRenderFramework::new_for_test(asset_manager)
        .expect("WGPU framework should initialize for the advanced PBR product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_SIZE))
        .expect("advanced PBR product viewport should be created");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("advanced PBR product quality profile should be accepted");

    let start = Instant::now();
    framework
        .submit_frame_extract(viewport, three_sphere_extract(model, materials))
        .expect("advanced PBR product frame should submit");
    let submit_cpu_micros = start.elapsed().as_micros();
    let frame = framework
        .capture_frame(viewport)
        .expect("advanced PBR product capture should succeed")
        .expect("advanced PBR product viewport should expose a frame");
    let stats = framework
        .query_stats()
        .expect("advanced PBR product stats should exist");
    framework
        .destroy_viewport(viewport)
        .expect("advanced PBR product viewport should be destroyed");

    ProductRender {
        frame,
        stats,
        submit_cpu_micros,
    }
}

#[derive(Clone, Copy)]
pub(super) struct ProductMaterials {
    clearcoat: ResourceHandle<MaterialMarker>,
    anisotropy: ResourceHandle<MaterialMarker>,
    glass: ResourceHandle<MaterialMarker>,
    magenta: ResourceHandle<MaterialMarker>,
    cyan: ResourceHandle<MaterialMarker>,
}

impl ProductMaterials {
    pub(super) fn register(asset_manager: &ProjectAssetManager, mode: ProductMaterialMode) -> Self {
        let advanced = mode == ProductMaterialMode::Advanced;
        Self {
            clearcoat: register_material(
                asset_manager,
                "clearcoat",
                [0.08, 0.24, 0.72, 1.0],
                0.18,
                0.22,
                AlphaMode::Opaque,
                advanced.then_some(&[
                    ("clearcoat", toml::Value::Float(1.0)),
                    ("clearcoat_perceptual_roughness", toml::Value::Float(0.05)),
                ]),
            ),
            anisotropy: register_material(
                asset_manager,
                "anisotropy",
                [0.88, 0.29, 0.055, 1.0],
                0.18,
                0.3,
                AlphaMode::Opaque,
                advanced.then_some(&[
                    ("anisotropy_strength", toml::Value::Float(0.94)),
                    ("anisotropy_rotation", toml::Value::Float(0.72)),
                ]),
            ),
            glass: register_material(
                asset_manager,
                "glass",
                [0.24, 0.72, 0.98, 0.56],
                0.0,
                0.05,
                AlphaMode::Blend,
                advanced.then_some(&[
                    ("specular_transmission", toml::Value::Float(0.92)),
                    ("diffuse_transmission", toml::Value::Float(0.08)),
                    ("thickness", toml::Value::Float(2.4)),
                    ("ior", toml::Value::Float(1.38)),
                    (
                        "attenuation_color",
                        toml::Value::Array(vec![
                            toml::Value::Float(0.28),
                            toml::Value::Float(0.78),
                            toml::Value::Float(1.0),
                        ]),
                    ),
                    ("attenuation_distance", toml::Value::Float(2.0)),
                ]),
            ),
            magenta: register_material(
                asset_manager,
                "backing-magenta",
                [0.94, 0.04, 0.34, 1.0],
                0.0,
                0.44,
                AlphaMode::Opaque,
                None,
            ),
            cyan: register_material(
                asset_manager,
                "backing-cyan",
                [0.02, 0.82, 0.92, 1.0],
                0.0,
                0.38,
                AlphaMode::Opaque,
                None,
            ),
        }
    }
}

pub(super) fn three_sphere_extract(
    model: ResourceHandle<ModelMarker>,
    materials: ProductMaterials,
) -> RenderFrameExtract {
    let environment = EnvironmentExtract::procedural_default();
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.28, 7.4), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 49.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 50.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(PRODUCT_SIZE);
    let meshes = vec![
        product_mesh(
            18_100,
            Vec3::new(-2.05, 0.0, 0.0),
            Vec3::splat(1.04),
            Quat::IDENTITY,
            model,
            materials.clearcoat,
        ),
        product_mesh(
            18_101,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::splat(1.04),
            Quat::from_rotation_z(0.34),
            model,
            materials.anisotropy,
        ),
        product_mesh(
            18_102,
            Vec3::new(2.05, 0.0, 0.0),
            Vec3::splat(1.04),
            Quat::IDENTITY,
            model,
            materials.glass,
        ),
        product_mesh(
            18_103,
            Vec3::new(1.72, 0.3, -1.45),
            Vec3::splat(0.42),
            Quat::IDENTITY,
            model,
            materials.magenta,
        ),
        product_mesh(
            18_104,
            Vec3::new(2.4, -0.26, -1.5),
            Vec3::splat(0.38),
            Quat::IDENTITY,
            model,
            materials.cyan,
        ),
    ];
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes,
            directional_lights: vec![
                RenderDirectionalLightSnapshot {
                    node_id: 18_110,
                    light_id: 18_110,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                    direction: Vec3::new(-0.72, -0.42, -0.56).normalize_or_zero(),
                    color: Vec3::new(1.0, 0.9, 0.78),
                    intensity: 3.4,
                    mobility: Mobility::Dynamic,
                    shadow: None,
                },
                RenderDirectionalLightSnapshot {
                    node_id: 18_111,
                    light_id: 18_111,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                    direction: Vec3::new(0.58, -0.18, -0.8).normalize_or_zero(),
                    color: Vec3::new(0.34, 0.56, 1.0),
                    intensity: 1.35,
                    mobility: Mobility::Dynamic,
                    shadow: None,
                },
            ],
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
            Vec4::new(0.012, 0.016, 0.026, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(18_120), snapshot)
        .with_viewport_size(PRODUCT_SIZE)
}

fn product_mesh(
    node_id: u64,
    translation: Vec3,
    scale: Vec3,
    rotation: Quat,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 1,
        transform: Transform {
            translation,
            rotation,
            scale,
        },
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            ..Default::default()
        },
    }
}

fn register_material(
    asset_manager: &ProjectAssetManager,
    name: &str,
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    alpha_mode: AlphaMode,
    properties: Option<&[(&str, toml::Value)]>,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse(&format!(
        "res://materials/plan18-advanced-pbr-{name}.zmaterial"
    ))
    .expect("advanced PBR product material URI should be valid");
    let id = ResourceId::from_locator(&uri);
    let mut material = MaterialAsset {
        name: Some(format!("Plan18 Advanced PBR {name}")),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic,
        roughness,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0; 3],
        emissive_texture: None,
        alpha_mode,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    for (property, value) in properties.into_iter().flatten() {
        material
            .property_values
            .insert((*property).to_string(), value.clone());
    }
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri)
                .with_source_hash(format!("plan18-advanced-pbr-{name}-v1")),
            material,
        )
        .expect("advanced PBR product material should register");
    ResourceHandle::new(id)
}

pub(super) fn register_sphere_model(
    asset_manager: &ProjectAssetManager,
) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse("res://models/plan18-advanced-pbr-sphere.zmodel").unwrap();
    let id = ResourceId::from_locator(&uri);
    let (vertices, indices) = uv_sphere_geometry(40, 64);
    let model = ModelAsset {
        uri: uri.clone(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    };
    asset_manager
        .assets::<ModelAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Model, uri)
                .with_source_hash("plan18-advanced-pbr-sphere-v1"),
            model,
        )
        .expect("advanced PBR product sphere should register");
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

pub(super) fn product_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("advanced-pbr-three-sphere-product")
        .with_clustered_lighting(true)
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

fn pass_index(passes: &[String], executor: &str) -> usize {
    passes
        .iter()
        .position(|candidate| candidate == executor)
        .unwrap_or_else(|| panic!("executor {executor} missing from {passes:?}"))
}

fn region_difference(
    baseline: &CapturedFrame,
    advanced: &CapturedFrame,
    x_min: u32,
    y_min: u32,
    x_max: u32,
    y_max: u32,
) -> RegionDifference {
    assert_eq!(
        (baseline.width, baseline.height),
        (advanced.width, advanced.height)
    );
    let mut difference = RegionDifference::default();
    let mut absolute_error_sum = 0_u64;
    let mut channel_count = 0_u64;
    for y in y_min.min(baseline.height)..y_max.min(baseline.height) {
        for x in x_min.min(baseline.width)..x_max.min(baseline.width) {
            let index = ((y * baseline.width + x) * 4) as usize;
            let mut changed = false;
            for channel in 0..3 {
                let error = baseline.rgba[index + channel].abs_diff(advanced.rgba[index + channel]);
                absolute_error_sum += u64::from(error);
                difference.max_rgb_error = difference.max_rgb_error.max(error);
                changed |= error != 0;
                channel_count += 1;
            }
            difference.changed_pixel_count += changed as u64;
        }
    }
    if channel_count > 0 {
        difference.mean_absolute_rgb_error = absolute_error_sum as f64 / channel_count as f64;
    }
    difference
}

fn assert_region_changed(
    label: &str,
    difference: RegionDifference,
    minimum_changed_pixels: u64,
    minimum_mean_error: f64,
) {
    assert!(
        difference.changed_pixel_count > minimum_changed_pixels,
        "{label} region did not expose enough changed pixels: {difference:?}"
    );
    assert!(
        difference.mean_absolute_rgb_error > minimum_mean_error,
        "{label} region did not expose a measurable shading difference: {difference:?}"
    );
}

fn product_report(
    baseline: &ProductRender,
    advanced: &ProductRender,
    clearcoat: RegionDifference,
    anisotropy: RegionDifference,
    glass: RegionDifference,
    image_path: &Path,
) -> String {
    format!(
        concat!(
            "Plan 18 AF-M1 advanced PBR WGPU product evidence\n",
            "image={}\n",
            "viewport={}x{}\n",
            "layout=baseline_left_advanced_right\n",
            "subjects=clearcoat_left_anisotropy_center_glass_right\n",
            "clearcoat_changed_pixels={}\n",
            "clearcoat_mean_absolute_rgb_error={:.4}\n",
            "anisotropy_changed_pixels={}\n",
            "anisotropy_mean_absolute_rgb_error={:.4}\n",
            "glass_changed_pixels={}\n",
            "glass_mean_absolute_rgb_error={:.4}\n",
            "baseline_submit_cpu_micros={}\n",
            "advanced_submit_cpu_micros={}\n",
            "advanced_executed_passes={}\n",
            "transmission_steps=1\n",
            "transmission_queue=2900\n",
        ),
        image_path.display(),
        PRODUCT_SIZE.x,
        PRODUCT_SIZE.y,
        clearcoat.changed_pixel_count,
        clearcoat.mean_absolute_rgb_error,
        anisotropy.changed_pixel_count,
        anisotropy.mean_absolute_rgb_error,
        glass.changed_pixel_count,
        glass.mean_absolute_rgb_error,
        baseline.submit_cpu_micros,
        advanced.submit_cpu_micros,
        advanced.stats.last_graph_executed_passes.join(","),
    )
}
