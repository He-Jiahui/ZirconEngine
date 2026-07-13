use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, build_source_cubemap_irradiance_cube,
    source_cubemap_face_mip_offset, source_cubemap_mip_size, CubemapFace, EnvironmentExtract,
    PreviewEnvironmentExtract, ProjectionMode, RenderOverlayExtract, RenderSceneSnapshot,
    SceneViewportExtractRequest, SourceCubemapEnvironment, SourceCubemapMipChain,
    ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::graphics::{SceneRenderer, ViewportFrame};

#[path = "runtime_shader_pbr_hdri_export/fixture_assets.rs"]
mod fixture_assets;
#[path = "runtime_shader_pbr_hdri_export/frame_assertions.rs"]
mod frame_assertions;
#[path = "runtime_shader_pbr_hdri_export/hdri_metrics.rs"]
mod hdri_metrics;
#[path = "runtime_shader_pbr_hdri_export/pbr_matrix.rs"]
mod pbr_matrix;
#[path = "runtime_shader_pbr_hdri_export/pbr_matrix_quantitative.rs"]
mod pbr_matrix_quantitative;
#[path = "runtime_shader_pbr_hdri_export/scene_fixtures.rs"]
mod scene_fixtures;
#[path = "runtime_shader_pbr_hdri_export/sphere_reflection.rs"]
mod sphere_reflection;

use fixture_assets::{
    ambientcg_metal009_texture_uri, ambientcg_metal_texture_uri,
    write_ambientcg_metal009_texture_assets, write_ambientcg_metal_texture_assets,
    AmbientCgMetalFixture, AMBIENTCG_METAL008, AMBIENTCG_METAL009_COLOR,
    AMBIENTCG_METAL009_METALLIC_ROUGHNESS, AMBIENTCG_METAL009_NORMAL_GL, AMBIENTCG_METAL025,
    AMBIENTCG_METAL029,
};
use frame_assertions::{
    assert_mirror_sphere_matches_source_reference,
    assert_mirror_sphere_matches_source_reference_with_camera_view,
    assert_mirror_sphere_reflection_orientation, assert_single_sphere_reflects_environment,
    assert_textured_material_has_surface_variation,
};
use scene_fixtures::{
    write_pbr_matrix_material, write_pbr_matrix_scene, write_single_pbr_material,
    write_single_pbr_sphere_scene_with_camera_view, write_uv_sphere_model,
    SinglePbrSphereCameraView,
};
use sphere_reflection::render_single_pbr_sphere_frame_with_environment;

const PBR_MATRIX_DIMENSION: usize = 8;
const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1600, 1200);
const PBR_MATRIX_ORTHO_SIZE: f32 = 5.8;
const PBR_MATRIX_STEP_X: f32 = 0.7;
const PBR_MATRIX_STEP_Y: f32 = 0.62;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.21;
const PBR_MATRIX_CELL_SAMPLE_SIZE: u32 = 40;
const PBR_MATRIX_HDRI_1K_PMREM_MIP_DIAGNOSTIC_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_1k_angular_source_pmrem_mip_diagnostic_20260706.png";
const PBR_TEXTURED_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260706.png";
const PBR_TEXTURED_HDRI_METAL008_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal008_texture_maps_20260707.png";
const PBR_TEXTURED_HDRI_METAL025_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal025_texture_maps_20260707.png";
const PBR_TEXTURED_HDRI_METAL029_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal029_texture_maps_20260707.png";
const POLYHAVEN_LAKES_1K_HDRI_ASSET: &str = "polyhaven_lakes_1k.hdr";
const POLYHAVEN_LAKES_2K_HDRI_ASSET: &str = "polyhaven_lakes_2k.hdr";
const LEGACY_EQUIRECT_SAMPLE_COLUMNS: u32 = 16;
const LEGACY_EQUIRECT_SAMPLE_ROWS: u32 = 8;
const LEGACY_GRID_SKY_SAMPLE_Y_MIN: u32 = 24;
const LEGACY_GRID_SKY_SAMPLE_Y_MAX: u32 = 220;
const LEGACY_GRID_OFFSET_SAMPLES: [i32; 6] = [-17, -11, -7, 7, 11, 17];
const PMREM_MIP_DIAGNOSTIC_TILE_SIZE: u32 = 96;

#[test]
fn runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_matches_blur_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(
        PBR_MATRIX_HDRI_1K_PMREM_MIP_DIAGNOSTIC_OUTPUT_NAME,
    );

    assert_shader_test_output_path(&output);
    hdri_metrics::assert_saved_pmrem_mip_diagnostic_blur_response(&output);
}

#[test]
#[ignore = "manual diagnostic export for source cubemap versus GGX PMREM mip blur validation"]
fn export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_mip_diagnostic".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_inner)
        .expect("spawn large-stack HDRI PMREM mip diagnostic export test")
        .join()
        .expect("HDRI PMREM mip diagnostic export test thread should not panic");
}

#[test]
#[ignore = "manual real texture-map material export for runtime PBR real HDRI validation"]
fn export_runtime_shader_pbr_real_hdri_textured_material_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_textured_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_textured_material_png_inner)
        .expect("spawn large-stack textured HDRI export test")
        .join()
        .expect("textured HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual ambientCG Metal008/025/029 material export for runtime PBR real HDRI validation"]
fn export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_ambientcg_metal_batch".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png_inner)
        .expect("spawn large-stack ambientCG metal batch HDRI export test")
        .join()
        .expect("ambientCG metal batch HDRI export test thread should not panic");
}

fn export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_inner() {
    let environment = polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_1K_HDRI_ASSET, 1);
    let output = runtime_shader_pbr_real_hdri_output_path(
        PBR_MATRIX_HDRI_1K_PMREM_MIP_DIAGNOSTIC_OUTPUT_NAME,
    );

    save_pmrem_mip_diagnostic(&environment.mip_chain, &output);
    assert_shader_test_output_path(&output);
}

fn export_runtime_shader_pbr_real_hdri_textured_material_png_inner() {
    export_runtime_shader_pbr_real_hdri_textured_material_png_with_metal009();
}

fn export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png_inner() {
    for (fixture, output_name) in [
        (AMBIENTCG_METAL008, PBR_TEXTURED_HDRI_METAL008_OUTPUT_NAME),
        (AMBIENTCG_METAL025, PBR_TEXTURED_HDRI_METAL025_OUTPUT_NAME),
        (AMBIENTCG_METAL029, PBR_TEXTURED_HDRI_METAL029_OUTPUT_NAME),
    ] {
        export_runtime_shader_pbr_real_hdri_textured_material_png_with_fixture(
            fixture,
            output_name,
        );
    }
}

fn export_runtime_shader_pbr_real_hdri_textured_material_png_with_metal009() {
    let environment = EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 3),
    );
    let frame = render_single_pbr_sphere_frame_with_environment(
        environment,
        "GraphicsPbrRealHdriTexturedMaterial",
        |paths| {
            write_ambientcg_metal009_texture_assets(paths);
            write_single_pbr_material(
                paths
                    .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                    .join("materials")
                    .join("single_metal_sphere.zmaterial"),
                "AmbientCG Metal009 Texture Maps",
                [1.0, 1.0, 1.0, 1.0],
                1.0,
                1.0,
                Some(&ambientcg_metal009_texture_uri(AMBIENTCG_METAL009_COLOR)),
                Some(&ambientcg_metal009_texture_uri(
                    AMBIENTCG_METAL009_NORMAL_GL,
                )),
                Some(&ambientcg_metal009_texture_uri(
                    AMBIENTCG_METAL009_METALLIC_ROUGHNESS,
                )),
            );
        },
    );
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_TEXTURED_HDRI_OUTPUT_NAME);

    save_viewport_frame_png(
        &frame,
        &output,
        "real HDRI PBR textured material screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, "textured PBR sphere");
    assert_textured_material_has_surface_variation(&frame);
}

fn export_runtime_shader_pbr_real_hdri_textured_material_png_with_fixture(
    fixture: AmbientCgMetalFixture,
    output_name: &str,
) {
    let environment = EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 10),
    );
    let frame = render_single_pbr_sphere_frame_with_environment(
        environment,
        &format!("GraphicsPbrRealHdri{}TextureMaps", fixture.id),
        |paths| {
            write_ambientcg_metal_texture_assets(paths, fixture);
            write_single_pbr_material(
                paths
                    .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                    .join("materials")
                    .join("single_metal_sphere.zmaterial"),
                &format!("AmbientCG {} Texture Maps", fixture.id),
                [1.0, 1.0, 1.0, 1.0],
                1.0,
                1.0,
                Some(&ambientcg_metal_texture_uri(fixture, fixture.color)),
                Some(&ambientcg_metal_texture_uri(fixture, fixture.normal_gl)),
                Some(&ambientcg_metal_texture_uri(
                    fixture,
                    fixture.metallic_roughness,
                )),
            );
        },
    );
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);

    save_viewport_frame_png(
        &frame,
        &output,
        "real HDRI PBR ambientCG textured material screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, fixture.id);
    assert_textured_material_has_surface_variation(&frame);
}

fn render_project_frame_with_environment(
    temp_label: &str,
    project_name: &str,
    scene_uri_text: &str,
    output_size: UVec2,
    environment: EnvironmentExtract,
    write_project_assets: impl FnOnce(&ProjectPaths),
) -> zircon_runtime::graphics::ViewportFrame {
    render_project_with_environment(
        temp_label,
        project_name,
        scene_uri_text,
        output_size,
        environment,
        write_project_assets,
        |renderer, snapshot| renderer.render(snapshot, output_size).unwrap(),
    )
}

fn render_project_with_environment<T>(
    temp_label: &str,
    project_name: &str,
    scene_uri_text: &str,
    output_size: UVec2,
    environment: EnvironmentExtract,
    write_project_assets: impl FnOnce(&ProjectPaths),
    render: impl FnOnce(&mut SceneRenderer, RenderSceneSnapshot) -> T,
) -> T {
    let root = unique_temp_project_root(temp_label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let scene_uri = AssetUri::parse(scene_uri_text).unwrap();
    ProjectManifest::new(project_name, scene_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    write_project_assets(&paths);

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world =
        zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(output_size),
        virtual_geometry_debug: None,
    });
    snapshot.environment = environment;
    snapshot.preview =
        PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
    snapshot.overlays = RenderOverlayExtract::default();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let output = render(&mut renderer, snapshot);
    let _ = fs::remove_dir_all(root);
    output
}

fn save_viewport_frame_png(
    frame: &zircon_runtime::graphics::ViewportFrame,
    output: &Path,
    context: &str,
) {
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("rendered real HDRI PBR frame should match output image dimensions")
        .save_with_format(output, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("write {context}: {error}"));
}

fn polyhaven_lakes_source_cubemap_environment(
    asset_name: &str,
    source_revision: u64,
) -> SourceCubemapEnvironment {
    let path = shader_test_asset_dir().join(asset_name);
    let bytes = fs::read(&path).expect("read Poly Haven lakes HDRI");
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)
        .expect("decode Poly Haven lakes HDRI")
        .to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let face_size =
        zircon_runtime::core::framework::render::source_cubemap_face_size_from_equirect_height(
            image.height(),
        );
    let mip_chain = zircon_runtime::core::framework::render::build_source_cubemap_from_equirect(
        face_size,
        |u, v| expose_hdr_sample(sample_hdri_bilinear(&image, u, v), exposure),
    );
    let irradiance_cube = build_source_cubemap_irradiance_cube(&mip_chain);

    let mut environment =
        SourceCubemapEnvironment::new(mip_chain, source_revision, source_hash_words(&bytes))
            .with_irradiance_cube(irradiance_cube);
    environment.intensity = 1.45;
    environment.rotation_radians = 0.0;
    environment
}

fn save_pmrem_mip_diagnostic(mip_chain: &SourceCubemapMipChain, output: &Path) {
    let source_mip_count = mip_chain.source_mip_count();
    let pmrem_mip_count = mip_chain.pmrem_mip_count();
    let face_count = CubemapFace::ALL.len() as u32;
    let width = PMREM_MIP_DIAGNOSTIC_TILE_SIZE * source_mip_count.max(pmrem_mip_count);
    let height = PMREM_MIP_DIAGNOSTIC_TILE_SIZE * face_count * 2;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    for (face_index, face) in CubemapFace::ALL.iter().copied().enumerate() {
        for mip in 0..source_mip_count {
            paint_mip_diagnostic_tile(
                &mut image,
                mip_chain.source_texels(),
                mip_chain.source_face_size(),
                source_mip_count,
                face,
                mip,
                mip * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
                face_index as u32 * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
            );
        }
        for mip in 0..pmrem_mip_count {
            paint_mip_diagnostic_tile(
                &mut image,
                mip_chain.pmrem_texels(),
                mip_chain.pmrem_face_size(),
                pmrem_mip_count,
                face,
                mip,
                mip * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
                (face_count + face_index as u32) * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
            );
        }
    }

    image
        .save_with_format(output, ImageFormat::Png)
        .expect("write PMREM mip diagnostic screenshot");
}

fn paint_mip_diagnostic_tile(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip: u32,
    origin_x: u32,
    origin_y: u32,
) {
    for y in 0..PMREM_MIP_DIAGNOSTIC_TILE_SIZE {
        for x in 0..PMREM_MIP_DIAGNOSTIC_TILE_SIZE {
            let u = (x as f32 + 0.5) / PMREM_MIP_DIAGNOSTIC_TILE_SIZE as f32;
            let v = (y as f32 + 0.5) / PMREM_MIP_DIAGNOSTIC_TILE_SIZE as f32;
            image.put_pixel(
                origin_x + x,
                origin_y + y,
                Rgba(linear_hdr_to_srgb8(sample_face_mip_bilinear(
                    texels, face_size, mip_count, face, mip, u, v,
                ))),
            );
        }
    }
}

fn sample_face_mip_bilinear(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip: u32,
    u: f32,
    v: f32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let x = u.clamp(0.0, 1.0) * mip_size.saturating_sub(1) as f32;
    let y = v.clamp(0.0, 1.0) * mip_size.saturating_sub(1) as f32;
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(mip_size.saturating_sub(1));
    let y1 = (y0 + 1).min(mip_size.saturating_sub(1));
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let c00 = face_mip_texel(texels, face_size, mip_count, face, mip, x0, y0);
    let c10 = face_mip_texel(texels, face_size, mip_count, face, mip, x1, y0);
    let c01 = face_mip_texel(texels, face_size, mip_count, face, mip, x0, y1);
    let c11 = face_mip_texel(texels, face_size, mip_count, face, mip, x1, y1);

    lerp4(lerp4(c00, c10, tx), lerp4(c01, c11, tx), ty)
}

fn face_mip_texel(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip: u32,
    x: u32,
    y: u32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
    texels[offset + y as usize * mip_size as usize + x as usize]
}

fn linear_hdr_to_srgb8(rgba: [f32; 4]) -> [u8; 4] {
    [
        linear_channel_to_srgb8(rgba[0]),
        linear_channel_to_srgb8(rgba[1]),
        linear_channel_to_srgb8(rgba[2]),
        255,
    ]
}

fn linear_channel_to_srgb8(channel: f32) -> u8 {
    let mapped = channel.max(0.0) / (1.0 + channel.max(0.0));
    let srgb = if mapped <= 0.003_130_8 {
        mapped * 12.92
    } else {
        1.055 * mapped.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
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

fn runtime_shader_pbr_real_hdri_output_path(output_name: &str) -> PathBuf {
    shader_test_output_dir().join(output_name)
}

fn assert_shader_test_output_path(path: &Path) {
    let output_dir = shader_test_output_dir();
    assert!(
        path.starts_with(&output_dir),
        "shader validation image should be written under docs/tests/runtime/shader, path={path:?}, expected_dir={output_dir:?}"
    );
}

fn shader_test_asset_dir() -> PathBuf {
    let asset_dir = shader_test_output_dir().join("assets");
    fs::create_dir_all(&asset_dir).unwrap();
    asset_dir
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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}
