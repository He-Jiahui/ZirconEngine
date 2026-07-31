use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::assets::{
    AlphaMode, MaterialAsset, ModelAsset, ModelPrimitiveAsset,
    texture_asset_from_lightmap_bake_output,
};
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, AssetUri, MeshVertex, TextureAsset};
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, DisplayMode, EnvironmentExtract, LightmapBakeOutput,
    PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderMeshStaticState, RenderOverlayExtract, RenderPipelineHandle,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    render_mesh_stable_instance_key,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;

const FIXTURE: &str = include_str!("fixtures/plan11_baked_lighting_v1.json");
const PRODUCT_SIZE: UVec2 = UVec2::new(640, 360);
const STATIC_NODE_ID: u64 = 100;
const DYNAMIC_NODE_ID: u64 = 101;
const FORWARD_PIPELINE: RenderPipelineHandle = RenderPipelineHandle::new(1);
const DEFERRED_PIPELINE: RenderPipelineHandle = RenderPipelineHandle::new(2);
const PRODUCT_IMAGE_NAME: &str = "plan11_lightmap_probe_forward_deferred_wgpu_20260713.png";
const PRODUCT_REPORT_NAME: &str = "plan11_lightmap_probe_forward_deferred_wgpu_20260713.txt";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RegionDifference {
    changed_pixels: u64,
    mean_absolute_rgb_error: f64,
    max_rgb_error: u8,
}

struct ProductCapture {
    frame: CapturedFrame,
    stats: RenderStats,
}

#[test]
fn render_env_external_bake_fixture_preserves_static_slot_and_probe_grid() {
    let output = external_bake_fixture();

    output.validate().expect("external fixture should validate");
    assert_eq!(output.atlas.page_size, 2);
    assert_eq!(output.atlas_pages[0].texels_rgba16f_le.len(), 32);
    assert_eq!(
        output.slots[0].0,
        render_mesh_stable_instance_key(STATIC_NODE_ID, 0)
    );
    assert_eq!(output.probe_grid.as_ref().unwrap().dims, [2, 2, 2]);
}

#[test]
fn render_product_baked_lightmap_and_dynamic_probe_match_forward_deferred() {
    let baseline = capture_scene(FORWARD_PIPELINE, false, 11_300);
    let forward = capture_scene(FORWARD_PIPELINE, true, 11_301);
    let deferred = capture_scene(DEFERRED_PIPELINE, true, 11_302);

    let static_forward = region_difference(&baseline.frame, &forward.frame, 0, 340);
    let dynamic_forward = region_difference(&baseline.frame, &forward.frame, 340, 640);
    let static_deferred = region_difference(&baseline.frame, &deferred.frame, 0, 340);
    let dynamic_deferred = region_difference(&baseline.frame, &deferred.frame, 340, 640);
    let parity = region_difference(&forward.frame, &deferred.frame, 0, PRODUCT_SIZE.x);

    assert!(
        static_forward.changed_pixels > 5_000 && static_deferred.changed_pixels > 5_000,
        "the UV2 lightmap must illuminate a broad static surface: forward={static_forward:?}, deferred={static_deferred:?}"
    );
    assert!(
        dynamic_forward.changed_pixels > 1_000 && dynamic_deferred.changed_pixels > 1_000,
        "the unmapped dynamic object must receive probe-grid irradiance: forward={dynamic_forward:?}, deferred={dynamic_deferred:?}"
    );
    assert!(
        static_forward.mean_absolute_rgb_error > 2.0
            && dynamic_forward.mean_absolute_rgb_error > 0.5,
        "baked lighting must be numerically visible in both fixture regions"
    );
    assert!(
        parity.mean_absolute_rgb_error < 4.0,
        "Forward+ and Deferred should consume the same baked-indirect contract: {parity:?}"
    );
    assert_eq!(forward.stats.last_pipeline, Some(FORWARD_PIPELINE));
    assert_eq!(deferred.stats.last_pipeline, Some(DEFERRED_PIPELINE));
}

#[test]
#[ignore = "exports the Plan 11 EL-M3 external-bake Forward+/Deferred WGPU product evidence"]
fn export_render_product_baked_lightmap_probe_png() {
    let baseline = capture_scene(FORWARD_PIPELINE, false, 11_310);
    let forward = capture_scene(FORWARD_PIPELINE, true, 11_311);
    let deferred = capture_scene(DEFERRED_PIPELINE, true, 11_312);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");
    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_three_panel_png(
        &image_path,
        [&baseline.frame, &forward.frame, &deferred.frame],
    );

    let report = format!(
        concat!(
            "Plan 11 EL-M3 lightmap/probe WGPU product evidence\n",
            "fixture=zircon_runtime/src/graphics/tests/fixtures/plan11_baked_lighting_v1.json\n",
            "image={}\n",
            "viewport={}x{}\n",
            "static_forward={:?}\n",
            "dynamic_forward={:?}\n",
            "static_deferred={:?}\n",
            "dynamic_deferred={:?}\n",
            "forward_deferred_parity={:?}\n",
            "forward_passes={:?}\n",
            "deferred_passes={:?}\n",
        ),
        image_path.display(),
        PRODUCT_SIZE.x,
        PRODUCT_SIZE.y,
        region_difference(&baseline.frame, &forward.frame, 0, 340),
        region_difference(&baseline.frame, &forward.frame, 340, 640),
        region_difference(&baseline.frame, &deferred.frame, 0, 340),
        region_difference(&baseline.frame, &deferred.frame, 340, 640),
        region_difference(&forward.frame, &deferred.frame, 0, PRODUCT_SIZE.x),
        forward.stats.last_graph_executed_passes,
        deferred.stats.last_graph_executed_passes,
    );
    fs::write(output_dir.join(PRODUCT_REPORT_NAME), report)
        .expect("baked-lighting product report should be writable");
}

fn capture_scene(
    pipeline: RenderPipelineHandle,
    baked_lighting: bool,
    world_id: u64,
) -> ProductCapture {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = register_fixture_plane(&asset_manager);
    let material = register_fixture_material(&asset_manager);
    let environment = register_fixture_environment(&asset_manager, baked_lighting);
    let framework = WgpuRenderFramework::new_for_test(asset_manager)
        .expect("WGPU framework should initialize for baked-lighting product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_SIZE))
        .expect("baked-lighting product viewport should be created");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("product pipeline should be accepted");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("baked-lighting product profile should be accepted");
    framework
        .submit_frame_extract(
            viewport,
            fixture_extract(world_id, environment, model, material),
        )
        .expect("baked-lighting product frame should submit");
    let frame = framework
        .capture_frame(viewport)
        .expect("baked-lighting capture should succeed")
        .expect("baked-lighting viewport should expose a frame");
    let stats = framework.query_stats().expect("render stats should exist");
    framework
        .destroy_viewport(viewport)
        .expect("baked-lighting viewport should be destroyed");
    ProductCapture { frame, stats }
}

fn register_fixture_environment(
    asset_manager: &ProjectAssetManager,
    enabled: bool,
) -> EnvironmentExtract {
    if !enabled {
        return EnvironmentExtract::disabled();
    }
    let output = external_bake_fixture();
    let uri = AssetUri::parse("res://lighting/plan11-external-fixture.lightmap-array")
        .expect("fixture lightmap URI should be valid");
    let atlas_id = ResourceId::from_locator(&uri);
    let texture = texture_asset_from_lightmap_bake_output(uri.clone(), &output)
        .expect("external bake fixture should import as an RGBA16F array");
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(atlas_id, ResourceKind::Texture, uri)
                .with_source_hash("plan11-external-bake-v1"),
            texture,
        )
        .expect("external fixture lightmap should register");
    let (contract, probe_grid) = output
        .into_consume_contract(atlas_id)
        .expect("external bake fixture should produce a consume contract");
    EnvironmentExtract::disabled()
        .try_with_baked_lighting(contract, probe_grid)
        .expect("matching fixture generations should be accepted")
}

fn external_bake_fixture() -> LightmapBakeOutput {
    serde_json::from_str(FIXTURE).expect("Plan 11 external bake fixture JSON should decode")
}

fn fixture_extract(
    world_id: u64,
    environment: EnvironmentExtract,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 7.0), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 48.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 30.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(PRODUCT_SIZE);
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                fixture_mesh(
                    STATIC_NODE_ID,
                    Vec3::new(-1.55, 0.0, 0.0),
                    Vec3::new(1.35, 1.65, 1.0),
                    model,
                    material,
                    Mobility::Static,
                ),
                fixture_mesh(
                    DYNAMIC_NODE_ID,
                    Vec3::new(1.6, 0.0, 0.0),
                    Vec3::splat(1.25),
                    ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
                    material,
                    Mobility::Dynamic,
                ),
            ],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: environment.clone(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            false,
            Vec4::new(0.005, 0.007, 0.01, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(world_id), snapshot)
        .with_viewport_size(PRODUCT_SIZE)
}

fn fixture_mesh(
    node_id: u64,
    translation: Vec3,
    scale: Vec3,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    mobility: Mobility,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: 1,
        transform: Transform {
            translation,
            scale,
            ..Transform::default()
        },
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility,
        static_state: RenderMeshStaticState::new(mobility == Mobility::Static, 1, 1),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: mobility == Mobility::Static,
            ..Default::default()
        },
    }
}

fn register_fixture_plane(asset_manager: &ProjectAssetManager) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse("res://models/plan11-lightmap-plane.zmodel").unwrap();
    let id = ResourceId::from_locator(&uri);
    let vertices = vec![
        MeshVertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::Z, Vec2::new(0.0, 1.0))
            .with_uv1(Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(1.0, -1.0, 0.0), Vec3::Z, Vec2::new(1.0, 1.0))
            .with_uv1(Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(-1.0, 1.0, 0.0), Vec3::Z, Vec2::new(0.0, 0.0))
            .with_uv1(Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(1.0, 1.0, 0.0), Vec3::Z, Vec2::new(1.0, 0.0))
            .with_uv1(Vec2::new(1.0, 0.0)),
    ];
    let model = ModelAsset {
        uri: uri.clone(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices: vec![0, 1, 2, 2, 1, 3],
            mesh: None,
            virtual_geometry: None,
        }],
    };
    asset_manager
        .assets::<ModelAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Model, uri)
                .with_source_hash("plan11-lightmap-plane-v1"),
            model,
        )
        .expect("fixture plane should register");
    ResourceHandle::new(id)
}

fn register_fixture_material(
    asset_manager: &ProjectAssetManager,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse("res://materials/plan11-baked-white.zmaterial").unwrap();
    let id = ResourceId::from_locator(&uri);
    let mut material = MaterialAsset {
        name: Some("Plan11 Baked White".to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.92, 0.92, 0.92, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.72,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0; 3],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri)
                .with_source_hash("plan11-baked-white-v1"),
            material,
        )
        .expect("fixture material should register");
    ResourceHandle::new(id)
}

fn product_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("plan11-baked-lighting-product")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_reflection_probes(false)
        .with_baked_lighting(true)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
}

fn region_difference(
    baseline: &CapturedFrame,
    candidate: &CapturedFrame,
    x_start: u32,
    x_end: u32,
) -> RegionDifference {
    assert_eq!(
        (baseline.width, baseline.height),
        (candidate.width, candidate.height)
    );
    let mut difference = RegionDifference::default();
    let mut absolute_error = 0_u64;
    let mut sample_count = 0_u64;
    for y in 0..baseline.height {
        for x in x_start.min(baseline.width)..x_end.min(baseline.width) {
            let index = ((y * baseline.width + x) * 4) as usize;
            let mut changed = false;
            for channel in 0..3 {
                let error =
                    baseline.rgba[index + channel].abs_diff(candidate.rgba[index + channel]);
                changed |= error != 0;
                difference.max_rgb_error = difference.max_rgb_error.max(error);
                absolute_error += u64::from(error);
                sample_count += 1;
            }
            difference.changed_pixels += u64::from(changed);
        }
    }
    difference.mean_absolute_rgb_error = absolute_error as f64 / sample_count.max(1) as f64;
    difference
}

fn write_three_panel_png(path: &Path, frames: [&CapturedFrame; 3]) {
    const SEPARATOR: u32 = 6;
    let width = frames[0].width * 3 + SEPARATOR * 2;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(width, frames[0].height, |x, y| {
        if x < frames[0].width {
            captured_pixel(frames[0], x, y)
        } else if x < frames[0].width + SEPARATOR {
            Rgba([235, 239, 245, 255])
        } else if x < frames[0].width * 2 + SEPARATOR {
            captured_pixel(frames[1], x - frames[0].width - SEPARATOR, y)
        } else if x < frames[0].width * 2 + SEPARATOR * 2 {
            Rgba([235, 239, 245, 255])
        } else {
            captured_pixel(frames[2], x - frames[0].width * 2 - SEPARATOR * 2, y)
        }
    });
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("baked-lighting three-panel PNG should be writable");
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

fn render_product_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should have a repository parent")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
