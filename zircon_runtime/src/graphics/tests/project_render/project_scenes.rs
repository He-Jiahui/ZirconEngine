use super::*;
use crate::graphics::ViewportFrame;

const PBR_MATRIX_DIMENSION: usize = 8;
const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1280, 960);
const PBR_MATRIX_ORTHO_SIZE: f32 = 6.4;
const PBR_MATRIX_STEP_X: f32 = 0.74;
const PBR_MATRIX_STEP_Y: f32 = 0.68;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.27;

#[test]
fn directory_project_scene_renders_non_background_frame_with_gizmo_overlay() {
    let root = unique_temp_project_root("graphics_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material(
        paths.assets_root().join("materials").join("grid.zmaterial"),
        "res://shaders/pbr.wgsl",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/grid.zmaterial",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(320, 240)),
        virtual_geometry_debug: None,
    });

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, UVec2::new(320, 240)).unwrap();

    let background = [20_u8, 23_u8, 28_u8, 255_u8];
    assert!(frame.rgba.chunks_exact(4).any(|pixel| pixel != background));
    assert!(frame
        .rgba
        .chunks_exact(4)
        .any(|pixel| { pixel[3] == 255 && (pixel[0] > 200 || pixel[1] > 200 || pixel[2] > 200) }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn example_vampire_scene_renders_visible_mesh_pixels() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("examples")
        .join("vampire");
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let mut snapshot = world.to_render_snapshot();
    snapshot.preview.skybox_enabled = false;
    snapshot.preview.fallback_skybox = FallbackSkyboxKind::None;
    snapshot.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, UVec2::new(320, 240)).unwrap();
    let background: [u8; 4] = frame.rgba[..4].try_into().unwrap();
    let visible_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| **pixel != background)
        .count();

    assert!(
        visible_pixels > 256,
        "expected vampire example meshes to draw visible pixels, found {visible_pixels}"
    );
}

#[test]
#[ignore = "manual screenshot export for the vampire example"]
fn export_example_vampire_scene_png() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("examples")
        .join("vampire");
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
        virtual_geometry_debug: None,
    });
    snapshot.preview.skybox_enabled = false;
    snapshot.preview.fallback_skybox = FallbackSkyboxKind::None;
    snapshot.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, UVec2::new(1280, 720)).unwrap();
    let output =
        shader_test_output_dir().join("runtime_shader_material_vampire_offscreen_20260703.png");
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba)
        .expect("rendered frame should match output image dimensions")
        .save_with_format(&output, ImageFormat::Png)
        .expect("write vampire screenshot");
}

#[test]
#[ignore = "manual screenshot export for the runtime material sphere"]
fn export_runtime_shader_material_sphere_png() {
    let root = unique_temp_project_root("graphics_material_sphere");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsMaterialSphere",
        AssetUri::parse("res://scenes/material_sphere.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_dir = paths.assets_root().join("shaders").join("material_sphere");
    write_compound_shader_meta(&paths, "res://shaders/material_sphere", "material_sphere");
    write_material_sphere_zshader(shader_dir.join("material_sphere.zshader"));
    write_material_sphere_wgsl(shader_dir.join("material_sphere.wgsl"));
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("sphere_albedo.png"),
        [255, 226, 196, 255],
    );
    write_uv_sphere_obj(
        paths
            .assets_root()
            .join("models")
            .join("material_sphere.obj"),
        32,
        64,
    );
    write_material_with_base_color_and_texture(
        paths
            .assets_root()
            .join("materials")
            .join("material_sphere.zmaterial"),
        "res://shaders/material_sphere",
        [1.0, 0.7, 0.5, 1.0],
        "res://textures/sphere_albedo.png",
    );
    write_material_sphere_scene(
        paths
            .assets_root()
            .join("scenes")
            .join("material_sphere.scene.toml"),
        "res://materials/material_sphere.zmaterial",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/material_sphere.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    match project
        .load_artifact(&AssetUri::parse("res://shaders/material_sphere").unwrap())
        .unwrap()
    {
        crate::asset::ImportedAsset::Shader(shader) => {
            assert_eq!(shader.kind, ShaderAssetKind::Surface);
            assert!(shader.source.contains("material_sphere_color"));
        }
        other => panic!("expected material sphere shader artifact, found {other:?}"),
    }
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let output_size = UVec2::new(1024, 1024);
    let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(output_size),
        virtual_geometry_debug: None,
    });
    snapshot.preview.skybox_enabled = false;
    snapshot.preview.fallback_skybox = FallbackSkyboxKind::None;
    snapshot.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, output_size).unwrap();
    let background: [u8; 4] = frame.rgba[..4].try_into().unwrap();
    let visible_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| **pixel != background)
        .count();
    let luma_range = visible_luma_range(&frame, background).unwrap_or((0.0, 0.0));

    let output =
        shader_test_output_dir().join("runtime_shader_material_sphere_offscreen_20260703.png");
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba)
        .expect("rendered frame should match output image dimensions")
        .save_with_format(&output, ImageFormat::Png)
        .expect("write material sphere screenshot");

    assert!(
        visible_pixels > 12_000,
        "expected material sphere to draw visible pixels, found {visible_pixels}"
    );
    assert!(
        luma_range.1 - luma_range.0 > 18.0,
        "expected material sphere shading gradient, luma min {:.2}, max {:.2}",
        luma_range.0,
        luma_range.1
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "manual screenshot export for the runtime PBR metallic/smoothness matrix"]
fn export_runtime_shader_pbr_metallic_smoothness_matrix_png() {
    let frame = render_pbr_matrix_frame_with_environment(
        "graphics_pbr_matrix",
        "GraphicsPbrMatrix",
        EnvironmentExtract::procedural_default(),
    );
    let output = shader_test_output_dir()
        .join("runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png");
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("rendered PBR matrix frame should match output image dimensions")
        .save_with_format(&output, ImageFormat::Png)
        .expect("write PBR metallic/smoothness matrix screenshot");
    assert_pbr_matrix_environment_response(&frame);
}

#[test]
#[ignore = "manual screenshot export for the runtime PBR real HDRI reflection matrix"]
fn export_runtime_shader_pbr_real_hdri_reflection_png() {
    let frame = render_pbr_matrix_frame_with_environment(
        "graphics_pbr_real_hdri",
        "GraphicsPbrRealHdri",
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
    temp_label: &str,
    project_name: &str,
    environment: EnvironmentExtract,
) -> ViewportFrame {
    let root = unique_temp_project_root(temp_label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        project_name,
        AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_uv_sphere_obj(
        paths
            .assets_root()
            .join("models")
            .join("pbr_matrix_sphere.obj"),
        24,
        48,
    );
    for row in 0..PBR_MATRIX_DIMENSION {
        for column in 0..PBR_MATRIX_DIMENSION {
            let metallic = pbr_matrix_axis_value(column);
            let smoothness = pbr_matrix_axis_value(row);
            write_pbr_matrix_material(
                paths
                    .assets_root()
                    .join("materials")
                    .join(format!("pbr_matrix_r{row}_c{column}.zmaterial")),
                metallic,
                smoothness,
            );
        }
    }
    write_pbr_matrix_scene(
        paths
            .assets_root()
            .join("scenes")
            .join("pbr_matrix.scene.toml"),
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();

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
    snapshot.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, PBR_MATRIX_OUTPUT_SIZE).unwrap();

    let _ = fs::remove_dir_all(root);
    frame
}

#[test]
fn directory_project_material_shader_drives_pipeline_color_output() {
    let root = unique_temp_project_root("graphics_shader_pipeline");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsShaderPipeline",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(paths.assets_root().join("shaders").join("flat_green.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/flat_green.zmaterial",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut snapshot = world.to_render_snapshot();
    snapshot.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, UVec2::new(320, 240)).unwrap();

    let green_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            pixel[3] == 255
                && pixel[1] > 160
                && pixel[1] > pixel[0] + 50
                && pixel[1] > pixel[2] + 30
        })
        .count();
    assert!(
        green_pixels > 32,
        "expected project shader to dominate visible pixels, found {green_pixels}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn wire_only_mode_reduces_filled_surface_pixels() {
    let root = unique_temp_project_root("graphics_wire_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsWireOnly",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(paths.assets_root().join("shaders").join("flat_green.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material(
        paths.assets_root().join("materials").join("grid.zmaterial"),
        "res://shaders/flat_green.wgsl",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/grid.zmaterial",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut shaded = world.to_render_snapshot();
    shaded.preview.skybox_enabled = false;
    shaded.preview.fallback_skybox = FallbackSkyboxKind::None;
    shaded.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut wire_only = shaded.clone();
    wire_only.overlays.display_mode = DisplayMode::WireOnly;

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let shaded_frame = renderer.render(shaded, UVec2::new(320, 240)).unwrap();
    let wire_frame = renderer.render(wire_only, UVec2::new(320, 240)).unwrap();

    let background: [u8; 4] = wire_frame.rgba[..4].try_into().unwrap();
    let shaded_surface_pixels = shaded_frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != background)
        .count();
    let wire_surface_pixels = wire_frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != background)
        .count();

    assert!(
        shaded_surface_pixels > 0 && wire_surface_pixels < shaded_surface_pixels,
        "wire-only mode should suppress most filled surface pixels ({wire_surface_pixels} vs {shaded_surface_pixels})"
    );

    let _ = fs::remove_dir_all(root);
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
                    model: asset_reference("res://models/pbr_matrix_sphere.obj"),
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

fn pbr_matrix_axis_value(index: usize) -> f32 {
    index as f32 / (PBR_MATRIX_DIMENSION - 1) as f32
}

fn pbr_matrix_world_x(column: usize) -> f32 {
    (column as f32 - (PBR_MATRIX_DIMENSION as f32 - 1.0) * 0.5) * PBR_MATRIX_STEP_X
}

fn pbr_matrix_world_y(row: usize) -> f32 {
    ((PBR_MATRIX_DIMENSION as f32 - 1.0) * 0.5 - row as f32) * PBR_MATRIX_STEP_Y
}

fn assert_pbr_matrix_environment_response(frame: &ViewportFrame) {
    let top_left_sky = average_region_rgb(frame, 24, 24, 80, 80);
    assert!(
        top_left_sky[2] > top_left_sky[0] + 16.0,
        "procedural skybox should render a cool sky background, sampled RGB={top_left_sky:?}"
    );

    let center_rough_dielectric = pbr_matrix_cell_rgb(frame, 0, 0);
    let center_smooth_dielectric = pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, 0);
    let center_smooth_metal =
        pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, PBR_MATRIX_DIMENSION - 1);
    let center_rough_metal = pbr_matrix_cell_rgb(frame, 0, PBR_MATRIX_DIMENSION - 1);
    let smooth_metal_blue_bias = center_smooth_metal[2] - center_smooth_metal[0];
    let smooth_dielectric_blue_bias = center_smooth_dielectric[2] - center_smooth_dielectric[0];
    let metal_smooth_delta = luma(center_smooth_metal) - luma(center_rough_metal);

    assert!(
        center_rough_dielectric
            .iter()
            .any(|channel| *channel > 24.0),
        "rough dielectric matrix cell should be visible, sampled RGB={center_rough_dielectric:?}"
    );
    assert!(
        smooth_metal_blue_bias > smooth_dielectric_blue_bias + 4.0,
        "high metallic + smooth cell should pick up stronger sky reflection: smooth metal RGB={center_smooth_metal:?}, smooth dielectric RGB={center_smooth_dielectric:?}"
    );
    assert!(
        metal_smooth_delta > 2.0,
        "metal smoothness ramp should change reflected brightness: smooth metal RGB={center_smooth_metal:?}, rough metal RGB={center_rough_metal:?}"
    );
}

fn assert_real_hdri_reflection_response(frame: &ViewportFrame) {
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

fn polyhaven_lakes_sampled_environment() -> SampledEquirectangularEnvironment {
    let path = shader_test_asset_dir().join("polyhaven_lakes_1k.hdr");
    let bytes = fs::read(&path).expect("read Poly Haven lakes HDRI");
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)
        .expect("decode Poly Haven lakes HDRI")
        .to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let samples = crate::core::framework::render::build_sampled_equirect_mip_chain(|x, y| {
        let u = (x as f32 + 0.5)
            / crate::core::framework::render::SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH as f32;
        let v = (y as f32 + 0.5)
            / crate::core::framework::render::SAMPLED_EQUIRECT_ENVIRONMENT_BASE_HEIGHT as f32;
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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
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

fn pbr_matrix_cell_rgb(frame: &ViewportFrame, row: usize, column: usize) -> [f32; 3] {
    let aspect = frame.width as f32 / frame.height as f32;
    // ViewProjectionMatrixPair treats orthographic size as camera half-height.
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

fn average_region_rgb(frame: &ViewportFrame, x: u32, y: u32, width: u32, height: u32) -> [f32; 3] {
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

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn color_distance(first: [f32; 3], second: [f32; 3]) -> f32 {
    let dr = first[0] - second[0];
    let dg = first[1] - second[1];
    let db = first[2] - second[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

fn shader_test_asset_dir() -> PathBuf {
    let asset_dir = shader_test_output_dir().join("assets");
    fs::create_dir_all(&asset_dir).unwrap();
    asset_dir
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

fn visible_luma_range(frame: &ViewportFrame, background: [u8; 4]) -> Option<(f32, f32)> {
    let mut min_luma = f32::INFINITY;
    let mut max_luma = f32::NEG_INFINITY;
    for pixel in frame.rgba.chunks_exact(4) {
        if pixel == background {
            continue;
        }
        let luma = 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
        min_luma = min_luma.min(luma);
        max_luma = max_luma.max(luma);
    }

    if min_luma.is_finite() && max_luma.is_finite() {
        Some((min_luma, max_luma))
    } else {
        None
    }
}
