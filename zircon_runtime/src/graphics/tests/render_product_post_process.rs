use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT};
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, PreviewEnvironmentExtract,
    RenderChromaticAberrationSettings, RenderColorGradingSettings, RenderColorLookupSettings,
    RenderColorLookupTextureLayout, RenderColorLutReadbackReference, RenderDitherSettings,
    RenderFilmGrainSettings, RenderFrameExtract, RenderFramework, RenderImageColorSpace,
    RenderPostProcessEffectStackSettings, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderStats, RenderTonemapOperator, RenderTonemapSettings,
    RenderViewportDescriptor, RenderVignetteSettings, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot, COLOR_LUT_SIZE_DEFAULT, DEFAULT_CAMERA_EXPOSURE_EV100,
};
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::WgpuRenderFramework;

#[path = "render_product_post_process/motion_blur.rs"]
mod motion_blur;

#[test]
fn render_product_post_uber_light_effects_change_final_frame() {
    let viewport_size = UVec2::new(128, 96);
    let server =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let baseline_viewport = create_post_process_viewport(&server, viewport_size, "post-baseline");
    let effects_viewport = create_post_process_viewport(&server, viewport_size, "post-uber-light");

    let (baseline, _) = submit_and_capture_post_process_product(
        &server,
        baseline_viewport,
        post_process_product_extract(
            viewport_size,
            RenderPostProcessEffectStackSettings::default(),
        ),
    );
    let (effects, stats) = submit_and_capture_post_process_product(
        &server,
        effects_viewport,
        post_process_product_extract(viewport_size, light_effect_stack()),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec![
            "vignette".to_string(),
            "film-grain".to_string(),
            "dither".to_string(),
            "chromatic-aberration".to_string(),
        ]
    );
    assert!(
        stats
            .last_post_process_effect_stack_report
            .missing_resources
            .is_empty(),
        "motion blur product scene should not miss required resources; report={:?}",
        stats.last_post_process_effect_stack_report
    );

    let corner = UVec2::new(0, 0);
    let sample = UVec2::new(24, 24);
    let baseline_corner_luma = average_luma_in_region(&baseline, corner, sample);
    let effects_corner_luma = average_luma_in_region(&effects, corner, sample);
    let frame_delta = frame_rgb_abs_delta(&effects, &baseline);

    assert!(
        baseline_corner_luma > 40.0,
        "baseline clear product should be visible; luma={baseline_corner_luma:.2}"
    );
    assert!(
        effects_corner_luma + 8.0 < baseline_corner_luma,
        "vignette/light style effects should darken the final-frame corner; baseline={baseline_corner_luma:.2}, effects={effects_corner_luma:.2}"
    );
    assert!(
        frame_delta > 10_000,
        "uber light effects should produce a measurable final-frame delta; delta={frame_delta}"
    );
}

#[test]
fn render_product_post_non_neutral_tonemap_grading_changes_final_frame() {
    let viewport_size = UVec2::new(128, 96);
    let server =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let baseline_viewport = create_post_process_viewport_with_profile(
        &server,
        viewport_size,
        post_process_color_transform_profile("post-color-baseline"),
    );
    let graded_viewport = create_post_process_viewport_with_profile(
        &server,
        viewport_size,
        post_process_color_transform_profile("post-color-graded"),
    );

    let (baseline, _) = submit_and_capture_post_process_product(
        &server,
        baseline_viewport,
        post_process_product_extract(
            viewport_size,
            RenderPostProcessEffectStackSettings::default(),
        ),
    );
    let (graded, stats) = submit_and_capture_post_process_product(
        &server,
        graded_viewport,
        post_process_color_transform_extract(viewport_size),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "color-lut-bake");
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, "post.color-lut-bake");
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");

    let exposure_report = stats.last_exposure_readback_report;
    assert!(
        exposure_report.available,
        "exposure readback should be available"
    );
    assert!(
        exposure_report.history_valid(),
        "exposure resolve should write a valid history word; report={exposure_report:?}"
    );
    assert!(
        exposure_report.multiplier_within_epsilon(1.0, 0.0001),
        "manual default exposure should resolve to multiplier 1.0; report={exposure_report:?}"
    );
    assert!(
        (exposure_report.resolved_ev100() - DEFAULT_CAMERA_EXPOSURE_EV100 as f32).abs() < 0.0001,
        "manual exposure should resolve to default EV100; report={exposure_report:?}"
    );
    assert!(
        (exposure_report.average_ev100() - DEFAULT_CAMERA_EXPOSURE_EV100 as f32).abs() < 0.0001,
        "manual exposure average EV100 should retain manual default; report={exposure_report:?}"
    );

    let lut_report = stats.last_color_lut_readback_report;
    assert!(
        lut_report.available,
        "color LUT readback should be available"
    );
    assert_eq!(
        lut_report.reference,
        RenderColorLutReadbackReference::ColorTransform
    );
    assert_eq!(
        lut_report.size,
        [
            COLOR_LUT_SIZE_DEFAULT,
            COLOR_LUT_SIZE_DEFAULT,
            COLOR_LUT_SIZE_DEFAULT,
        ]
    );
    assert_eq!(
        lut_report.sample_count,
        COLOR_LUT_SIZE_DEFAULT.pow(3) as usize
    );
    assert!(!lut_report.invalid_byte_len);
    assert_eq!(lut_report.invalid_sample_count, 0);
    assert!(
        lut_report.color_transform_within_epsilon(),
        "non-neutral LUT should match the CPU tonemap/grading reference; report={lut_report:?}"
    );
    assert!(
        !lut_report.identity_within_epsilon(),
        "non-neutral tonemap and grading should bake a non-identity LUT; report={lut_report:?}"
    );
    assert!(
        lut_report.identity_out_of_tolerance_sample_count > 0,
        "non-neutral LUT should differ from the identity reference; report={lut_report:?}"
    );

    let frame_delta = frame_rgb_abs_delta(&graded, &baseline);
    let sample_origin = UVec2::new(32, 24);
    let sample_size = UVec2::new(64, 48);
    let baseline_luma = average_luma_in_region(&baseline, sample_origin, sample_size);
    let graded_luma = average_luma_in_region(&graded, sample_origin, sample_size);

    assert!(
        frame_delta > 20_000,
        "non-neutral tonemap/grading should produce a measurable final-frame delta; delta={frame_delta}"
    );
    assert!(
        (graded_luma - baseline_luma).abs() > 5.0,
        "non-neutral tonemap/grading should shift final-frame luma; baseline={baseline_luma:.2}, graded={graded_luma:.2}"
    );
}

#[test]
fn render_product_post_user_lut_texture_changes_final_frame_and_matches_readback_reference() {
    let viewport_size = UVec2::new(128, 96);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let user_lut_texture = insert_user_lut_texture(
        &asset_manager,
        "res://tests/post-process/lut/invert-green-half-32",
    );
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let baseline_viewport =
        create_post_process_viewport(&server, viewport_size, "post-lut-baseline");
    let lut_viewport = create_post_process_viewport(&server, viewport_size, "post-lut-user");

    let (baseline, _) = submit_and_capture_post_process_product(
        &server,
        baseline_viewport,
        post_process_product_extract(
            viewport_size,
            RenderPostProcessEffectStackSettings::default(),
        ),
    );
    let (lut_frame, stats) = submit_and_capture_post_process_product(
        &server,
        lut_viewport,
        post_process_product_extract(viewport_size, user_lut_effect_stack(user_lut_texture)),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "color-lut-bake");
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, "post.color-lut-bake");
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_eq!(stats.last_post_process_lut_request_count, 1);
    assert_eq!(stats.last_post_process_lut_ready_count, 1);
    assert_eq!(stats.last_post_process_lut_fallback_count, 0);
    assert_eq!(stats.last_post_process_lut_2d_strip_ready_count, 1);
    assert_eq!(stats.last_post_process_lut_3d_request_count, 0);
    assert_eq!(stats.last_post_process_lut_unsupported_shape_count, 0);
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec!["lut".to_string()]
    );
    assert!(
        stats
            .last_post_process_effect_stack_report
            .missing_resources
            .is_empty(),
        "motion blur product scene should not miss required resources; report={:?}",
        stats.last_post_process_effect_stack_report
    );

    let lut_report = stats.last_color_lut_readback_report;
    assert!(
        lut_report.available,
        "user LUT readback should be available"
    );
    assert_eq!(
        lut_report.reference,
        RenderColorLutReadbackReference::UserLut
    );
    assert_eq!(
        lut_report.size,
        [
            COLOR_LUT_SIZE_DEFAULT,
            COLOR_LUT_SIZE_DEFAULT,
            COLOR_LUT_SIZE_DEFAULT,
        ]
    );
    assert_eq!(
        lut_report.sample_count,
        COLOR_LUT_SIZE_DEFAULT.pow(3) as usize
    );
    assert!(!lut_report.invalid_byte_len);
    assert_eq!(lut_report.invalid_sample_count, 0);
    assert!(
        lut_report.user_lut_within_epsilon(),
        "baked LUT should match the registered user LUT reference; report={lut_report:?}"
    );
    assert!(
        !lut_report.identity_within_epsilon(),
        "non-neutral user LUT should not be reported as identity; report={lut_report:?}"
    );
    assert!(
        lut_report.identity_out_of_tolerance_sample_count > 0,
        "user LUT should differ from identity; report={lut_report:?}"
    );

    let frame_delta = frame_rgb_abs_delta(&lut_frame, &baseline);
    let sample_origin = UVec2::new(32, 24);
    let sample_size = UVec2::new(64, 48);
    let baseline_rgb = average_rgb_in_region(&baseline, sample_origin, sample_size);
    let lut_rgb = average_rgb_in_region(&lut_frame, sample_origin, sample_size);

    assert!(
        frame_delta > 100_000,
        "user LUT should produce a measurable final-frame delta; delta={frame_delta}"
    );
    assert!(
        lut_rgb[0] + 20.0 < baseline_rgb[0],
        "user LUT should invert/reduce red in the final frame; baseline={baseline_rgb:?}, lut={lut_rgb:?}"
    );
    assert!(
        lut_rgb[1] + 20.0 < baseline_rgb[1],
        "user LUT should reduce green in the final frame; baseline={baseline_rgb:?}, lut={lut_rgb:?}"
    );
}

#[test]
fn render_product_post_internal_lut_uses_effective_stack_without_caller_rebuild() {
    use crate::core::framework::render::PostProcessEffectKind;

    const DISPLAY_ABS: f32 = 0.007;
    let size = UVec2::new(32, 32);
    let server = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
        .expect("a WGPU device is required for the LUT producer product regression");
    let startup = server.query_stats().unwrap();
    let adapter = startup
        .device_diagnostics
        .as_ref()
        .expect("actual adapter identity");
    eprintln!(
        "LUT producer adapter={:?} type={} backend={} display_linear_abs={DISPLAY_ABS}",
        adapter.adapter_name, adapter.adapter_device_type, startup.capabilities.backend_name
    );

    let profile = post_process_color_transform_profile("effective-color-lut-producer");
    let stale_view = create_post_process_viewport_with_profile(&server, size, profile.clone());
    let rebuilt_view = create_post_process_viewport_with_profile(&server, size, profile);
    let mut stale = post_process_product_extract(
        size,
        RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Reinhard,
                exposure_bias: 0.0,
                white_point: 1.0,
            },
            ..Default::default()
        },
    );
    let low = [0.25, 0.5, 0.75];
    stale.post_process.preview.clear_color = Vec4::new(0.25, 0.5, 0.75, 1.0);
    let mut rebuilt = stale.clone();
    rebuilt.post_process.rebuild_graph(false, false);
    let authored_contains_bake = |extract: &RenderFrameExtract| {
        extract
            .post_process
            .graph
            .nodes
            .iter()
            .any(|node| node.kind == PostProcessEffectKind::ColorLutBake)
    };
    assert!(!authored_contains_bake(&stale));
    assert!(authored_contains_bake(&rebuilt));

    let observe = |case, viewport, extract, expected_src: [f32; 3]| {
        let (frame, stats) = submit_and_capture_post_process_product(&server, viewport, extract);
        assert_graph_executor_executed(&stats, "post.color-lut-bake");
        assert_graph_executor_executed(&stats, "post.uber");
        assert_graph_executor_executed(&stats, "post.output-transfer");
        let exposure = stats.last_exposure_readback_report;
        assert!(exposure.history_valid() && exposure.multiplier_within_epsilon(1.0, 0.0001));
        let pass = stats
            .last_graph_execution_profile_report
            .pass_profiles
            .iter()
            .find(|pass| pass.executor_id == "post.color-lut-bake")
            .expect("compiled bake pass profile, even if no work was encoded");
        let lut = stats.last_color_lut_readback_report;
        let hdr = server
            .capture_scene_color_hdr(viewport)
            .unwrap()
            .expect("actual HDR input capture");
        assert_eq!(hdr.generation, frame.generation);
        let center = |w, h| ((h / 2) * w + w / 2) as usize;
        let src = hdr.rgba16f[center(hdr.width, hdr.height)];
        assert_eq!(&src[..3], &expected_src[..]);
        let at = center(frame.width, frame.height) * 4;
        assert_eq!(frame.rgba[at + 3], 255);
        let srgb = [frame.rgba[at], frame.rgba[at + 1], frame.rgba[at + 2]];
        let rgb = srgb.map(|byte| {
            let s = byte as f32 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        });
        eprintln!(
            "LUT producer case={case} src={src:?} bake_dispatches={} graph_missing_dispatches={} lut={lut:?} srgb8={srgb:?} rgb={rgb:?}",
            pass.dispatch_count, stats.last_graph_compute_missing_dispatch_count,
        );
        (pass.dispatch_count, lut, rgb)
    };

    // Make the unrebuilt arm the first use of this 3D allocation so a prior
    // successful bake cannot accidentally provide its texels.
    let mut hdr_control = rebuilt.clone();
    hdr_control.post_process.preview.clear_color = Vec4::new(1.0, 2.0, 4.0, 1.0);
    let without_rebuild = observe("without-caller-rebuild", stale_view, stale, low);
    let with_rebuild = observe("caller-rebuilt", rebuilt_view, rebuilt, low);
    // Diagnostic only: a healthy producer can still feed a consumer that
    // clips its input domain. Do not freeze the old out-of-domain behavior.
    let _ = observe(
        "caller-rebuilt-hdr-diagnostic",
        rebuilt_view,
        hdr_control,
        [1.0, 2.0, 4.0],
    );

    for (case, (dispatches, lut, rgb)) in [
        ("caller-rebuilt", with_rebuild),
        ("without-caller-rebuild", without_rebuild),
    ] {
        assert!(
            dispatches > 0,
            "{case}: executor admission did not record any bake dispatch"
        );
        assert_eq!(
            lut.size, [COLOR_LUT_SIZE_DEFAULT; 3],
            "{case}: fresh 3D LUT readback required"
        );
        assert_eq!(lut.sample_count, COLOR_LUT_SIZE_DEFAULT.pow(3) as usize);
        assert_eq!(
            lut.alpha_out_of_tolerance_sample_count, 0,
            "{case}: bake must write alpha=1 across the volume"
        );
        assert!(
            lut.color_transform_within_epsilon(),
            "{case}: bad baked LUT content: {lut:?}"
        );
        let expected = low.map(|c| c / (1.0 + c));
        assert!(
            rgb.into_iter()
                .zip(expected)
                .all(|(actual, want)| (actual - want).abs() <= DISPLAY_ABS),
            "{case}: low-domain pixel did not use a valid LUT: {rgb:?}, expected {expected:?}"
        );
    }
}

fn create_post_process_viewport(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> crate::core::framework::render::RenderViewportHandle {
    create_post_process_viewport_with_profile(
        server,
        viewport_size,
        post_process_product_profile(profile_name),
    )
}

fn create_post_process_viewport_with_profile(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile: RenderQualityProfile,
) -> crate::core::framework::render::RenderViewportHandle {
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server.set_quality_profile(viewport, profile).unwrap();
    viewport
}

fn post_process_product_profile(profile_name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn post_process_color_transform_profile(profile_name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(true)
        .with_anti_alias(false)
}

fn post_process_product_extract(
    viewport_size: UVec2,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(920),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::new(0.68, 0.74, 0.82, 1.0),
            },
            virtual_geometry_debug: None,
        },
    );
    extract.apply_viewport_size(viewport_size);
    extract.post_process.effect_stack = effect_stack;
    extract
}

fn post_process_color_transform_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = post_process_product_extract(viewport_size, color_transform_effect_stack());
    extract.post_process.color_grading = RenderColorGradingSettings {
        exposure: 1.25,
        contrast: 1.35,
        saturation: 0.55,
        gamma: 0.85,
        tint: Vec3::new(1.12, 0.82, 0.62),
    };
    extract
}

fn color_transform_effect_stack() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: RenderTonemapOperator::Aces,
            exposure_bias: 0.75,
            white_point: 1.15,
        },
        ..Default::default()
    }
}

fn light_effect_stack() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        vignette: RenderVignetteSettings {
            intensity: 0.65,
            smoothness: 0.45,
            roundness: 1.0,
        },
        grain: RenderFilmGrainSettings {
            intensity: 0.18,
            response: 1.0,
        },
        dither: RenderDitherSettings {
            intensity: 0.08,
            scale: 1.0,
        },
        chromatic_aberration: RenderChromaticAberrationSettings {
            intensity: 0.25,
            sample_spread: 3.0,
        },
        ..Default::default()
    }
}

fn user_lut_effect_stack(
    texture: ResourceHandle<TextureMarker>,
) -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        color_lookup: RenderColorLookupSettings {
            texture: Some(texture),
            texture_layout: RenderColorLookupTextureLayout::Texture2dStrip {
                size: COLOR_LUT_SIZE_DEFAULT,
            },
            intensity: 1.0,
        },
        ..Default::default()
    }
}

fn insert_user_lut_texture(
    asset_manager: &ProjectAssetManager,
    uri: &str,
) -> ResourceHandle<TextureMarker> {
    let size = COLOR_LUT_SIZE_DEFAULT;
    let width = size * size;
    let height = size;
    let texture_uri = AssetUri::parse(uri).unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            TextureAsset::new_rgba8(texture_uri, width, height, user_lut_strip_rgba8(size))
                .with_descriptor(user_lut_texture_descriptor()),
        )
        .expect("user LUT texture insert");
    ResourceHandle::<TextureMarker>::new(texture_id)
}

fn user_lut_texture_descriptor() -> TextureAssetDescriptor {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = RGBA8_UNORM_FORMAT.to_string();
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor
}

fn user_lut_strip_rgba8(size: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * size * 4) as usize);
    for green in 0..size {
        for blue in 0..size {
            for red in 0..size {
                let source_color = [
                    lut_axis_value(red, size),
                    lut_axis_value(green, size),
                    lut_axis_value(blue, size),
                ];
                let expected = expected_user_lut_color(source_color);
                rgba.push(linear_channel_to_u8(expected[0]));
                rgba.push(linear_channel_to_u8(expected[1]));
                rgba.push(linear_channel_to_u8(expected[2]));
                rgba.push(255);
            }
        }
    }
    rgba
}

fn expected_user_lut_color(source_color: [f32; 3]) -> [f32; 3] {
    [
        1.0 - source_color[0],
        source_color[1] * 0.5,
        source_color[2],
    ]
}

fn lut_axis_value(index: u32, size: u32) -> f32 {
    if size <= 1 {
        0.0
    } else {
        index as f32 / (size - 1) as f32
    }
}

fn linear_channel_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn submit_and_capture_post_process_product(
    server: &WgpuRenderFramework,
    viewport: crate::core::framework::render::RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    server.submit_frame_extract(viewport, extract).unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("post-process product frame should be capturable");
    let stats = server.query_stats().unwrap();
    (frame, stats)
}

fn assert_post_process_node_executed(stats: &RenderStats, node: &str) {
    assert!(
        stats
            .last_post_process_graph_executed_nodes
            .iter()
            .any(|executed| executed == node),
        "expected post-process node `{node}` to execute; executed={:?}",
        stats.last_post_process_graph_executed_nodes
    );
}

fn assert_graph_executor_executed(stats: &RenderStats, executor_id: &str) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executed| executed == executor_id),
        "expected graph executor `{executor_id}` to execute; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn average_luma_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> f32 {
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let width = frame.width as usize;
    let mut sum = 0.0;
    let mut count = 0usize;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4;
            let r = frame.rgba[index] as f32;
            let g = frame.rgba[index + 1] as f32;
            let b = frame.rgba[index + 2] as f32;
            sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
            count += 1;
        }
    }
    assert!(count > 0, "sample region should contain pixels");
    sum / count as f32
}

fn average_rgb_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> [f32; 3] {
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let width = frame.width as usize;
    let mut sum = [0.0; 3];
    let mut count = 0usize;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4;
            sum[0] += frame.rgba[index] as f32;
            sum[1] += frame.rgba[index + 1] as f32;
            sum[2] += frame.rgba[index + 2] as f32;
            count += 1;
        }
    }
    assert!(count > 0, "sample region should contain pixels");
    [
        sum[0] / count as f32,
        sum[1] / count as f32,
        sum[2] / count as f32,
    ]
}

fn frame_rgb_abs_delta(left: &CapturedFrame, right: &CapturedFrame) -> u64 {
    assert_eq!(left.width, right.width);
    assert_eq!(left.height, right.height);
    assert_eq!(left.rgba.len(), right.rgba.len());
    left.rgba
        .chunks_exact(4)
        .zip(right.rgba.chunks_exact(4))
        .map(|(left, right)| {
            left[0].abs_diff(right[0]) as u64
                + left[1].abs_diff(right[1]) as u64
                + left[2].abs_diff(right[2]) as u64
        })
        .sum()
}
