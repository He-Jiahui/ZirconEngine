use super::*;

mod ao_input_qualification;
mod external_history_leases;

#[test]
fn primary_surface_graph_owns_the_terminal_present_pass() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .expect("default forward-plus surface graph compiles");
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == crate::graphics::pipeline::SURFACE_PRESENT_PASS_NAME)
        .expect("primary surface graph must end in surface-present");

    assert_eq!(
        compiled.pass_stage(crate::graphics::pipeline::SURFACE_PRESENT_PASS_NAME),
        Some(RenderPassStage::Present)
    );
    assert!(!pass.flags.allow_culling);
    assert!(pass.flags.has_side_effects);
    assert!(!pass.culled);
    assert_eq!(pass.resources.len(), 1);
    assert_eq!(
        pass.resources[0].access,
        RenderGraphResourceAccessKind::Read
    );
    assert!(
        [
            PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
            PostProcessGraphResourceNames::FINAL_COLOR,
        ]
        .contains(&pass.resources[0].name.as_str())
    );
}

#[test]
fn srgb_texture_target_graph_owns_a_read_only_direct_import_terminal_pass() {
    let compiled = compile_texture_target(RenderGraphCompileTextureTargetFormat::Rgba8UnormSrgb);
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == crate::graphics::pipeline::OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME)
        .expect("sRGB texture graph terminal direct-import pass");

    assert_eq!(
        compiled.pass_stage(crate::graphics::pipeline::OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME),
        Some(RenderPassStage::Present)
    );
    assert!(!pass.flags.allow_culling);
    assert!(pass.flags.has_side_effects);
    assert_eq!(
        pass.executor_id.as_deref(),
        Some(crate::graphics::pipeline::OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID)
    );
    assert_eq!(pass.resources.len(), 1);
    assert_eq!(
        pass.resources[0].access,
        RenderGraphResourceAccessKind::Read
    );
    assert!(
        graph
            .resource_declaration_by_name(
                crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME
            )
            .is_none()
    );
    assert!(graph.passes().iter().all(|candidate| {
        candidate.name != crate::graphics::pipeline::OUTPUT_TARGET_WRITEBACK_PASS_NAME
    }));
}

#[test]
fn linear_texture_target_graph_owns_a_typed_conversion_writeback_pass() {
    let compiled = compile_texture_target(RenderGraphCompileTextureTargetFormat::Rgba8Unorm);
    assert_texture_conversion_writeback_contract(&compiled);
}

#[test]
fn headless_graph_does_not_author_a_surface_present_pass() {
    let mut extract = test_extract();
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("test extract selected camera")
        .target = crate::core::framework::render::RenderCameraTarget::Headless {
        size: crate::core::math::UVec2::new(64, 64),
    };
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .expect("default forward-plus headless graph compiles");

    assert!(
        compiled
            .graph()
            .passes()
            .iter()
            .all(|pass| pass.name != crate::graphics::pipeline::SURFACE_PRESENT_PASS_NAME)
    );
    assert!(
        compiled.graph().passes().iter().all(|pass| {
            pass.name != crate::graphics::pipeline::OUTPUT_TARGET_WRITEBACK_PASS_NAME
        })
    );
    assert!(compiled.graph().passes().iter().all(|pass| {
        pass.name != crate::graphics::pipeline::OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME
    }));
}

fn compile_texture_target(
    format: RenderGraphCompileTextureTargetFormat,
) -> crate::graphics::pipeline::CompiledRenderPipeline {
    let mut extract = test_extract();
    let id = crate::core::resource::ResourceId::from_stable_label(
        "tests/render-graph/texture-output-target",
    );
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("test extract selected camera")
        .target = crate::core::framework::render::RenderCameraTarget::Texture(
        crate::core::resource::ResourceHandle::<crate::core::resource::TextureMarker>::new(id),
    );
    extract.apply_viewport_size(crate::core::math::UVec2::new(96, 54));

    RenderPipelineAsset::default_forward_plus()
        .compile_with_options_and_camera_target(
            &extract,
            &RenderPipelineCompileOptions::default(),
            RenderGraphCompileCameraTargetFingerprint::Texture {
                id,
                width: 96,
                height: 54,
                format,
            },
        )
        .expect("texture output target graph compiles")
}

fn assert_texture_conversion_writeback_contract(
    compiled: &crate::graphics::pipeline::CompiledRenderPipeline,
) {
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == crate::graphics::pipeline::OUTPUT_TARGET_WRITEBACK_PASS_NAME)
        .expect("texture graph terminal writeback pass");
    let output = graph
        .resource_declaration_by_name(
            crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
        )
        .expect("typed output target resource");
    let lifetime = graph
        .resource_lifetime_by_name(crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME)
        .expect("output target lifetime");
    let desc = lifetime
        .external_texture_desc
        .as_ref()
        .expect("output target physical descriptor");
    let access = graph
        .access_metadata_for(
            pass.id,
            output.resource,
            RenderGraphResourceAccessKind::Write,
        )
        .expect("output target write access metadata");
    let source = pass
        .resources
        .iter()
        .find(|resource| resource.access == RenderGraphResourceAccessKind::Read)
        .expect("terminal writeback source access");
    let source_declaration = graph
        .resource_declaration_by_name(&source.name)
        .expect("terminal writeback source declaration");
    let source_access = graph
        .access_metadata_for(
            pass.id,
            source_declaration.resource,
            RenderGraphResourceAccessKind::Read,
        )
        .expect("terminal writeback source access metadata");
    let source_desc = graph
        .resource_lifetime_by_name(&source.name)
        .and_then(|lifetime| lifetime.external_texture_desc.as_ref())
        .expect("terminal source physical descriptor");

    assert_eq!(
        compiled.pass_stage(crate::graphics::pipeline::OUTPUT_TARGET_WRITEBACK_PASS_NAME),
        Some(RenderPassStage::Present)
    );
    assert!(!pass.flags.allow_culling);
    assert!(pass.flags.has_side_effects);
    assert_eq!(
        pass.executor_id.as_deref(),
        Some(crate::graphics::pipeline::OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID)
    );
    assert_eq!(desc.width, 96);
    assert_eq!(desc.height, 54);
    assert_eq!(desc.format, crate::rhi::TextureFormat::Rgba8Unorm);
    assert!(
        desc.usage
            .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT)
    );
    assert_eq!(source_desc.width, 96);
    assert_eq!(source_desc.height, 54);
    assert_eq!(
        source_desc.format,
        crate::rhi::TextureFormat::Rgba8UnormSrgb
    );
    assert!(
        source_desc
            .usage
            .contains(crate::rhi::TextureUsage::SAMPLED)
    );
    assert!(
        source_desc
            .usage
            .contains(crate::rhi::TextureUsage::COPY_SRC)
    );
    assert_eq!(
        source_access.intent,
        RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT)
    );
    assert_eq!(
        access.intent,
        RenderGraphResourceAccessIntent::ColorAttachment
    );
    assert_eq!(
        pass.resources
            .iter()
            .find(|resource| {
                resource.name == crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME
            })
            .and_then(|resource| resource.attachment_ops),
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert!(graph
        .passes()
        .iter()
        .all(|candidate| candidate.name != crate::graphics::pipeline::SURFACE_PRESENT_PASS_NAME));
}

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
    assert_eq!(
        compiled.execution_passes_in_graph_order().count(),
        compiled.graph().passes().len()
    );
}

#[test]
fn compile_propagates_explicit_cull_root_roles_for_history_outputs() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .expect("default forward-plus pipeline compiles");

    let final_color = compiled
        .graph()
        .resource_declaration_by_name(PostProcessGraphResourceNames::FINAL_COLOR)
        .expect("final color declaration");
    let final_color_desc = compiled
        .graph()
        .resource_lifetime_by_name(PostProcessGraphResourceNames::FINAL_COLOR)
        .and_then(|lifetime| lifetime.external_texture_desc.as_ref())
        .expect("final color physical descriptor");
    let ambient_occlusion = compiled
        .graph()
        .resource_declaration_by_name(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
        .expect("ambient occlusion declaration");
    let taa_history_current = compiled
        .graph()
        .resource_declaration_by_name(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT)
        .expect("current TAA history declaration");
    let screen_space_reflection_history = compiled
        .graph()
        .resource_declaration_by_name(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        )
        .expect("screen-space reflection history declaration");
    let hzb_furthest = compiled
        .graph()
        .resource_declaration_by_name(PostProcessGraphResourceNames::HZB_FURTHEST)
        .expect("HZB history source declaration");
    let exposure_previous = compiled
        .graph()
        .resource_declaration_by_name(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS)
        .expect("previous exposure declaration");
    let exposure_current = compiled
        .graph()
        .resource_declaration_by_name(PostProcessGraphResourceNames::EXPOSURE_CURRENT)
        .expect("current exposure declaration");

    assert!(final_color.usage.present);
    assert_eq!(
        final_color_desc.format,
        crate::rhi::TextureFormat::Rgba8UnormSrgb
    );
    assert!(
        final_color_desc
            .usage
            .contains(crate::rhi::TextureUsage::SAMPLED)
    );
    assert!(
        final_color_desc
            .usage
            .contains(crate::rhi::TextureUsage::COPY_SRC)
    );
    assert!(ambient_occlusion.usage.persistent);
    assert_eq!(
        ambient_occlusion.resource.kind(),
        crate::render_graph::RenderGraphResourceKind::External
    );
    assert!(taa_history_current.usage.persistent);
    assert!(screen_space_reflection_history.usage.persistent);
    assert!(hzb_furthest.usage.persistent);
    assert!(exposure_previous.usage.persistent);
    assert!(exposure_current.usage.persistent);
    assert_eq!(
        exposure_previous.resource.kind(),
        crate::render_graph::RenderGraphResourceKind::External
    );
    assert_eq!(
        exposure_current.resource.kind(),
        crate::render_graph::RenderGraphResourceKind::External
    );
    let allocation = compiled.graph().transient_allocation_plan();
    assert_eq!(
        allocation.slot_for(PostProcessGraphResourceNames::HZB_FURTHEST),
        None
    );
    assert_eq!(
        allocation.slot_for(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY),
        None
    );
}

#[test]
fn compile_preserves_exact_external_ssao_uniform_buffer_lease() {
    let mut pipeline = RenderPipelineAsset::default_deferred();
    pipeline
        .renderer
        .features
        .push(crate::graphics::pipeline::RendererFeatureAsset::builtin(
            BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
        ));
    let compiled = pipeline
        .compile(&test_extract())
        .expect("deferred pipeline with qualified SSAO inputs compiles");
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "ssao-evaluate")
        .expect("SSAO evaluate pass");
    let declaration = graph
        .resource_declaration_by_name(PostProcessGraphResourceNames::SSAO_PARAMS)
        .expect("SSAO params declaration");
    let lifetime = graph
        .resource_lifetime_by_name(PostProcessGraphResourceNames::SSAO_PARAMS)
        .expect("SSAO params lifetime");
    let desc = lifetime
        .external_buffer_desc
        .as_ref()
        .expect("SSAO params physical descriptor");
    let expected_size = std::mem::size_of::<crate::graphics::feature::SsaoParams>() as u64;
    let access_id = graph
        .access_id_for(
            pass.id,
            declaration.resource,
            RenderGraphResourceAccessKind::Read,
        )
        .expect("SSAO params exact read access");
    let metadata = graph
        .access_metadata(access_id)
        .expect("SSAO params exact access metadata");
    let packet = graph
        .external_access_packet()
        .access(access_id)
        .expect("SSAO params external access packet");
    let spatial = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "ssao-spatial-denoise")
        .expect("SSAO spatial denoise pass");
    let lighting = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "deferred-lighting")
        .expect("deferred lighting pass");

    assert_eq!(desc.size_bytes, expected_size);
    assert_eq!(
        desc.usage,
        crate::rhi::BufferUsage::UNIFORM | crate::rhi::BufferUsage::COPY_DST
    );
    assert_eq!(
        metadata.range,
        crate::render_graph::RenderGraphResourceAccessRange::Buffer(
            crate::render_graph::RenderGraphBufferRange::new(0, Some(expected_size)),
        )
    );
    assert_eq!(
        metadata.intent,
        crate::render_graph::RenderGraphResourceAccessIntent::UniformBuffer {
            stages: crate::render_graph::RenderGraphShaderStages::COMPUTE,
        }
    );
    assert_eq!(packet.access_id, access_id);
    assert_eq!(packet.key.range, metadata.range);
    assert!(!spatial.culled);
    assert!(spatial.resources.iter().any(|access| {
        access.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
            && access.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(lighting.resources.iter().any(|access| {
        access.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
            && access.access == RenderGraphResourceAccessKind::Read
    }));

    let profile = compiled
        .ambient_occlusion_profile()
        .expect("compiled AO profile");
    let outputs = compiled
        .ambient_occlusion_outputs()
        .expect("compiled AO outputs");
    assert_eq!(
        outputs.indirect_diffuse_ao().resource_name(),
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION
    );
    assert_eq!(
        outputs.indirect_diffuse_ao().producer_pass_name(),
        "ssao-spatial-denoise"
    );
    assert_eq!(outputs.format(), crate::rhi::TextureFormat::Rgba8Unorm);
    assert_eq!(
        outputs.extent(),
        profile.input_qualification().allocation_extent()
    );
    assert_eq!(
        outputs.valid_rect(),
        profile.input_qualification().render_rect()
    );
    assert_eq!(outputs.producer_generation(), profile.pipeline_generation());
}

#[test]
fn default_deferred_lighting_does_not_read_unproduced_ambient_occlusion() {
    let compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .expect("default deferred pipeline compiles without SSAO");
    let lighting = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "deferred-lighting")
        .expect("deferred lighting pass");

    assert!(compiled.ambient_occlusion_profile().is_none());
    assert!(compiled.ambient_occlusion_outputs().is_none());
    assert!(
        lighting
            .resources
            .iter()
            .all(|access| { access.name != PostProcessGraphResourceNames::AMBIENT_OCCLUSION })
    );
}

#[test]
fn compile_preserves_compute_workload_from_feature_descriptor() {
    let policy = crate::render_graph::RenderGraphComputePipelineFallbackPolicy::last_good(
        "ambient-occlusion.evaluate",
        2,
    );
    let workload = RenderGraphComputeWorkload::per_pixel(
        "zircon-ssao-pipeline",
        [8, 8, 1],
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        [8, 8],
    )
    .with_pipeline_fallback_policy(policy.clone());
    let pipeline = compute_workload_test_pipeline(workload);

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "ssao-evaluate")
        .unwrap();
    let workload = pass.compute_workload.as_ref().unwrap();

    assert_eq!(workload.pipeline_label, "zircon-ssao-pipeline");
    assert_eq!(workload.pipeline_fallback_policy, policy);
    assert_eq!(workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string(),
            local_size: [8, 8],
        }
    );
}

#[test]
fn compile_rejects_unversioned_compute_pipeline_fallback_family() {
    let invalid_policy = crate::render_graph::RenderGraphComputePipelineFallbackPolicy::last_good(
        "ambient-occlusion.evaluate",
        0,
    );
    let workload = RenderGraphComputeWorkload::per_pixel(
        "zircon-ssao-pipeline",
        [8, 8, 1],
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        [8, 8],
    )
    .with_pipeline_fallback_policy(invalid_policy);

    let error = compute_workload_test_pipeline(workload)
        .compile(&test_extract())
        .expect_err("unversioned last-good family must fail during pipeline compile");

    assert!(error.contains("invalid compute pipeline fallback policy"));
    assert!(error.contains("requires a non-zero interface generation"));
}

fn compute_workload_test_pipeline(workload: RenderGraphComputeWorkload) -> RenderPipelineAsset {
    RenderPipelineAsset {
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
                    vec![
                        RenderFeaturePassDescriptor::new(
                            RenderPassStage::AmbientOcclusion,
                            "ssao-evaluate",
                            QueueLane::AsyncCompute,
                        )
                        .with_executor_id("compute.generic")
                        .with_compute_workload(workload)
                        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
                        .write_storage_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION),
                    ],
                ),
            )],
        },
    }
}

#[test]
fn compile_options_append_environment_ibl_bake_graph_passes() {
    let request = crate::core::framework::render::IblBakeArtifactRequest::new(
        crate::core::framework::render::ProceduralSkyParams::default_gradient().ibl_bake_key(),
        16,
        5,
    )
    .with_required_contents(crate::core::framework::render::IblBakeArtifactContents::SH9);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_environment_ibl_bake_request(request),
        )
        .unwrap();

    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == crate::graphics::scene::IBL_BAKE_IRRADIANCE_SH9_PASS)
        .expect("IBL SH9 graph pass should compile from environment bake request");

    assert_eq!(compiled.environment_ibl_bake_request, Some(request));
    assert_eq!(
        compiled.pass_stage(crate::graphics::scene::IBL_BAKE_IRRADIANCE_SH9_PASS),
        Some(RenderPassStage::AmbientOcclusion)
    );
    assert_eq!(
        pass.executor_id.as_deref(),
        Some(crate::graphics::scene::IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID)
    );
    assert_eq!(pass.queue, QueueLane::AsyncCompute);
    assert_pass_reads(
        &compiled,
        crate::graphics::scene::IBL_BAKE_IRRADIANCE_SH9_PASS,
        crate::graphics::scene::IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
    );
}

#[test]
fn compile_without_environment_ibl_request_keeps_bake_graph_absent() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    assert!(compiled.environment_ibl_bake_request.is_none());
    assert!(
        !compiled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name == crate::graphics::scene::IBL_BAKE_IRRADIANCE_SH9_PASS),
        "IBL bake graph pass should be opt-in through compile options"
    );
}

#[test]
fn compile_skips_core_particle_pass_when_particle_sprites_miss_selected_camera_layers() {
    let mut extract = test_extract();
    let mut descriptor =
        CameraRenderDescriptor::from_camera_payload(Some(7), ViewportCameraSnapshot::default());
    descriptor.culling_mask = RenderLayerSet::layer(2);
    extract.select_camera_descriptor(descriptor);
    extract
        .particles
        .sprites
        .push(RenderParticleSpriteSnapshot {
            entity: 1,
            stable_sprite_key: 1,
            position: Vec3::ZERO,
            size: 1.0,
            color: Vec4::ONE,
            intensity: 1.0,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1 << 1),
            ..RenderParticleSpriteSnapshot::default()
        });

    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    assert!(
        !compiled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name == "particle-render"),
        "hidden particle sprites should not auto-enable the core particle pass"
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
    let reflection_pyramid_coarse_alias = reflection_pyramid_coarse
        .texture_view_alias
        .expect("SSR coarse pyramid is a graph-declared parent mip view");
    assert_eq!(
        reflection_pyramid_coarse_alias.range,
        crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1)
    );
    assert_eq!(
        reflection_pyramid_coarse_alias.parent,
        match compiled
            .graph()
            .resource_declaration_by_name(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
            )
            .expect("SSR reflection pyramid declaration")
            .resource
        {
            crate::render_graph::RenderGraphResource::TransientTexture(parent) => parent,
            resource => panic!("SSR reflection pyramid has unexpected resource kind: {resource:?}"),
        }
    );
    assert_eq!(
        compiled.graph().transient_allocation_plan().slot_for(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
        ),
        None
    );
}
