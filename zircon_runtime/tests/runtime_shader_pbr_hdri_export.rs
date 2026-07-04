use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::assets::{
    AlphaMode, MaterialAsset, MeshAssetUsage, MeshAttributeValues, MeshIndices, SceneAsset,
    SceneCameraAsset, SceneDirectionalLightAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMobilityAsset, TransformAsset, ZMeshDocument, MESH_ATTRIBUTE_COLOR,
    MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0, ZMESH_DOCUMENT_VERSION,
};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, ProjectionMode, RenderMeshTopology,
    RenderOverlayExtract, SampledEquirectangularEnvironment, SceneViewportExtractRequest,
    ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::graphics::SceneRenderer;

const PBR_MATRIX_DIMENSION: usize = 8;
const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1280, 960);
const PBR_MATRIX_ORTHO_SIZE: f32 = 6.4;
const PBR_MATRIX_STEP_X: f32 = 0.74;
const PBR_MATRIX_STEP_Y: f32 = 0.68;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.27;

#[test]
#[ignore = "manual screenshot export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_export".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_reflection_png_inner)
        .expect("spawn large-stack HDRI export test")
        .join()
        .expect("HDRI export test thread should not panic");
}

fn export_runtime_shader_pbr_real_hdri_reflection_png_inner() {
    let frame = render_pbr_matrix_frame_with_environment(
        EnvironmentExtract::sampled_equirectangular(polyhaven_lakes_sampled_environment()),
    );
    let output = shader_test_output_dir()
        .join("runtime_shader_pbr_real_hdri_lakes_pmrem_reflection_20260705.png");

    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("rendered real HDRI PBR frame should match output image dimensions")
        .save_with_format(&output, ImageFormat::Png)
        .expect("write real HDRI PBR reflection screenshot");

    assert_real_hdri_reflection_response(&frame);
}

fn render_pbr_matrix_frame_with_environment(
    environment: EnvironmentExtract,
) -> zircon_runtime::graphics::ViewportFrame {
    let root = unique_temp_project_root("graphics_pbr_real_hdri_integration");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsPbrRealHdriIntegration",
        AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_uv_sphere_zmesh(
        paths
            .assets_root()
            .join("models")
            .join("pbr_matrix_sphere.zmesh"),
        24,
        48,
    );
    for row in 0..PBR_MATRIX_DIMENSION {
        for column in 0..PBR_MATRIX_DIMENSION {
            write_pbr_matrix_material(
                paths
                    .assets_root()
                    .join("materials")
                    .join(format!("pbr_matrix_r{row}_c{column}.zmaterial")),
                pbr_matrix_axis_value(column),
                pbr_matrix_axis_value(row),
            );
        }
    }
    write_pbr_matrix_scene(
        paths
            .assets_root()
            .join("scenes")
            .join("pbr_matrix.scene.toml"),
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world =
        zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(PBR_MATRIX_OUTPUT_SIZE),
        virtual_geometry_debug: None,
    });
    snapshot.environment = environment;
    snapshot.preview =
        PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
    snapshot.overlays = RenderOverlayExtract::default();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, PBR_MATRIX_OUTPUT_SIZE).unwrap();
    let _ = fs::remove_dir_all(root);
    frame
}

fn write_uv_sphere_zmesh(path: PathBuf, rings: usize, segments: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let rings = rings.max(3);
    let segments = segments.max(6);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
    let mut colors = Vec::new();
    let mut joint_indices = Vec::new();
    let mut joint_weights = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = theta.cos();
        let radius = theta.sin();
        for segment in 0..=segments {
            let phi = std::f32::consts::TAU * segment as f32 / segments as f32;
            let x = radius * phi.cos();
            let z = radius * phi.sin();
            positions.push([x, y, z]);
            normals.push([x, y, z]);
            uvs.push([segment as f32 / segments as f32, ring as f32 / rings as f32]);
            tangents.push([-phi.sin(), 0.0, phi.cos(), 1.0]);
            colors.push([1.0, 1.0, 1.0, 1.0]);
            joint_indices.push([0, 0, 0, 0]);
            joint_weights.push([0.0, 0.0, 0.0, 0.0]);
        }
    }
    for ring in 0..rings {
        for segment in 0..segments {
            let a = (ring * (segments + 1) + segment) as u32;
            let b = a + 1;
            let c = a + (segments + 1) as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    let mut attributes = BTreeMap::new();
    attributes.insert(
        MESH_ATTRIBUTE_POSITION.to_string(),
        MeshAttributeValues::Float32x3(positions),
    );
    attributes.insert(
        MESH_ATTRIBUTE_NORMAL.to_string(),
        MeshAttributeValues::Float32x3(normals),
    );
    attributes.insert(
        MESH_ATTRIBUTE_UV0.to_string(),
        MeshAttributeValues::Float32x2(uvs),
    );
    attributes.insert(
        MESH_ATTRIBUTE_TANGENT.to_string(),
        MeshAttributeValues::Float32x4(tangents),
    );
    attributes.insert(
        MESH_ATTRIBUTE_COLOR.to_string(),
        MeshAttributeValues::Float32x4(colors),
    );
    attributes.insert(
        MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
        MeshAttributeValues::Uint16x4(joint_indices),
    );
    attributes.insert(
        MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
        MeshAttributeValues::Float32x4(joint_weights),
    );

    let document = ZMeshDocument {
        version: ZMESH_DOCUMENT_VERSION,
        name: Some("PBR matrix sphere".to_string()),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U32(indices)),
        asset_usage: MeshAssetUsage::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    fs::write(path, document.to_toml_string().unwrap()).unwrap();
}

fn write_pbr_matrix_material(path: PathBuf, metallic: f32, smoothness: f32) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let roughness = (1.0 - smoothness).clamp(0.04, 1.0);
    let mut material = MaterialAsset {
        name: Some(format!(
            "PBR Matrix M{:.3} S{:.3}",
            metallic.clamp(0.0, 1.0),
            smoothness.clamp(0.0, 1.0)
        )),
        shader: asset_reference("builtin://shader/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.78, 0.74, 0.66, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic,
        roughness,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
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
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn write_pbr_matrix_scene(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut entities = Vec::with_capacity(PBR_MATRIX_DIMENSION * PBR_MATRIX_DIMENSION + 2);
    entities.push(SceneEntityAsset {
        entity: 1,
        name: "Camera".to_string(),
        parent: None,
        transform: TransformAsset {
            translation: [0.0, 0.0, 8.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Dynamic,
        camera: Some(SceneCameraAsset {
            projection_mode: ProjectionMode::Orthographic,
            ortho_size: PBR_MATRIX_ORTHO_SIZE,
            z_near: 0.1,
            z_far: 100.0,
            post_process_settings: None,
            ..SceneCameraAsset::default()
        }),
        mesh: None,
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    });
    entities.push(SceneEntityAsset {
        entity: 2,
        name: "Key Light".to_string(),
        parent: None,
        transform: TransformAsset {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Dynamic,
        camera: None,
        mesh: None,
        ambient_light: None,
        directional_light: Some(SceneDirectionalLightAsset {
            direction: [-0.35, -0.55, -0.76],
            color: [1.0, 0.96, 0.88],
            intensity: 1.25,
        }),
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    });

    let mut entity_id = 10_u64;
    for row in 0..PBR_MATRIX_DIMENSION {
        for column in 0..PBR_MATRIX_DIMENSION {
            entities.push(SceneEntityAsset {
                entity: entity_id,
                name: format!(
                    "PBR M{:.2} S{:.2}",
                    pbr_matrix_axis_value(column),
                    pbr_matrix_axis_value(row)
                ),
                parent: None,
                transform: TransformAsset {
                    translation: [pbr_matrix_world_x(column), pbr_matrix_world_y(row), 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [
                        PBR_MATRIX_SPHERE_SCALE,
                        PBR_MATRIX_SPHERE_SCALE,
                        PBR_MATRIX_SPHERE_SCALE,
                    ],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/pbr_matrix_sphere.zmesh"),
                    mesh: None,
                    material: asset_reference(&format!(
                        "res://materials/pbr_matrix_r{row}_c{column}.zmaterial"
                    )),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            });
            entity_id += 1;
        }
    }

    fs::write(path, SceneAsset { entities }.to_toml_string().unwrap()).unwrap();
}

fn polyhaven_lakes_sampled_environment() -> SampledEquirectangularEnvironment {
    let path = shader_test_output_dir()
        .join("assets")
        .join("polyhaven_lakes_1k.hdr");
    let bytes = fs::read(&path).expect("read Poly Haven lakes HDRI");
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)
        .expect("decode Poly Haven lakes HDRI")
        .to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let samples =
        zircon_runtime::core::framework::render::build_sampled_equirect_mip_chain(|x, y| {
            let u = (x as f32 + 0.5)
                / zircon_runtime::core::framework::render::SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH
                    as f32;
            let v = (y as f32 + 0.5)
                / zircon_runtime::core::framework::render::SAMPLED_EQUIRECT_ENVIRONMENT_BASE_HEIGHT
                    as f32;
            tone_map_hdr_sample(sample_hdri_bilinear(&image, u, v), exposure)
        });

    let mut environment =
        SampledEquirectangularEnvironment::new(samples, 1, source_hash_words(&bytes));
    environment.intensity = 1.45;
    environment.rotation_radians = 0.0;
    environment
}

fn sample_hdri_bilinear(image: &image::Rgb32FImage, u: f32, v: f32) -> [f32; 3] {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let texel_x = u.fract() * width as f32 - 0.5;
    let texel_y = v.clamp(0.0, 1.0) * height as f32 - 0.5;
    let x0 = texel_x.floor() as i32;
    let y0 = texel_y.floor() as i32;
    let tx = texel_x - texel_x.floor();
    let ty = texel_y - texel_y.floor();
    let x0u = ((x0 % width as i32 + width as i32) % width as i32) as u32;
    let x1u = (x0u + 1) % width;
    let y0u = (y0.clamp(0, height.saturating_sub(1) as i32)) as u32;
    let y1u = (y0u + 1).min(height - 1);
    let c00 = image.get_pixel(x0u, y0u).0;
    let c10 = image.get_pixel(x1u, y0u).0;
    let c01 = image.get_pixel(x0u, y1u).0;
    let c11 = image.get_pixel(x1u, y1u).0;
    [
        lerp(lerp(c00[0], c10[0], tx), lerp(c01[0], c11[0], tx), ty),
        lerp(lerp(c00[1], c10[1], tx), lerp(c01[1], c11[1], tx), ty),
        lerp(lerp(c00[2], c10[2], tx), lerp(c01[2], c11[2], tx), ty),
    ]
}

fn sampled_hdri_exposure(image: &image::Rgb32FImage) -> f32 {
    let step_x = (image.width() / 128).max(1);
    let step_y = (image.height() / 64).max(1);
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    let mut y = 0;
    while y < image.height() {
        let mut x = 0;
        while x < image.width() {
            sum += luma(image.get_pixel(x, y).0);
            count += 1.0;
            x += step_x;
        }
        y += step_y;
    }
    (0.45 / (sum / count.max(1.0)).max(0.0001)).clamp(0.02, 4.0)
}

fn tone_map_hdr_sample(rgb: [f32; 3], exposure: f32) -> [f32; 4] {
    let mapped = rgb.map(|channel| {
        let exposed = (channel.max(0.0) * exposure).min(64.0);
        exposed / (1.0 + exposed)
    });
    [mapped[0], mapped[1], mapped[2], 1.0]
}

fn source_hash_words(bytes: &[u8]) -> [u32; 4] {
    let mut state = [0x811c9dc5_u32, 0x9e3779b9, 0x85ebca6b, 0xc2b2ae35];
    for (index, byte) in bytes.iter().enumerate() {
        let slot = index & 3;
        state[slot] ^= u32::from(*byte);
        state[slot] = state[slot].wrapping_mul(16_777_619);
    }
    state
}

fn assert_real_hdri_reflection_response(frame: &zircon_runtime::graphics::ViewportFrame) {
    let upper_sky = average_region_rgb(frame, 40, 32, 96, 96);
    let lower_sky = average_region_rgb(frame, 40, frame.height.saturating_sub(128), 96, 96);
    let smooth_dielectric = pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, 0);
    let smooth_metal =
        pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, PBR_MATRIX_DIMENSION - 1);
    let rough_metal = pbr_matrix_cell_rgb(frame, 0, PBR_MATRIX_DIMENSION - 1);

    assert!(
        color_distance(upper_sky, lower_sky) > 8.0,
        "real HDRI skybox should show directional scene variation, upper={upper_sky:?}, lower={lower_sky:?}"
    );
    assert!(
        color_distance(smooth_metal, smooth_dielectric) > 4.0,
        "smooth metallic cells should visibly differ from dielectric cells under real HDRI, metal={smooth_metal:?}, dielectric={smooth_dielectric:?}"
    );
    assert!(
        color_distance(smooth_metal, rough_metal) > 2.0,
        "smoothness should change real HDRI reflection response, smooth={smooth_metal:?}, rough={rough_metal:?}"
    );
}

fn pbr_matrix_cell_rgb(
    frame: &zircon_runtime::graphics::ViewportFrame,
    row: usize,
    column: usize,
) -> [f32; 3] {
    let aspect = frame.width as f32 / frame.height as f32;
    let half_height = PBR_MATRIX_ORTHO_SIZE;
    let half_width = half_height * aspect;
    let center_x = ((pbr_matrix_world_x(column) + half_width) / (half_width * 2.0)
        * frame.width as f32)
        .round()
        .clamp(0.0, frame.width.saturating_sub(1) as f32) as u32;
    let center_y = ((half_height - pbr_matrix_world_y(row)) / (half_height * 2.0)
        * frame.height as f32)
        .round()
        .clamp(0.0, frame.height.saturating_sub(1) as f32) as u32;
    average_region_rgb(
        frame,
        center_x.saturating_sub(20),
        center_y.saturating_sub(20),
        40,
        40,
    )
}

fn average_region_rgb(
    frame: &zircon_runtime::graphics::ViewportFrame,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> [f32; 3] {
    let x_end = x.saturating_add(width).min(frame.width);
    let y_end = y.saturating_add(height).min(frame.height);
    let frame_width = frame.width as usize;
    let mut sum = [0.0_f32; 3];
    let mut count = 0.0_f32;
    for py in y as usize..y_end as usize {
        for px in x as usize..x_end as usize {
            let index = (py * frame_width + px) * 4;
            sum[0] += frame.rgba[index] as f32;
            sum[1] += frame.rgba[index + 1] as f32;
            sum[2] += frame.rgba[index + 2] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [sum[0] / count, sum[1] / count, sum[2] / count]
    }
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn pbr_matrix_axis_value(index: usize) -> f32 {
    index as f32 / (PBR_MATRIX_DIMENSION - 1) as f32
}

fn pbr_matrix_world_x(column: usize) -> f32 {
    (column as f32 - (PBR_MATRIX_DIMENSION as f32 - 1.0) * 0.5) * PBR_MATRIX_STEP_X
}

fn pbr_matrix_world_y(row: usize) -> f32 {
    ((PBR_MATRIX_DIMENSION as f32 - 1.0) * 0.5 - row as f32) * PBR_MATRIX_STEP_Y
}

fn shader_test_output_dir() -> PathBuf {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader");
    fs::create_dir_all(&output_dir).unwrap();
    output_dir
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    static NEXT_TEMP_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let process_id = std::process::id();
    let sequence = NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon_graphics_{label}_{process_id}_{sequence}_{unique}"
    ))
}

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn color_distance(first: [f32; 3], second: [f32; 3]) -> f32 {
    let dr = first[0] - second[0];
    let dg = first[1] - second[1];
    let db = first[2] - second[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
