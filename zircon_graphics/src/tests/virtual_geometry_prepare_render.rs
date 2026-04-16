use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ProjectAssetManager, ProjectManifest,
    ProjectPaths,
};
use zircon_manager::AssetManager;
use zircon_math::{Transform, UVec2, Vec3, Vec4};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_scene::{
    default_render_layer_mask, DisplayMode, FallbackSkyboxKind, Mobility,
    PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract, RenderMeshSnapshot,
    RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};

use crate::{
    types::{
        EditorOrRuntimeFrame, VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
    ViewportState,
};

#[test]
fn virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities() {
    let root = unique_temp_project_root("graphics_virtual_geometry_prepare");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPrepare",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_red.wgsl"),
        [0.95, 0.08, 0.08],
    );
    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_red.material.toml"),
        "res://shaders/flat_red.wgsl",
        "res://textures/white.png",
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.material.toml"),
        "res://shaders/flat_green.wgsl",
        "res://textures/white.png",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = zircon_asset::ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let red_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_red.material.toml");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_extract(viewport_size, model, red_material, green_material);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_async_compute(false),
        )
        .unwrap();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let unfiltered = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), ViewportState::new(viewport_size)),
            &compiled,
            None,
        )
        .unwrap();
    let filtered = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let unfiltered_green = average_channel(&unfiltered.rgba, 1);
    let filtered_red = average_channel(&filtered.rgba, 0);
    let filtered_green = average_channel(&filtered.rgba, 1);

    assert!(
        filtered_green > filtered_red + 8.0,
        "expected filtered Virtual Geometry fallback to keep only the green entity; red={filtered_red:.2}, green={filtered_green:.2}"
    );
    assert!(
        filtered_green > unfiltered_green + 4.0,
        "expected filtering to increase the green contribution; unfiltered={unfiltered_green:.2}, filtered={filtered_green:.2}"
    );
    assert_ne!(
        unfiltered.rgba, filtered.rgba,
        "expected Virtual Geometry prepare filtering to change the fallback frame output"
    );

    let _ = fs::remove_dir_all(root);
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}

fn build_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    red_material: ResourceHandle<MaterialMarker>,
    green_material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                RenderMeshSnapshot {
                    node_id: 1,
                    transform: Transform {
                        translation: Vec3::new(-0.55, 0.0, 0.0),
                        scale: Vec3::new(0.55, 0.55, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material: red_material,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
                RenderMeshSnapshot {
                    node_id: 2,
                    transform: Transform {
                        translation: Vec3::new(0.55, 0.0, 0.0),
                        scale: Vec3::new(0.55, 0.55, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material: green_material,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
            ],
            lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity: 1,
                cluster_id: 1,
                page_id: 200,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::new(-0.55, 0.0, 0.0),
                bounds_radius: 0.8,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::new(0.55, 0.0, 0.0),
                bounds_radius: 0.8,
                screen_space_error: 1.0,
            },
        ],
        pages: vec![
            RenderVirtualGeometryPage {
                page_id: 200,
                resident: true,
                size_bytes: 2048,
            },
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: true,
                size_bytes: 4096,
            },
        ],
    });
    extract
}

fn write_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            r#"
struct SceneUniform {{
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
}};
struct ModelUniform {{
    model: mat4x4<f32>,
    tint: vec4<f32>,
}};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {{
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    return output;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>({:.6}, {:.6}, {:.6}, alpha) * model_data.tint;
}}
"#,
            color[0], color[1], color[2]
        ),
    )
    .unwrap();
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 1.0 1.0 0.0
v -1.0 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0
vt 1.0 0.0
vt 0.0 0.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
",
    )
    .unwrap();
}

fn write_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("VirtualGeometryTest".to_string()),
        shader: asset_reference(shader_uri),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: Some(asset_reference(texture_uri)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}

fn average_channel(rgba: &[u8], channel: usize) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let total = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[channel] as f32)
        .sum::<f32>();
    total / (rgba.len() as f32 / 4.0)
}
