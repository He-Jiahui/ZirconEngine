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
};

#[test]
fn virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_unified_indirect");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryUnifiedIndirect",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_two_primitive_gltf(
        paths
            .assets_root()
            .join("models")
            .join("double_triangle.gltf"),
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/double_triangle.gltf");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, green_material);
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
    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
                    cluster_draw_segments: vec![crate::types::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        resident_slot: Some(1),
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.last_virtual_geometry_indirect_draw_count(),
        2,
        "expected one indirect draw per primitive in the multi-primitive model"
    );
    assert_eq!(
        renderer.last_virtual_geometry_indirect_buffer_count(),
        1,
        "expected the multi-primitive model to keep reusing one shared indirect args buffer"
    );
    assert_eq!(
        renderer.last_virtual_geometry_indirect_segment_count(),
        1,
        "expected both primitive draws to reference one visibility-owned indirect segment instead of duplicating the same segment per draw"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_args()
            .unwrap()
            .len(),
        2,
        "expected the unified segment to still produce one indirect args record per primitive draw"
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

fn build_single_entity_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
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
            meshes: vec![RenderMeshSnapshot {
                node_id: 2,
                transform: Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
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
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        pages: vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
    });
    extract
}

fn write_two_primitive_gltf(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut binary = Vec::new();
    for value in [
        -1.0f32, -1.0, 0.0, //
        0.0, 1.0, 0.0, //
        -1.0, 1.0, 0.0,
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    for value in [
        0.0f32, -1.0, 0.0, //
        1.0, 1.0, 0.0, //
        1.0, -1.0, 0.0,
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }

    let buffer_path = path
        .ancestors()
        .nth(3)
        .expect("gltf path should live under <root>/assets/models")
        .join(".generated")
        .join("double_triangle.bin");
    if let Some(parent) = buffer_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&buffer_path, &binary).unwrap();

    let gltf = format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [
    {{"uri": "../../.generated/double_triangle.bin", "byteLength": {}}}
  ],
  "bufferViews": [
    {{"buffer":0,"byteOffset":0,"byteLength":36,"target":34962}},
    {{"buffer":0,"byteOffset":36,"byteLength":12,"target":34963}},
    {{"buffer":0,"byteOffset":48,"byteLength":36,"target":34962}},
    {{"buffer":0,"byteOffset":84,"byteLength":12,"target":34963}}
  ],
  "accessors": [
    {{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[0.0,1.0,0.0]}},
    {{"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR","min":[0],"max":[2]}},
    {{"bufferView":2,"componentType":5126,"count":3,"type":"VEC3","min":[0.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
    {{"bufferView":3,"componentType":5125,"count":3,"type":"SCALAR","min":[0],"max":[2]}}
  ],
  "meshes": [
    {{
      "primitives": [
        {{"attributes":{{"POSITION":0}},"indices":1}},
        {{"attributes":{{"POSITION":2}},"indices":3}}
      ]
    }}
  ],
  "nodes": [
    {{"mesh": 0}}
  ],
  "scenes": [
    {{"nodes": [0]}}
  ],
  "scene": 0
}}"#,
        binary.len()
    );
    fs::write(path, gltf).unwrap();
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

fn write_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("VirtualGeometryUnifiedIndirect".to_string()),
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
