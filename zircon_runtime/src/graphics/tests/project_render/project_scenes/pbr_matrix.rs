use super::*;

const PBR_MATRIX_DIMENSION: usize = 8;
pub(super) const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1280, 960);
const PBR_MATRIX_ORTHO_SIZE: f32 = 6.4;
const PBR_MATRIX_PRODUCT_CAMERA_Z: f32 = 8.0;
const PBR_MATRIX_PRODUCT_FOV_Y_RADIANS: f32 = 1.047_197_6;
const PBR_MATRIX_STEP_X: f32 = 0.74;
const PBR_MATRIX_STEP_Y: f32 = 0.68;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.27;

pub(super) fn plan11_ibl_product_capture_quality_profile(
) -> crate::core::framework::render::RenderQualityProfile {
    crate::core::framework::render::RenderQualityProfile::new("plan11-ibl-wgpu-capture")
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_clustered_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(false)
        .with_solari(false)
}

pub(super) fn build_pbr_matrix_product_capture_snapshot(
    world: &World,
    environment: EnvironmentExtract,
) -> crate::core::framework::render::SceneViewportRenderPacket {
    let mut snapshot = build_pbr_matrix_snapshot(world, environment);
    snapshot.scene.camera.projection_mode = ProjectionMode::Perspective;
    snapshot.scene.camera.fov_y_radians = PBR_MATRIX_PRODUCT_FOV_Y_RADIANS;
    snapshot.scene.camera.z_near = 0.1;
    snapshot.scene.camera.z_far = 100.0;
    snapshot
        .scene
        .camera
        .apply_viewport_size(PBR_MATRIX_OUTPUT_SIZE);
    snapshot
}

pub(super) fn render_pbr_matrix_frame_with_environment(
    temp_label: &str,
    project_name: &str,
    environment: EnvironmentExtract,
) -> ViewportFrame {
    let root = unique_temp_project_root(temp_label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    write_pbr_matrix_project(&paths, project_name);

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let snapshot = build_pbr_matrix_snapshot(&world, environment);

    let frame = render_snapshot_through_framework(asset_manager, snapshot, PBR_MATRIX_OUTPUT_SIZE);

    let _ = fs::remove_dir_all(root);
    frame
}

pub(super) fn write_pbr_matrix_project(paths: &ProjectPaths, project_name: &str) {
    ProjectManifest::new(
        project_name,
        AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_uv_sphere_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
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
                    .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                    .join("materials")
                    .join(format!("pbr_matrix_r{row}_c{column}.zmaterial")),
                metallic,
                smoothness,
            );
        }
    }
    write_pbr_matrix_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("pbr_matrix.scene.toml"),
    );
}

pub(super) fn build_pbr_matrix_snapshot(
    world: &World,
    environment: EnvironmentExtract,
) -> crate::core::framework::render::SceneViewportRenderPacket {
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
    snapshot
}

fn write_pbr_matrix_material(path: PathBuf, metallic: f32, smoothness: f32) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let roughness = (1.0 - smoothness).clamp(
        crate::core::framework::render::STANDARD_MATERIAL_MIN_ROUGHNESS,
        1.0,
    );
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
            translation: [0.0, 0.0, PBR_MATRIX_PRODUCT_CAMERA_Z],
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
            volumetric: false,
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

pub(super) fn assert_pbr_matrix_environment_response(frame: &ViewportFrame) {
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

pub(super) fn assert_real_hdri_reflection_response(frame: &ViewportFrame) {
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

fn assert_source_cubemap_product_response(frame: &ViewportFrame) {
    let upper_sky = average_region_rgb(frame, 40, 32, 96, 96);
    let lower_sky = average_region_rgb(frame, 40, frame.height.saturating_sub(128), 96, 96);
    let smooth_dielectric = pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, 0);
    let smooth_metal =
        pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, PBR_MATRIX_DIMENSION - 1);
    let rough_metal = pbr_matrix_cell_rgb(frame, 0, PBR_MATRIX_DIMENSION - 1);

    assert!(
        color_distance(upper_sky, lower_sky) > 5.0,
        "source cubemap skybox should show directional variation, upper={upper_sky:?}, lower={lower_sky:?}"
    );
    assert!(
        color_distance(smooth_metal, smooth_dielectric) > 3.0,
        "source cubemap metallic cells should visibly differ from dielectric cells, metal={smooth_metal:?}, dielectric={smooth_dielectric:?}"
    );
    assert!(
        color_distance(smooth_metal, rough_metal) > 1.5,
        "source cubemap smoothness should change reflection response, smooth={smooth_metal:?}, rough={rough_metal:?}"
    );
}

pub(super) fn assert_source_cubemap_product_capture_response(frame: &CapturedFrame) {
    let upper_sky = average_region_rgb_captured(frame, 40, 32, 96, 96);
    let lower_sky =
        average_region_rgb_captured(frame, 40, frame.height.saturating_sub(128), 96, 96);
    let smooth_dielectric =
        pbr_matrix_perspective_capture_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, 0);
    let smooth_metal = pbr_matrix_perspective_capture_cell_rgb(
        frame,
        PBR_MATRIX_DIMENSION - 1,
        PBR_MATRIX_DIMENSION - 1,
    );
    let rough_metal = pbr_matrix_perspective_capture_cell_rgb(frame, 0, PBR_MATRIX_DIMENSION - 1);

    assert!(
        color_distance(upper_sky, lower_sky) > 5.0,
        "source cubemap Wgpu capture skybox should show directional variation, upper={upper_sky:?}, lower={lower_sky:?}"
    );
    assert!(
        color_distance(smooth_metal, smooth_dielectric) > 3.0,
        "source cubemap Wgpu capture metallic cells should visibly differ from dielectric cells, metal={smooth_metal:?}, dielectric={smooth_dielectric:?}"
    );
    assert!(
        color_distance(smooth_metal, rough_metal) > 1.5,
        "source cubemap Wgpu capture smoothness should change reflection response, smooth={smooth_metal:?}, rough={rough_metal:?}"
    );
}

pub(super) fn runtime_ibl_cache_source_cubemap_environment() -> SourceCubemapEnvironment {
    let mip_chain = build_source_cubemap_from_equirect(64, |u, v| {
        let horizon = (1.0 - (v - 0.56).abs() * 2.4).clamp(0.0, 1.0);
        let sky = (1.0 - v).clamp(0.0, 1.0);
        let sun_u = ((u - 0.18).abs()).min((u + 0.82).abs());
        let sun = (1.0 - (sun_u * 26.0 + (v - 0.36).abs() * 16.0)).clamp(0.0, 1.0);
        [
            0.05 + sky * 0.16 + horizon * 0.28 + sun * 1.8,
            0.10 + sky * 0.34 + horizon * 0.24 + sun * 1.25,
            0.18 + sky * 0.62 + horizon * 0.10 + sun * 0.38,
            1.0,
        ]
    });
    let irradiance_cube = build_source_cubemap_irradiance_cube(&mip_chain);
    let mut environment = SourceCubemapEnvironment::new(
        mip_chain,
        20260707,
        [0x706c_6e31, 0x6962_6c32, 0x6470_3030, 0x3030_0001],
    )
    .with_irradiance_cube(irradiance_cube);
    environment.intensity = 1.35;
    environment.rotation_radians = 0.15;
    environment
}

pub(super) fn ibl_executor_count(executor_ids: &[String]) -> usize {
    executor_ids
        .iter()
        .filter(|executor_id| is_ibl_executor_id(executor_id.as_str()))
        .count()
}

fn is_ibl_executor_id(executor_id: &str) -> bool {
    matches!(
        executor_id,
        IBL_BAKE_PMREM_EXECUTOR_ID
            | IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID
            | IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID
    )
}

pub(super) fn polyhaven_lakes_source_cubemap_environment() -> SourceCubemapEnvironment {
    let path = shader_test_asset_dir().join("polyhaven_lakes_1k.hdr");
    let bytes = fs::read(&path).expect("read Poly Haven lakes HDRI");
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)
        .expect("decode Poly Haven lakes HDRI")
        .to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let face_size = crate::core::framework::render::source_cubemap_face_size_from_equirect_height(
        image.height(),
    );
    let mip_chain =
        crate::core::framework::render::build_source_cubemap_from_equirect(face_size, |u, v| {
            expose_hdr_sample(sample_hdri_bilinear(&image, u, v), exposure)
        });
    let irradiance_cube =
        crate::core::framework::render::build_source_cubemap_irradiance_cube(&mip_chain);

    let mut environment = SourceCubemapEnvironment::new(mip_chain, 1, source_hash_words(&bytes))
        .with_irradiance_cube(irradiance_cube);
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

fn expose_hdr_sample(rgb: [f32; 3], exposure: f32) -> [f32; 4] {
    let exposed = rgb.map(|channel| (channel.max(0.0) * exposure).min(65_504.0));
    [exposed[0], exposed[1], exposed[2], 1.0]
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

fn pbr_matrix_perspective_capture_cell_rgb(
    frame: &CapturedFrame,
    row: usize,
    column: usize,
) -> [f32; 3] {
    let aspect = frame.width as f32 / frame.height as f32;
    let half_height = (PBR_MATRIX_PRODUCT_FOV_Y_RADIANS * 0.5).tan() * PBR_MATRIX_PRODUCT_CAMERA_Z;
    let half_width = half_height * aspect;
    let center_x = ((pbr_matrix_world_x(column) + half_width) / (half_width * 2.0)
        * frame.width as f32)
        .round()
        .clamp(0.0, frame.width.saturating_sub(1) as f32) as u32;
    let center_y = ((half_height - pbr_matrix_world_y(row)) / (half_height * 2.0)
        * frame.height as f32)
        .round()
        .clamp(0.0, frame.height.saturating_sub(1) as f32) as u32;
    average_region_rgb_captured(
        frame,
        center_x.saturating_sub(20),
        center_y.saturating_sub(20),
        40,
        40,
    )
}

fn average_region_rgb(frame: &ViewportFrame, x: u32, y: u32, width: u32, height: u32) -> [f32; 3] {
    average_region_rgb_pixels(&frame.rgba, frame.width, frame.height, x, y, width, height)
}

fn average_region_rgb_captured(
    frame: &CapturedFrame,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> [f32; 3] {
    average_region_rgb_pixels(&frame.rgba, frame.width, frame.height, x, y, width, height)
}

fn average_region_rgb_pixels(
    rgba: &[u8],
    frame_width: u32,
    frame_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> [f32; 3] {
    let x_end = x.saturating_add(width).min(frame_width);
    let y_end = y.saturating_add(height).min(frame_height);
    let frame_width = frame_width as usize;
    let mut sum = [0.0_f32; 3];
    let mut count = 0.0_f32;
    for py in y as usize..y_end as usize {
        for px in x as usize..x_end as usize {
            let index = (py * frame_width + px) * 4;
            sum[0] += rgba[index] as f32;
            sum[1] += rgba[index + 1] as f32;
            sum[2] += rgba[index + 2] as f32;
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

pub(super) fn shader_test_output_dir() -> PathBuf {
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

pub(super) fn render_test_output_dir() -> PathBuf {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render");
    fs::create_dir_all(&output_dir).unwrap();
    output_dir
}

pub(super) fn visible_luma_range(frame: &ViewportFrame, background: [u8; 4]) -> Option<(f32, f32)> {
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
