use std::{fs, sync::Arc, time::Instant};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT};
use crate::core::framework::render::{
    CookieProjection, EnvironmentExtract, IrradianceVolumeData, LightCookieData, RenderFramework,
    RenderImageColorSpace, RenderImageDimension, RenderLayerSet, RenderViewportDescriptor,
};
use crate::core::math::{Mat4, Vec2, Vec3};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};
use crate::graphics::{
    irradiance_volume_render_pass_executor_registrations,
    light_cookie_render_pass_executor_registrations, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderPassStage, WgpuRenderFramework,
    IRRADIANCE_VOLUME_BIND_EXECUTOR_ID, IRRADIANCE_VOLUME_RESOURCE,
    LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID, LIGHT_COOKIE_ATLAS_RESOURCE,
};
use crate::render_graph::QueueLane;

use super::advanced_pbr::{
    product_quality_profile, register_sphere_model, three_sphere_extract, ProductMaterialMode,
    ProductMaterials, PRODUCT_SIZE,
};
use super::{render_product_output_dir, write_side_by_side_png, FrameDifference, ProductRender};

const PRODUCT_IMAGE_NAME: &str = "plan18_af_m2_light_cookie_irradiance_volume_wgpu_20260715.png";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AfM2Mode {
    Baseline,
    Cookie,
    CookieAndVolume,
}

#[test]
fn render_product_af_m2_feature_off_matches_graph_baseline_exactly() {
    let baseline = render_af_m2(AfM2Mode::Baseline, false);
    let registered_but_empty = render_af_m2(AfM2Mode::Baseline, true);

    assert_eq!(
        baseline.stats.last_graph_executed_passes,
        registered_but_empty.stats.last_graph_executed_passes
    );
    assert_eq!(
        baseline.stats.last_graph_executed_executor_ids,
        registered_but_empty.stats.last_graph_executed_executor_ids
    );
    assert_eq!(
        baseline.stats.last_graph_executed_resource_access_count,
        registered_but_empty
            .stats
            .last_graph_executed_resource_access_count
    );
    assert_eq!(
        baseline.stats.last_graph_executed_dependency_count,
        registered_but_empty
            .stats
            .last_graph_executed_dependency_count
    );
    for pass in [
        LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID,
        IRRADIANCE_VOLUME_BIND_EXECUTOR_ID,
    ] {
        assert!(!registered_but_empty
            .stats
            .last_graph_executed_passes
            .iter()
            .any(|executed| executed == pass));
    }
}

#[test]
fn render_product_af_m2_cookie_and_volume_execute_and_change_wgpu_frame() {
    let baseline = render_af_m2(AfM2Mode::Baseline, false);
    let cookie = render_af_m2(AfM2Mode::Cookie, true);
    let cookie_and_volume = render_af_m2(AfM2Mode::CookieAndVolume, true);

    assert_pass(&cookie, LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID);
    assert_pass(&cookie_and_volume, LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID);
    assert_pass(&cookie_and_volume, IRRADIANCE_VOLUME_BIND_EXECUTOR_ID);
    let cookie_difference = frame_difference(&baseline, &cookie);
    let volume_difference = frame_difference(&cookie, &cookie_and_volume);
    assert!(
        cookie_difference.changed_pixel_count > 2_000,
        "{cookie_difference:?}"
    );
    assert!(
        volume_difference.changed_pixel_count > 2_000,
        "{volume_difference:?}"
    );
}

#[test]
fn render_product_af_m2_frame_without_volume_clears_previous_volume_state() {
    let renders = render_af_m2_sequence(
        &[
            AfM2Mode::Cookie,
            AfM2Mode::Cookie,
            AfM2Mode::CookieAndVolume,
            AfM2Mode::CookieAndVolume,
            AfM2Mode::Cookie,
            AfM2Mode::Cookie,
        ],
        true,
    );
    let controls = render_af_m2_sequence(&[AfM2Mode::Cookie; 6], true);

    assert_pass(&renders[2], IRRADIANCE_VOLUME_BIND_EXECUTOR_ID);
    assert_pass(&renders[3], IRRADIANCE_VOLUME_BIND_EXECUTOR_ID);
    let active_difference = frame_difference(&controls[3], &renders[3]);
    let cleared_difference = frame_difference(&controls[5], &renders[5]);
    assert!(
        active_difference.changed_pixel_count > 2_000,
        "{active_difference:?}"
    );
    assert!(
        cleared_difference.mean_absolute_rgb_error
            < active_difference.mean_absolute_rgb_error * 0.25,
        "active={active_difference:?}, cleared={cleared_difference:?}"
    );
    for render in &renders[4..] {
        assert!(!render
            .stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == IRRADIANCE_VOLUME_BIND_EXECUTOR_ID));
    }
}

#[test]
#[ignore = "exports plan-18 AF-M2 real WGPU light-cookie and irradiance-volume evidence"]
fn export_render_product_af_m2_light_cookie_irradiance_volume_png() {
    let baseline = render_af_m2(AfM2Mode::Baseline, false);
    let active = render_af_m2(AfM2Mode::CookieAndVolume, true);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");
    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_side_by_side_png(&image_path, &baseline.frame, &active.frame);
    assert!(
        image_path.is_file(),
        "AF-M2 WGPU product PNG was not exported"
    );
}

#[test]
#[ignore = "captures plan-18 AF-M2 cookie atlas and irradiance-volume passes through RenderDoc"]
fn capture_render_product_af_m2_renderdoc() {
    let active = render_af_m2(AfM2Mode::CookieAndVolume, true);
    assert_pass(&active, LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID);
    assert_pass(&active, IRRADIANCE_VOLUME_BIND_EXECUTOR_ID);
}

fn render_af_m2(mode: AfM2Mode, register_features: bool) -> ProductRender {
    render_af_m2_sequence(&[mode, mode], register_features)
        .pop()
        .expect("the stabilized AF-M2 product frame should be rendered")
}

fn render_af_m2_sequence(modes: &[AfM2Mode], register_features: bool) -> Vec<ProductRender> {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = register_sphere_model(&asset_manager);
    let materials = ProductMaterials::register(&asset_manager, ProductMaterialMode::Baseline);
    let cookie_texture = register_cookie_texture(&asset_manager);
    let volume_texture = register_volume_texture(&asset_manager);
    let framework = if register_features {
        let mut registrations = light_cookie_render_pass_executor_registrations();
        registrations.extend(irradiance_volume_render_pass_executor_registrations());
        WgpuRenderFramework::new_for_test_with_plugin_render_features(
            Arc::clone(&asset_manager),
            [cookie_descriptor(), irradiance_volume_descriptor()],
            registrations,
            Vec::new(),
        )
    } else {
        WgpuRenderFramework::new_for_test(Arc::clone(&asset_manager))
    }
    .expect("WGPU framework should initialize for AF-M2 product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_SIZE))
        .expect("AF-M2 product viewport should be created");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("AF-M2 quality profile should be accepted");
    let renders = modes
        .iter()
        .copied()
        .map(|mode| {
            let mut extract = three_sphere_extract(model, materials);
            extract.environment = EnvironmentExtract::disabled();
            if mode != AfM2Mode::Baseline {
                extract.lighting.advanced_lighting.cookies = vec![LightCookieData {
                    light_id: 18_110,
                    texture: cookie_texture,
                    projection: CookieProjection::Directional {
                        offset: Vec2::new(0.5, 0.5),
                        scale: Vec2::splat(0.16),
                        wrap: crate::core::framework::render::CookieWrapMode::Repeat,
                    },
                }];
            }
            if mode == AfM2Mode::CookieAndVolume {
                extract.lighting.advanced_lighting.irradiance_volumes =
                    vec![IrradianceVolumeData {
                        volume_id: 18_220,
                        transform: Mat4::from_scale(Vec3::splat(0.05)),
                        voxels: volume_texture,
                        intensity: 1.65,
                        affects_lightmapped_meshes: true,
                        priority: 8,
                        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                    }];
            }
            let start = Instant::now();
            framework
                .submit_frame_extract(viewport, extract)
                .expect("AF-M2 product frame should submit");
            let submit_cpu_micros = start.elapsed().as_micros();
            let frame = framework
                .capture_frame(viewport)
                .expect("AF-M2 frame capture should succeed")
                .expect("AF-M2 viewport should expose a frame");
            let stats = framework.query_stats().expect("AF-M2 stats should exist");
            ProductRender {
                frame,
                stats,
                submit_cpu_micros,
            }
        })
        .collect();
    framework
        .destroy_viewport(viewport)
        .expect("AF-M2 viewport should be destroyed");
    renders
}

fn cookie_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "light_cookies",
        vec!["view".into(), "lighting".into(), "advanced_lighting".into()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID,
            QueueLane::Graphics,
        )
        .with_side_effects()
        .write_external_texture(LIGHT_COOKIE_ATLAS_RESOURCE)],
    )
    .when_advanced_lighting_cookies_enabled()
    .with_pass_read_external_texture("deferred-lighting", LIGHT_COOKIE_ATLAS_RESOURCE)
}

fn irradiance_volume_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "irradiance_volumes",
        vec!["view".into(), "lighting".into(), "advanced_lighting".into()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            IRRADIANCE_VOLUME_BIND_EXECUTOR_ID,
            QueueLane::Graphics,
        )
        .with_side_effects()
        .write_external_texture(IRRADIANCE_VOLUME_RESOURCE)],
    )
    .when_advanced_lighting_irradiance_volumes_enabled()
    .with_pass_read_external_texture("deferred-gbuffer", IRRADIANCE_VOLUME_RESOURCE)
}

fn register_cookie_texture(asset_manager: &ProjectAssetManager) -> ResourceId {
    let uri = AssetUri::parse("res://lighting/plan18-cookie-checker.rgba8").unwrap();
    let id = ResourceId::from_locator(&uri);
    let size = 64u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let bright = ((x / 32) + (y / 32)) % 2 == 0;
            let value = if bright { 255 } else { 0 };
            rgba.extend_from_slice(&[value, value, value, 255]);
        }
    }
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = RGBA8_UNORM_FORMAT.to_string();
    descriptor.color_space = RenderImageColorSpace::Linear;
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Texture, uri.clone()),
            TextureAsset::new_rgba8(uri, size, size, rgba).with_descriptor(descriptor),
        )
        .expect("cookie texture should register");
    id
}

fn register_volume_texture(asset_manager: &ProjectAssetManager) -> ResourceId {
    let uri = AssetUri::parse("res://lighting/plan18-irradiance-volume.rgba8").unwrap();
    let id = ResourceId::from_locator(&uri);
    let [width, height, depth] = [4u32, 8u32, 12u32];
    let mut rgba = Vec::with_capacity((width * height * depth * 4) as usize);
    for z in 0..depth {
        for y in 0..height {
            for _x in 0..width {
                let axis = z / 4;
                let negative = y >= 4;
                let color = match (axis, negative) {
                    (0, false) => [255, 68, 24, 255],
                    (0, true) => [32, 84, 255, 255],
                    (1, false) => [72, 255, 88, 255],
                    (1, true) => [180, 36, 220, 255],
                    (2, false) => [255, 220, 64, 255],
                    _ => [36, 220, 255, 255],
                };
                rgba.extend_from_slice(&color);
            }
        }
    }
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = RGBA8_UNORM_FORMAT.to_string();
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor.dimension = RenderImageDimension::D3;
    descriptor.depth_or_array_layers = depth;
    descriptor.array_layer_count = 1;
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Texture, uri.clone()),
            TextureAsset::new_rgba8(uri, width, height, rgba).with_descriptor(descriptor),
        )
        .expect("irradiance volume texture should register");
    id
}

fn assert_pass(render: &ProductRender, pass: &str) {
    assert!(
        render
            .stats
            .last_graph_executed_passes
            .iter()
            .any(|executed| executed == pass),
        "expected `{pass}` in {:?}",
        render.stats.last_graph_executed_passes
    );
}

fn frame_difference(left: &ProductRender, right: &ProductRender) -> FrameDifference {
    let mut difference = FrameDifference::default();
    let mut total_error = 0u64;
    for (left, right) in left
        .frame
        .rgba
        .chunks_exact(4)
        .zip(right.frame.rgba.chunks_exact(4))
    {
        let errors = left[..3]
            .iter()
            .zip(&right[..3])
            .map(|(left, right)| left.abs_diff(*right))
            .collect::<Vec<_>>();
        if errors.iter().any(|error| *error != 0) {
            difference.changed_pixel_count += 1;
        }
        for error in errors {
            difference.max_rgb_error = difference.max_rgb_error.max(error);
            total_error += u64::from(error);
        }
    }
    difference.mean_absolute_rgb_error =
        total_error as f64 / (left.frame.width as f64 * left.frame.height as f64 * 3.0);
    difference
}
