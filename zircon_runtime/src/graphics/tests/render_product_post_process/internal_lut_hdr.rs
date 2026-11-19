use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderExposureSettings, RenderImageDimension,
    RenderPipelineHandle,
};
use crate::core::math::Real;
use crate::graphics::{BuiltinRenderFeature, RenderPipelineAsset};

#[test]
fn render_product_post_internal_lut_preserves_hdr_highlights() {
    // Both paths write linear Rgba8Unorm before the final sRGB8 transfer. One
    // round trip costs at most ~0.0065 in linear light, plus LUT approximation
    // for the baked path. This budget still separates the 4 and 8 gray samples.
    const DIRECT_ABS: f32 = 0.007;
    const BAKED_VS_DIRECT_ABS: f32 = 0.018;
    const MIN_HIGHLIGHT_STEP: f32 = 0.04;
    const INPUTS: [([f32; 3], f32); 7] = [
        ([0.0, 0.18, 0.5], 1.0),
        ([1.0; 3], 1.0),
        ([2.0; 3], 1.0),
        ([4.0; 3], 1.0),
        ([8.0; 3], 1.0),
        ([0.18, 8.0, 16.0], 0.25),
        ([32.0, 0.5, 2.0], 1.0),
    ];

    let asset_manager = Arc::new(ProjectAssetManager::default());
    let external_lut = insert_hdr_control_lut(&asset_manager);
    let server = WgpuRenderFramework::new_for_test(asset_manager)
        .expect("HDR LUT pixel regression needs a real WGPU device");
    let stats = server.query_stats().unwrap();
    let device = stats
        .device_diagnostics
        .as_ref()
        .expect("HDR LUT pixel regression needs adapter identification");
    eprintln!(
        "HDR LUT adapter={:?} type={} backend={} linear_abs: direct={DIRECT_ABS}, baked_vs_direct={BAKED_VS_DIRECT_ABS}, min_highlight_step={MIN_HIGHLIGHT_STEP}",
        device.adapter_name, device.adapter_device_type, stats.capabilities.backend_name,
    );
    let viewport_size = UVec2::new(32, 32);
    let profile = post_process_color_transform_profile("hdr-lut-equivalence");
    let baked = create_post_process_viewport_with_profile(
        &server,
        viewport_size,
        profile
            .clone()
            .with_pipeline_asset(RenderPipelineAsset::DEFAULT_FORWARD_PLUS_HANDLE),
    );
    let mut direct_pipeline = RenderPipelineAsset::default_forward_plus();
    direct_pipeline.handle = RenderPipelineHandle::new(154);
    direct_pipeline.name = "hdr-lut-equivalence-direct".into();
    let post = direct_pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::PostProcess))
        .expect("forward-plus post-processing feature");
    let mut descriptor = post.descriptor();
    // Leave the builtin executor, settings, exposure and all other resources
    // intact. With no optional COLOR_LUT input, the uber pass uses direct mode.
    descriptor
        .stage_passes
        .retain(|pass| pass.executor_id.as_str() != "post.color-lut-bake");
    for pass in &mut descriptor.stage_passes {
        pass.resources
            .retain(|resource| resource.name != PostProcessGraphResourceNames::COLOR_LUT);
    }
    post.descriptor_override = Some(descriptor);
    let direct_handle = server.register_pipeline_asset(direct_pipeline).unwrap();
    let direct = create_post_process_viewport_with_profile(
        &server,
        viewport_size,
        profile.with_pipeline_asset(direct_handle),
    );

    let tonemap = RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: RenderTonemapOperator::Reinhard,
            exposure_bias: 0.0,
            white_point: 1.0,
        },
        ..Default::default()
    };
    let read = |viewport, input: [f32; 3], exposure: f32, stack, expect_bake| {
        let mut extract = post_process_product_extract(viewport_size, stack);
        let [r, g, b] = input.map(|c| c as Real);
        extract.post_process.preview.clear_color = Vec4::new(r, g, b, 1.0);
        extract.post_process.exposure = RenderExposureSettings::manual_ev100(
            DEFAULT_CAMERA_EXPOSURE_EV100 - exposure.log2() as Real,
        );
        let (frame, stats) = submit_and_capture_post_process_product(&server, viewport, extract);
        assert_graph_executor_executed(&stats, "post.uber");
        assert_graph_executor_executed(&stats, "post.output-transfer");
        assert_eq!(
            stats
                .last_graph_executed_executor_ids
                .iter()
                .any(|id| id == "post.color-lut-bake"),
            expect_bake,
            "unexpected bake execution; input={input:?} exposure={exposure} executors={:?}",
            stats.last_graph_executed_executor_ids,
        );
        let exposure_report = stats.last_exposure_readback_report;
        assert!(
            exposure_report.available
                && exposure_report.history_valid()
                && exposure_report.multiplier_within_epsilon(exposure, 0.0001),
            "incorrect GPU exposure for input={input:?}: {exposure_report:?}"
        );

        let hdr = server
            .capture_scene_color_hdr(viewport)
            .unwrap()
            .expect("linear HDR input readback");
        assert_eq!(hdr.generation, frame.generation);
        let center = |w, h| ((h / 2) * w + w / 2) as usize;
        let source = hdr.rgba16f[center(hdr.width, hdr.height)];
        let pixel = center(frame.width, frame.height) * 4;
        assert_eq!(frame.rgba[pixel + 3], 255);
        let rgb8 = [
            frame.rgba[pixel],
            frame.rgba[pixel + 1],
            frame.rgba[pixel + 2],
        ];
        for channel in 0..3 {
            assert!(
                (source[channel] - input[channel]).abs() <= 0.0001,
                "HDR input changed before tonemap: expected={input:?}, observed={source:?}"
            );
        }
        (source, rgb8, rgb8.map(hdr_capture_srgb8_to_linear), stats)
    };

    let mut observed = Vec::new();
    for (input, exposure) in INPUTS {
        let (direct_source, direct_bytes, direct_rgb, _) =
            read(direct, input, exposure, tonemap, false);
        let (baked_source, baked_bytes, baked_rgb, stats) =
            read(baked, input, exposure, tonemap, true);
        assert_eq!(
            baked_source, direct_source,
            "the two paths must use the same scene HDR input"
        );
        assert!(
            stats.last_color_lut_readback_report.available,
            "internal LUT readback required"
        );
        let expected = input.map(|c| {
            let c = c * exposure;
            c / (1.0 + c)
        });
        eprintln!(
            "HDR LUT src={baked_source:?} exp={exposure} direct_srgb8={direct_bytes:?} baked_srgb8={baked_bytes:?} direct={direct_rgb:?} baked={baked_rgb:?} expected={expected:?}"
        );
        observed.push((input, exposure, expected, direct_rgb, baked_rgb));
    }

    // This is the adjacent external 3D branch, not a second baked-path oracle.
    // The registered 2x2x2 texture is a linear RGB ramp with red/blue inverted.
    let external_stack = RenderPostProcessEffectStackSettings {
        color_lookup: RenderColorLookupSettings {
            texture: Some(external_lut),
            texture_layout: RenderColorLookupTextureLayout::Texture3d { size: 2 },
            intensity: 1.0,
        },
        ..tonemap
    };
    let (source, bytes, external, stats) = read(baked, [0.25, 2.0, 8.0], 1.0, external_stack, true);
    let expected = [0.8, 2.0 / 3.0, 1.0 / 9.0];
    eprintln!("HDR LUT external mode=3 src={source:?} exp=1 srgb8={bytes:?} linear={external:?} expected={expected:?} linear_abs={DIRECT_ABS}");
    assert_eq!(stats.last_post_process_lut_3d_request_count, 1);
    assert_eq!(stats.last_post_process_lut_ready_count, 1);
    assert_eq!(stats.last_post_process_lut_fallback_count, 0);
    for (actual, expected) in external.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= DIRECT_ABS,
            "external 3D LUT no longer uses normalized addressed color"
        );
    }

    // Check all sample readbacks (including the red baseline) before panicking.
    for (input, exposure, expected, direct_rgb, baked_rgb) in &observed {
        for c in 0..3 {
            assert!(
                (direct_rgb[c] - expected[c]).abs() <= DIRECT_ABS,
                "invalid direct oracle: input={input:?} exposure={exposure} direct={direct_rgb:?} expected={expected:?} abs={DIRECT_ABS}",
            );
            assert!(
                (baked_rgb[c] - direct_rgb[c]).abs() <= BAKED_VS_DIRECT_ABS,
                "internal LUT clipped or distorted scene HDR: input={input:?} exposure={exposure} baked={baked_rgb:?} direct={direct_rgb:?} abs={BAKED_VS_DIRECT_ABS}",
            );
        }
    }
    for pair in observed[1..5].windows(2) {
        assert!(
            pair[1].4[0] - pair[0].4[0] >= MIN_HIGHLIGHT_STEP,
            "baked highlight ramp flattened: {:?} -> {:?}",
            pair[0],
            pair[1],
        );
    }
}

fn hdr_capture_srgb8_to_linear(byte: u8) -> f32 {
    let srgb = byte as f32 / 255.0;
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn insert_hdr_control_lut(asset_manager: &ProjectAssetManager) -> ResourceHandle<TextureMarker> {
    let uri = AssetUri::parse("res://tests/post-process/lut/hdr-external-control").unwrap();
    let id = ResourceId::from_locator(&uri);
    let mut descriptor = user_lut_texture_descriptor();
    descriptor.dimension = RenderImageDimension::D3;
    descriptor.depth_or_array_layers = 2;
    let mut bytes = Vec::new();
    for z in [255, 0] {
        for y in [0, 255] {
            for x in [255, 0] {
                bytes.extend([x, y, z, 255]);
            }
        }
    }
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Texture, uri.clone()),
            TextureAsset::new_rgba8(uri, 2, 2, bytes).with_descriptor(descriptor),
        )
        .expect("external 3D LUT insert");
    ResourceHandle::new(id)
}
