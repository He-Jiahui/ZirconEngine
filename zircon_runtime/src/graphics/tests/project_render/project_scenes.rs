use super::*;
use crate::asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch;
use crate::core::framework::render::{
    IblBakeArtifactContents, build_source_cubemap_from_equirect,
    build_source_cubemap_irradiance_cube,
};
use crate::core::framework::render::{RenderFramework, RenderViewportDescriptor};
use crate::graphics::scene::{
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
    IBL_BAKE_PMREM_EXECUTOR_ID,
};
use crate::graphics::{ViewportFrame, ViewportRenderFrame, WgpuRenderFramework};

mod pbr_matrix;
mod reflection_probe_product;

use pbr_matrix::{
    PBR_MATRIX_OUTPUT_SIZE, assert_pbr_matrix_environment_response,
    assert_real_hdri_reflection_response, assert_source_cubemap_product_capture_response,
    build_pbr_matrix_product_capture_snapshot, ibl_executor_count,
    plan11_ibl_product_capture_quality_profile, polyhaven_lakes_source_cubemap_environment,
    render_pbr_matrix_frame_with_environment, render_test_output_dir,
    runtime_ibl_cache_source_cubemap_environment, shader_test_output_dir, visible_luma_range,
    write_pbr_matrix_project,
};

fn render_snapshot_through_framework(
    asset_manager: Arc<ProjectAssetManager>,
    snapshot: crate::core::framework::render::RenderSceneSnapshot,
    output_size: UVec2,
) -> ViewportFrame {
    let framework =
        WgpuRenderFramework::new_for_test(asset_manager).expect("project render framework");
    let viewport = framework
        .create_viewport(
            RenderViewportDescriptor::new(output_size).with_label("graphics.project-render"),
        )
        .expect("project render viewport");
    framework
        .submit_runtime_frame(
            viewport,
            ViewportRenderFrame::from_snapshot(snapshot, output_size),
        )
        .expect("submit compiled project render frame");
    let captured = framework
        .capture_frame(viewport)
        .expect("capture compiled project render frame")
        .expect("compiled project render frame should be available");

    ViewportFrame {
        width: captured.width,
        height: captured.height,
        rgba: captured.rgba,
        generation: captured.generation,
        capture_report: captured.capture_report,
    }
}

#[test]
fn project_render_capture_submission_uses_the_compiled_framework_path() {
    let source = include_str!("project_scenes.rs");

    assert!(source.contains(concat!("WgpuRenderFramework", "::new_for_test")));
    assert!(source.contains(concat!("submit_runtime_", "frame(")));
    assert!(source.contains(concat!("capture_", "frame(viewport)")));
    assert!(!source.contains(concat!("SceneRenderer", "::new")));
    assert!(!source.contains(concat!(".render", "(snapshot")));
}

#[test]
fn directory_project_scene_renders_non_background_frame_with_gizmo_overlay() {
    let root = unique_temp_project_root("graphics_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("pbr.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("grid.zmaterial"),
        "res://shaders/pbr.wgsl",
    );
    write_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
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

    let frame = render_snapshot_through_framework(asset_manager, snapshot, UVec2::new(320, 240));

    let background = [20_u8, 23_u8, 28_u8, 255_u8];
    assert!(frame.rgba.chunks_exact(4).any(|pixel| pixel != background));
    assert!(
        frame.rgba.chunks_exact(4).any(|pixel| {
            pixel[3] == 255 && (pixel[0] > 200 || pixel[1] > 200 || pixel[2] > 200)
        })
    );

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

    let frame = render_snapshot_through_framework(asset_manager, snapshot, UVec2::new(320, 240));
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

    let frame = render_snapshot_through_framework(asset_manager, snapshot, UVec2::new(1280, 720));
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
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsMaterialSphere",
        AssetUri::parse("res://scenes/material_sphere.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("material_sphere");
    write_compound_shader_meta(&paths, "res://shaders/material_sphere", "material_sphere");
    write_material_sphere_zshader(shader_dir.join("material_sphere.zshader"));
    write_material_sphere_wgsl(shader_dir.join("material_sphere.wgsl"));
    write_solid_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("sphere_albedo.png"),
        [255, 226, 196, 255],
    );
    write_uv_sphere_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("material_sphere.obj"),
        32,
        64,
    );
    write_material_with_base_color_and_texture(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("material_sphere.zmaterial"),
        "res://shaders/material_sphere",
        [1.0, 0.7, 0.5, 1.0],
        "res://textures/sphere_albedo.png",
    );
    write_material_sphere_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
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

    let frame = render_snapshot_through_framework(asset_manager, snapshot, output_size);
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
        EnvironmentExtract::source_cubemap(polyhaven_lakes_source_cubemap_environment()),
    );
    let output = shader_test_output_dir()
        .join("runtime_shader_pbr_real_hdri_lakes_hdr_pmrem_reflection_20260705.png");
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("rendered real HDRI PBR frame should match output image dimensions")
        .save_with_format(&output, ImageFormat::Png)
        .expect("write real HDRI PBR reflection screenshot");
    assert_real_hdri_reflection_response(&frame);
}

#[test]
#[ignore = "manual product screenshot export for Plan 11 runtime IBL cache second launch"]
fn export_runtime_render_ibl_cache_second_launch_dispatch_zero_png() {
    let root = unique_temp_project_root("graphics_ibl_cache_second_launch");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    write_pbr_matrix_project(&paths, "GraphicsIblCacheSecondLaunch");

    let environment =
        EnvironmentExtract::source_cubemap(runtime_ibl_cache_source_cubemap_environment());
    let request = environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::PMREM_SH9_IEM)
        .expect("source cubemap should request a runtime IBL bake artifact");

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let cache_store = asset_manager
        .ibl_bake_artifact_cache_store()
        .expect("opened project should expose runtime IBL cache storage");
    let cache_path = cache_store.runtime_cache_path(&request);
    let _ = fs::remove_file(&cache_path);

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(
            crate::core::framework::render::RenderViewportDescriptor::new(PBR_MATRIX_OUTPUT_SIZE),
        )
        .unwrap();
    framework
        .set_quality_profile(viewport, plan11_ibl_product_capture_quality_profile())
        .unwrap();
    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(1),
                build_pbr_matrix_product_capture_snapshot(&world, environment.clone()),
            ),
        )
        .unwrap();
    let first_stats = framework.query_stats().unwrap();
    assert_eq!(
        first_stats.last_pipeline,
        Some(crate::core::framework::render::RenderPipelineHandle::new(1)),
        "Plan 11 product Wgpu capture proof must use the Core3D forward pipeline"
    );
    let first_executor_ids = first_stats.last_graph_executed_executor_ids.clone();
    let first_ibl_executor_count = ibl_executor_count(&first_executor_ids);
    let first_compute_dispatch_count = first_stats.last_graph_compute_dispatch_count;
    assert!(
        first_ibl_executor_count > 0,
        "first launch should execute IBL bake executors, ids={first_executor_ids:?}"
    );
    assert!(
        cache_path.is_file(),
        "first launch should write a runtime IBL cache artifact at {}",
        cache_path.display()
    );

    let post_first_dispatch =
        resolve_ibl_bake_artifact_runtime_dispatch(&cache_store, &request, &[])
            .expect("runtime cache dispatch resolution after first launch should succeed");
    assert_eq!(
        post_first_dispatch.environment_compute_dispatch_count(),
        0,
        "cache hit should resolve to zero environment runtime compute dispatches"
    );
    assert!(
        !post_first_dispatch.requires_runtime_compute(),
        "cache hit should not require runtime IBL compute after first launch"
    );

    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(2),
                build_pbr_matrix_product_capture_snapshot(&world, environment),
            ),
        )
        .unwrap();
    let second_stats = framework.query_stats().unwrap();
    assert_eq!(
        second_stats.last_pipeline,
        Some(crate::core::framework::render::RenderPipelineHandle::new(1)),
        "second Plan 11 product Wgpu capture submit must stay on Core3D forward"
    );
    let second_executor_ids = second_stats.last_graph_executed_executor_ids.clone();
    let second_ibl_executor_count = ibl_executor_count(&second_executor_ids);
    let second_compute_dispatch_count = second_stats.last_graph_compute_dispatch_count;
    assert_eq!(
        second_ibl_executor_count, 0,
        "second launch cache hit should omit IBL bake executors, ids={second_executor_ids:?}"
    );

    let second_frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("second product Wgpu frame should be available for capture");
    assert_source_cubemap_product_capture_response(&second_frame);

    let output = render_test_output_dir()
        .join("plan11_ibl_product_wgpu_capture_second_launch_dispatch_zero_20260707.png");
    ImageBuffer::<Rgba<u8>, _>::from_raw(
        second_frame.width,
        second_frame.height,
        second_frame.rgba.clone(),
    )
    .expect("captured IBL second-launch frame should match output image dimensions")
    .save_with_format(&output, ImageFormat::Png)
    .expect("write Plan 11 runtime IBL cache second-launch Wgpu capture screenshot");

    let report = format!(
        "cache_path={}\nfirst_ibl_executor_count={first_ibl_executor_count}\nsecond_ibl_executor_count={second_ibl_executor_count}\nfirst_compute_dispatch_count={first_compute_dispatch_count}\nsecond_compute_dispatch_count={second_compute_dispatch_count}\npost_first_environment_dispatch_count={}\nfirst_pipeline={:?}\nsecond_pipeline={:?}\ncapture_report={:?}\ngraph_dump_present={}\n",
        cache_path.display(),
        post_first_dispatch.environment_compute_dispatch_count(),
        first_stats.last_pipeline,
        second_stats.last_pipeline,
        second_frame.capture_report,
        second_frame.graph_dump.is_some()
    );
    fs::write(
        render_test_output_dir()
            .join("plan11_ibl_product_wgpu_capture_second_launch_dispatch_zero_20260707.txt"),
        report,
    )
    .expect("write Plan 11 runtime IBL cache second-launch Wgpu capture report");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn directory_project_material_shader_drives_pipeline_color_output() {
    let root = unique_temp_project_root("graphics_shader_pipeline");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsShaderPipeline",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
    );
    write_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
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

    let frame = render_snapshot_through_framework(asset_manager, snapshot, UVec2::new(320, 240));

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
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsWireOnly",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("grid.zmaterial"),
        "res://shaders/flat_green.wgsl",
    );
    write_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
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

    let shaded_frame =
        render_snapshot_through_framework(Arc::clone(&asset_manager), shaded, UVec2::new(320, 240));
    let wire_frame =
        render_snapshot_through_framework(asset_manager, wire_only, UVec2::new(320, 240));

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
