use super::*;
use crate::graphics::ViewportFrame;

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
