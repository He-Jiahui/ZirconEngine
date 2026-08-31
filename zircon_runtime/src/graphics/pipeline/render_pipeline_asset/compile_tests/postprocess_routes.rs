use super::*;

mod plugin_inputs;

#[test]
fn compile_describes_color_lut_as_rgba16float_3d_transient_when_enabled() {
    let mut extract = test_extract();
    extract.post_process.color_grading.exposure = 1.05;
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        false,
        false,
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let color_lut = texture_lifetime(&compiled, PostProcessGraphResourceNames::COLOR_LUT);
    let color_lut_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "color-lut-bake")
        .expect("enabled color grading should compile the color LUT bake pass");

    assert_eq!(
        (color_lut.width, color_lut.height, color_lut.depth),
        (
            crate::core::framework::render::COLOR_LUT_SIZE_DEFAULT,
            crate::core::framework::render::COLOR_LUT_SIZE_DEFAULT,
            crate::core::framework::render::COLOR_LUT_SIZE_DEFAULT,
        )
    );
    assert_eq!(color_lut.dimension, crate::rhi::TextureDimension::D3);
    assert_eq!(color_lut.format, crate::rhi::TextureFormat::Rgba16Float);
    assert!(color_lut.usage.contains(crate::rhi::TextureUsage::STORAGE));
    assert!(color_lut.usage.contains(crate::rhi::TextureUsage::COPY_DST));
    let workload = color_lut_pass.compute_workload.as_ref().unwrap();
    assert_eq!(workload.pipeline_label, "zircon-color-lut-bake-pipeline");
    assert_eq!(workload.workgroup_size, [4, 4, 4]);
    assert_eq!(
        workload.dispatch_extent,
        crate::render_graph::RenderGraphComputeDispatchExtent::Fixed([8, 8, 8])
    );
    assert!(
        color_lut_pass
            .resources
            .iter()
            .any(|resource| resource.name == PostProcessGraphResourceNames::EXPOSURE_CURRENT)
    );
}

#[test]
fn compile_routes_bloom_extract_after_split_scene_color_passes() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings {
            intensity: 0.6,
            ..Default::default()
        },
        &extract.post_process.color_grading,
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 8,
            },
            blur: RenderBlurSettings { radius: 3.0 },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let bloom_extract = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "bloom-extract")
        .expect("enabled bloom should compile the bloom extract pass");
    assert!(bloom_extract.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::MOTION_BLURRED
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(!bloom_extract.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::SCENE_COLOR
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(bloom_extract.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::BLOOM
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
    let uber = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("enabled post-process stack should retain uber");
    let output_transfer = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("enabled post-process stack should retain output transfer");
    assert!(uber.dependencies.contains(&bloom_extract.id));
    assert!(output_transfer.dependencies.contains(&uber.id));
    assert!(graph_pass_index(&compiled, "bloom-extract") < graph_pass_index(&compiled, "uber"));
    assert!(graph_pass_index(&compiled, "uber") < graph_pass_index(&compiled, "output-transfer"));
    let bloom = texture_lifetime(&compiled, PostProcessGraphResourceNames::BLOOM);
    assert_eq!(bloom.format, crate::rhi::TextureFormat::Rg11b10Ufloat);
}

#[test]
fn compile_orders_unstacked_bloom_provider_before_uber() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .expect("the default pipeline must order Bloom before its consumer");
    let bloom_extract = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "bloom-extract")
        .expect("the unstacked default pipeline should retain Bloom");
    let uber = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("the unstacked default pipeline should retain uber");
    let output_transfer = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("the unstacked default pipeline should retain output transfer");

    assert!(uber.dependencies.contains(&bloom_extract.id));
    assert!(output_transfer.dependencies.contains(&uber.id));
    assert!(graph_pass_index(&compiled, "bloom-extract") < graph_pass_index(&compiled, "uber"));
    assert!(graph_pass_index(&compiled, "uber") < graph_pass_index(&compiled, "output-transfer"));
}

#[test]
fn compile_removes_bloom_consumer_when_compile_options_disable_the_feature() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &RenderBloomSettings {
            intensity: 0.6,
            ..Default::default()
        },
        &extract.post_process.color_grading,
        false,
        false,
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_post_process_stack(stack),
        )
        .expect("disabled Bloom must not leave an unproduced uber input");

    assert!(!graph_has_pass(&compiled, "bloom-extract"));
    assert_pass_does_not_read(&compiled, "uber", PostProcessGraphResourceNames::BLOOM);
}

#[test]
fn compile_removes_bloom_consumer_when_pipeline_has_no_bloom_provider() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &RenderBloomSettings {
            intensity: 0.6,
            ..Default::default()
        },
        &extract.post_process.color_grading,
        false,
        false,
    );
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .retain(|feature| !feature.is_builtin(BuiltinRenderFeature::Bloom));
    let compiled = pipeline
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .expect("missing Bloom provider must not leave an unproduced uber input");

    assert!(!graph_has_pass(&compiled, "bloom-extract"));
    assert_pass_does_not_read(&compiled, "uber", PostProcessGraphResourceNames::BLOOM);
}

#[test]
fn compile_bloom_uses_the_motion_blur_version_without_an_exposure_ordering_rule() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale_phases(
        &RenderBloomSettings {
            intensity: 0.6,
            ..Default::default()
        },
        &extract.post_process.color_grading,
        RenderExposureSettings::histogram(),
        &RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 8,
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
        false,
        false,
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let motion_blur = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-blur")
        .expect("motion blur pass");
    let bloom = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "bloom-extract")
        .expect("bloom extract pass");
    let exposure_histogram = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "exposure-histogram")
        .expect("exposure histogram pass");

    assert!(bloom.dependencies.contains(&motion_blur.id));
    assert!(!bloom.dependencies.contains(&exposure_histogram.id));
}

#[test]
fn compile_routes_blur_split_through_uber_and_output_transfer() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &extract.post_process.color_grading,
        &RenderPostProcessEffectStackSettings {
            blur: RenderBlurSettings { radius: 3.0 },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    assert_pass_reads(
        &compiled,
        "blur",
        PostProcessGraphResourceNames::SCENE_COLOR,
    );
    assert_pass_writes(&compiled, "blur", PostProcessGraphResourceNames::BLURRED);
    assert_pass_reads(&compiled, "uber", PostProcessGraphResourceNames::BLURRED);
    assert_pass_does_not_read(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::SCENE_COLOR,
    );
    assert_pass_writes(&compiled, "uber", PostProcessGraphResourceNames::TONEMAPPED);
    assert_pass_reads(
        &compiled,
        "output-transfer",
        PostProcessGraphResourceNames::TONEMAPPED,
    );
}

#[test]
fn compile_keeps_split_postprocess_passes_before_exposure_when_they_do_not_sample_exposure() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &extract.post_process.color_grading,
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 8,
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    for pass_name in ["depth-of-field", "motion-blur"] {
        assert_pass_does_not_read(
            &compiled,
            pass_name,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
        );
    }
    assert_pass_reads(
        &compiled,
        "exposure-resolve",
        PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
    );
    assert_pass_reads(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::EXPOSURE_CURRENT,
    );
}

#[test]
fn compile_declares_uber_light_list_frame_resource_for_default_stack() {
    let extract = test_extract();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_post_process_stack(PostProcessStackDescriptor::default()),
        )
        .unwrap();
    let uber = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("default postprocess stack should compile uber");

    assert!(uber.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::LIGHT_LIST
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    let lifetime = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == PostProcessGraphResourceNames::LIGHT_LIST)
        .expect("live light-list resource should have a graph lifetime");
    assert!(matches!(
        lifetime.kind,
        RenderGraphResourceKind::TransientBuffer | RenderGraphResourceKind::External
    ));
}

#[test]
fn compile_declares_uber_light_list_as_external_when_clustered_lighting_is_disabled() {
    let extract = test_extract();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_post_process_stack(PostProcessStackDescriptor::default()),
        )
        .unwrap();
    let uber = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("default postprocess stack should compile uber");

    assert!(uber.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::LIGHT_LIST
            && resource.kind == RenderGraphResourceKind::External
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    let lifetime = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == PostProcessGraphResourceNames::LIGHT_LIST)
        .expect("disabled clustered lighting should keep light-list as a frame external");
    assert_eq!(lifetime.kind, RenderGraphResourceKind::External);
}

#[test]
fn compile_routes_output_transfer_through_fxaa_terminal_input() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_anti_alias(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        false,
        false,
        &AntiAliasSettings::fxaa(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let output_transfer = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("postprocess stack should compile output transfer");
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let fxaa = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "fxaa")
        .expect("enabled terminal FXAA should compile the anti-alias pass");
    assert!(
        !fxaa.flags.has_side_effects,
        "output transfer consumes the terminal FXAA result and owns the present root"
    );
    assert!(graph_pass_index(&compiled, "fxaa") < graph_pass_index(&compiled, "output-transfer"));
    assert!(fxaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::TONEMAPPED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(fxaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let terminal_input =
        texture_lifetime(&compiled, PostProcessGraphResourceNames::FINAL_COMPOSITED);
    assert_eq!(
        terminal_input.format,
        crate::rhi::TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(terminal_input.sample_count, 1);
    assert!(
        terminal_input
            .usage
            .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT)
    );
    assert!(
        terminal_input
            .usage
            .contains(crate::rhi::TextureUsage::SAMPLED)
    );
}

#[test]
fn compile_routes_output_transfer_through_smaa_terminal_input() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_anti_alias(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        false,
        false,
        &AntiAliasSettings::smaa(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    let output_transfer = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("postprocess stack should compile output transfer");
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let smaa = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "smaa")
        .expect("enabled terminal SMAA should compile the anti-alias pass");
    assert!(
        !smaa.flags.has_side_effects,
        "output transfer consumes the terminal SMAA result and owns the present root"
    );
    assert!(graph_pass_index(&compiled, "smaa") < graph_pass_index(&compiled, "output-transfer"));
    assert!(smaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::TONEMAPPED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(smaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
    assert!(
        !compiled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name == "fxaa")
    );

    let terminal_input =
        texture_lifetime(&compiled, PostProcessGraphResourceNames::FINAL_COMPOSITED);
    assert_eq!(
        terminal_input.format,
        crate::rhi::TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(terminal_input.sample_count, 1);
    assert!(
        terminal_input
            .usage
            .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT)
    );
    assert!(
        terminal_input
            .usage
            .contains(crate::rhi::TextureUsage::SAMPLED)
    );
}

#[test]
fn compile_describes_hzb_as_half_power_of_two_mip_chain() {
    let mut extract = test_extract();
    extract.apply_viewport_size(crate::core::math::UVec2::new(1923, 1081));
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    let hzb_furthest = texture_lifetime(&compiled, PostProcessGraphResourceNames::HZB_FURTHEST);
    let hzb_indirect_args = compiled
        .graph()
        .resource_lifetime_by_name("mesh.indirect-args")
        .expect("HZB occlusion indirect args external lifetime");
    let hzb_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "hzb-build")
        .expect("default 3D pipelines should build the shared HZB resource");

    assert_eq!((hzb_furthest.width, hzb_furthest.height), (1024, 1024));
    assert_eq!(hzb_furthest.mip_levels, 11);
    assert_eq!(hzb_furthest.format, crate::rhi::TextureFormat::Rgba16Float);
    assert_eq!(
        hzb_indirect_args.external_binding,
        RenderGraphExternalResourceBinding::required_buffer()
    );
    assert!(!hzb_pass.culled);
    assert!(hzb_pass.flags.has_side_effects);
}

fn graph_has_pass(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
) -> bool {
    compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == pass_name)
}

fn graph_pass_index(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
) -> usize {
    compiled
        .graph()
        .passes()
        .iter()
        .position(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"))
}

fn graph_pass<'a>(
    compiled: &'a crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
) -> &'a crate::render_graph::CompiledRenderPass {
    compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"))
}
