use crate::core::framework::render::{
    AntiAliasSettings, PostProcessGraphResourceNames, PostProcessStackDescriptor,
    RenderBloomSettings, RenderBlurSettings, RenderDepthOfFieldSettings, RenderFrameExtract,
    RenderMotionBlurSettings, RenderPhase, RenderPipelineHandle,
    RenderPostProcessEffectStackSettings, RenderWorldSnapshotHandle,
};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
};
use crate::graphics::pipeline::{
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererAsset,
};
use crate::render_graph::{
    QueueLane, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};
use crate::scene::world::World;

#[test]
fn compile_preserves_renderer_stage_for_each_graph_pass() {
    let pipeline = RenderPipelineAsset {
        handle: RenderPipelineHandle::new(77),
        revision: 1,
        name: "stage-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![
            crate::core::framework::render::RenderPhase::Prepass,
            crate::core::framework::render::RenderPhase::Transparent3d,
        ],
        renderer: RendererAsset {
            name: "stage-test-renderer".to_string(),
            stages: vec![
                RenderPassStage::DepthPrepass,
                RenderPassStage::Transparent3d,
            ],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "stage-test-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![
                        RenderFeaturePassDescriptor::new(
                            RenderPassStage::Transparent3d,
                            "particle-render",
                            QueueLane::Graphics,
                        )
                        .with_executor_id("particle.transparent"),
                        RenderFeaturePassDescriptor::new(
                            RenderPassStage::DepthPrepass,
                            "depth-prepass",
                            QueueLane::Graphics,
                        )
                        .with_executor_id("mesh.depth-prepass"),
                    ],
                ),
            )],
        },
    };

    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled.pass_stage("depth-prepass"),
        Some(RenderPassStage::DepthPrepass)
    );
    assert_eq!(
        compiled.pass_stage("particle-render"),
        Some(RenderPassStage::Transparent3d)
    );
    assert_eq!(compiled.pass_stage("missing-pass"), None);
    assert_eq!(compiled.pass_stages.len(), compiled.graph.passes().len());
}

#[test]
fn compile_preserves_compute_workload_from_feature_descriptor() {
    let pipeline = RenderPipelineAsset {
        handle: RenderPipelineHandle::new(78),
        revision: 1,
        name: "compute-workload-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::PostProcess],
        renderer: RendererAsset {
            name: "compute-workload-renderer".to_string(),
            stages: vec![RenderPassStage::AmbientOcclusion],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "compute-workload-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::AmbientOcclusion,
                        "ssao-evaluate",
                        QueueLane::AsyncCompute,
                    )
                    .with_executor_id("ao.ssao-evaluate")
                    .with_compute_workload(RenderGraphComputeWorkload::viewport(
                        "zircon-ssao-pipeline",
                        [8, 8, 1],
                    ))
                    .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
                    .write_storage_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)],
                ),
            )],
        },
    };

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "ssao-evaluate")
        .unwrap();
    let workload = pass.compute_workload.as_ref().unwrap();

    assert_eq!(workload.pipeline_label, "zircon-ssao-pipeline");
    assert_eq!(workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Viewport
    );
}

#[test]
fn compile_describes_hzb_and_ssr_reflection_pyramids_as_mip_chain_transients() {
    let mut extract = test_extract();
    extract.apply_viewport_size(crate::core::math::UVec2::new(128, 64));
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    let hzb_furthest = texture_lifetime(&compiled, PostProcessGraphResourceNames::HZB_FURTHEST);
    let reflection_pyramid = texture_lifetime(
        &compiled,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
    );
    let reflection_pyramid_coarse = texture_lifetime(
        &compiled,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
    );

    assert_eq!((hzb_furthest.width, hzb_furthest.height), (64, 32));
    assert_eq!(hzb_furthest.mip_levels, 7);
    assert_eq!(
        (reflection_pyramid.width, reflection_pyramid.height),
        (64, 32)
    );
    assert_eq!(reflection_pyramid.mip_levels, 7);
    assert_eq!(
        reflection_pyramid.format,
        crate::rhi::TextureFormat::Rg11b10Ufloat
    );
    assert_eq!(
        (
            reflection_pyramid_coarse.width,
            reflection_pyramid_coarse.height
        ),
        (32, 16)
    );
    assert_eq!(reflection_pyramid_coarse.mip_levels, 1);
    assert_eq!(
        reflection_pyramid_coarse.format,
        crate::rhi::TextureFormat::Rg11b10Ufloat
    );
}

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
        .graph
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
    assert!(color_lut_pass
        .resources
        .iter()
        .any(|resource| resource.name == PostProcessGraphResourceNames::EXPOSURE_CURRENT));
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
        .graph
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
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("default postprocess stack should compile uber");

    assert!(uber.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::LIGHT_LIST
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    let lifetime = compiled
        .graph
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
        .graph
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
        .graph
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
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("postprocess stack should compile output transfer");
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
    assert!(!output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let fxaa = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "fxaa")
        .expect("enabled terminal FXAA should compile the anti-alias pass");
    assert!(fxaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(fxaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let terminal_input =
        texture_lifetime(&compiled, PostProcessGraphResourceNames::FINAL_COMPOSITED);
    assert_eq!(
        terminal_input.format,
        crate::rhi::TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(terminal_input.sample_count, 1);
    assert!(terminal_input
        .usage
        .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT));
    assert!(terminal_input
        .usage
        .contains(crate::rhi::TextureUsage::SAMPLED));
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
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("postprocess stack should compile output transfer");
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
    assert!(!output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let smaa = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "smaa")
        .expect("enabled terminal SMAA should compile the anti-alias pass");
    assert!(smaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.kind == RenderGraphResourceKind::TransientTexture
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(smaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
    assert!(!compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "fxaa"));

    let terminal_input =
        texture_lifetime(&compiled, PostProcessGraphResourceNames::FINAL_COMPOSITED);
    assert_eq!(
        terminal_input.format,
        crate::rhi::TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(terminal_input.sample_count, 1);
    assert!(terminal_input
        .usage
        .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT));
    assert!(terminal_input
        .usage
        .contains(crate::rhi::TextureUsage::SAMPLED));
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
        .graph
        .resource_lifetime_by_name("mesh.indirect-args")
        .expect("HZB occlusion indirect args external lifetime");
    let hzb_pass = compiled
        .graph
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

#[test]
fn compile_preserves_required_external_texture_binding() {
    let pipeline = RenderPipelineAsset {
        handle: RenderPipelineHandle::new(80),
        revision: 1,
        name: "required-external-texture-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::PostProcess],
        renderer: RendererAsset {
            name: "required-external-texture-renderer".to_string(),
            stages: vec![RenderPassStage::PostProcess],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "required-external-texture-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "history-consumer",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("test.history-consumer")
                    .with_side_effects()
                    .read_required_external_texture("history.previous-color")],
                ),
            )],
        },
    };

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let lifetime = compiled
        .graph
        .resource_lifetime_by_name("history.previous-color")
        .expect("required external texture lifetime");

    assert_eq!(
        lifetime.external_binding,
        RenderGraphExternalResourceBinding::required_texture()
    );
}

#[test]
fn compile_rejects_conflicting_required_external_texture_and_buffer_binding() {
    let pipeline = RenderPipelineAsset {
        handle: RenderPipelineHandle::new(81),
        revision: 1,
        name: "required-external-conflict-test".to_string(),
        core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::PostProcess],
        renderer: RendererAsset {
            name: "required-external-conflict-renderer".to_string(),
            stages: vec![RenderPassStage::PostProcess],
            features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "required-external-conflict-feature",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "conflicting-external",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("test.conflicting-external")
                    .with_side_effects()
                    .read_required_external_texture("shared.external")
                    .write_required_external_buffer("shared.external")],
                ),
            )],
        },
    };

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `shared.external` has conflicting external resource binding"),
        "{error}"
    );
}

#[test]
fn compile_rejects_compute_workload_on_non_compute_queue() {
    let pipeline =
        RenderPipelineAsset {
            handle: RenderPipelineHandle::new(79),
            revision: 1,
            name: "compute-workload-queue-test".to_string(),
            core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
            phase_mapping: vec![RenderPhase::PostProcess],
            renderer: RendererAsset {
                name: "compute-workload-queue-renderer".to_string(),
                stages: vec![RenderPassStage::PostProcess],
                features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                    RenderFeatureDescriptor::new(
                        "invalid-compute-workload-feature",
                        Vec::new(),
                        Vec::new(),
                        vec![RenderFeaturePassDescriptor::new(
                            RenderPassStage::PostProcess,
                            "bad-compute",
                            QueueLane::Graphics,
                        )
                        .with_executor_id("bad.compute")
                        .with_compute_workload(
                            RenderGraphComputeWorkload::fixed("bad-pipeline", [1, 1, 1], [1, 1, 1]),
                        )],
                    ),
                )],
            },
        };

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains(
            "feature descriptor `invalid-compute-workload-feature` pass `bad-compute` cannot declare compute workload on `Graphics` queue"
        ),
        "{error}"
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn assert_pass_reads(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
) {
    let pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"));
    assert!(
        pass.resources.iter().any(|resource| {
            resource.name == resource_name && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "`{pass_name}` should read `{resource_name}`"
    );
}

fn assert_pass_does_not_read(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
) {
    let pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing graph pass `{pass_name}`"));
    assert!(
        !pass.resources.iter().any(|resource| {
            resource.name == resource_name && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "`{pass_name}` should not read `{resource_name}`"
    );
}

fn texture_lifetime<'a>(
    compiled: &'a crate::graphics::pipeline::CompiledRenderPipeline,
    name: &str,
) -> &'a crate::rhi::TextureDesc {
    let lifetime = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == name)
        .unwrap_or_else(|| panic!("missing graph resource lifetime `{name}`"));
    match &lifetime.desc {
        crate::render_graph::RenderGraphResourceDesc::Texture(desc) => desc,
        other => panic!("expected texture desc for `{name}`, got {other:?}"),
    }
}
