use super::*;

#[test]
fn compile_preserves_exact_ssao_history_texture_lease() {
    let mut extract = test_extract();
    extract.apply_viewport_size(crate::core::math::UVec2::new(1920, 1080));
    extract.view.apply_view_family_pipeline(
        crate::core::framework::render::RenderViewFamilyPipeline::resolve(
            crate::core::math::UVec2::new(1920, 1080),
            crate::core::framework::render::RenderResolutionPolicy::with_temporal_fractions(
                0.5, 0.75,
            ),
            crate::core::framework::render::RenderUpscalerKind::Temporal,
        ),
    );
    let mut pipeline = RenderPipelineAsset::default_deferred();
    pipeline
        .renderer
        .features
        .push(crate::graphics::pipeline::RendererFeatureAsset::builtin(
            BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
        ));
    let compiled = pipeline
        .compile(&extract)
        .expect("deferred pipeline with qualified SSAO inputs compiles");
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "ssao-evaluate")
        .expect("SSAO evaluate pass");
    let declaration = graph
        .resource_declaration_by_name(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION,
        )
        .expect("previous SSAO history declaration");
    let lifetime = graph
        .resource_lifetime_by_name(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION,
        )
        .expect("previous SSAO history lifetime");
    let desc = lifetime
        .external_texture_desc
        .as_ref()
        .expect("previous SSAO history physical descriptor");
    let access_id = graph
        .access_id_for(
            pass.id,
            declaration.resource,
            RenderGraphResourceAccessKind::Read,
        )
        .expect("previous SSAO history exact read access");
    let metadata = graph
        .access_metadata(access_id)
        .expect("previous SSAO history exact access metadata");
    let packet = graph
        .external_access_packet()
        .access(access_id)
        .expect("previous SSAO history external access packet");

    assert_eq!(desc.format, crate::rhi::TextureFormat::Rgba8Unorm);
    assert_eq!((desc.width, desc.height), (720, 408));
    assert_eq!(
        desc.usage,
        crate::rhi::TextureUsage::SAMPLED | crate::rhi::TextureUsage::RENDER_ATTACHMENT
    );
    assert_eq!(
        metadata.range,
        crate::render_graph::RenderGraphResourceAccessRange::Texture(
            crate::render_graph::RenderGraphTextureSubresourceRange::full(),
        )
    );
    assert_eq!(
        metadata.intent,
        crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
            crate::render_graph::RenderGraphShaderStages::COMPUTE,
        )
    );
    assert_eq!(packet.access_id, access_id);
    assert_eq!(packet.key.range, metadata.range);
}

#[test]
fn compile_preserves_exact_external_exposure_buffer_leases() {
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
        .expect("default forward-plus pipeline compiles");
    let graph = compiled.graph();
    let exposure_size = u64::from(crate::core::framework::render::EXPOSURE_BUFFER_WORD_COUNT)
        * std::mem::size_of::<f32>() as u64;
    let compiled_range = crate::render_graph::RenderGraphResourceAccessRange::Buffer(
        crate::render_graph::RenderGraphBufferRange::new(0, Some(exposure_size)),
    );

    for resource_name in [
        PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
        PostProcessGraphResourceNames::EXPOSURE_CURRENT,
    ] {
        let lifetime = graph
            .resource_lifetime_by_name(resource_name)
            .expect("exposure external lifetime");
        let desc = lifetime
            .external_buffer_desc
            .as_ref()
            .expect("exposure external buffer descriptor");
        assert_eq!(desc.size_bytes, exposure_size);
        assert!(desc.usage.contains(crate::rhi::BufferUsage::STORAGE));
        assert!(desc.usage.contains(crate::rhi::BufferUsage::COPY_SRC));
        assert!(desc.usage.contains(crate::rhi::BufferUsage::COPY_DST));
    }

    let expected_accesses = [
        (
            "exposure-resolve",
            PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
            RenderGraphResourceAccessKind::Read,
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        ),
        (
            "exposure-resolve",
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        ),
        (
            "scene-composite",
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            RenderGraphResourceAccessKind::Read,
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::FRAGMENT),
        ),
        (
            "color-lut-bake",
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            RenderGraphResourceAccessKind::Read,
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        ),
        (
            "uber",
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            RenderGraphResourceAccessKind::Read,
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::FRAGMENT),
        ),
    ];

    for (pass_name, resource_name, access, intent) in expected_accesses {
        let pass = graph
            .passes()
            .iter()
            .find(|pass| pass.name == pass_name)
            .unwrap_or_else(|| panic!("missing exposure consumer pass `{pass_name}`"));
        let resource = graph
            .resource_declaration_by_name(resource_name)
            .expect("exposure resource declaration")
            .resource;
        let access_id = graph
            .access_id_for(pass.id, resource, access)
            .expect("exact exposure access id");
        let metadata = graph
            .access_metadata(access_id)
            .expect("exact exposure access metadata");
        let packet = graph
            .external_access_packet()
            .access(access_id)
            .expect("exact exposure external lease packet");

        assert_eq!(metadata.range, compiled_range);
        assert_eq!(metadata.intent, intent);
        assert!(matches!(
            &packet.desc,
            crate::render_graph::RenderGraphResourceDesc::Buffer(desc)
                if desc.size_bytes == exposure_size
        ));
    }
}

#[test]
fn compile_preserves_exact_external_taa_history_texture_leases() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .expect("default forward-plus pipeline compiles");
    let graph = compiled.graph();
    let taa_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "taa-resolve")
        .expect("TAA resolve pass");
    assert!(!taa_pass.culled);
    let compiled_range = crate::render_graph::RenderGraphResourceAccessRange::Texture(
        crate::render_graph::RenderGraphTextureSubresourceRange {
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
            aspect: crate::render_graph::RenderGraphTextureAspect::All,
        },
    );

    for (resource_name, access_kind, intent) in [
        (
            PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
            RenderGraphResourceAccessKind::Read,
            crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
                crate::render_graph::RenderGraphShaderStages::FRAGMENT,
            ),
        ),
        (
            PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
            RenderGraphResourceAccessKind::Write,
            crate::render_graph::RenderGraphResourceAccessIntent::ColorAttachment,
        ),
    ] {
        let lifetime = graph
            .resource_lifetime_by_name(resource_name)
            .expect("TAA history lifetime");
        let desc = lifetime
            .external_texture_desc
            .as_ref()
            .expect("TAA history physical descriptor");
        assert_eq!(desc.format, crate::rhi::TextureFormat::Rgba16Float);
        assert_eq!(desc.mip_levels, 1);
        assert!(desc.usage.contains(crate::rhi::TextureUsage::SAMPLED));
        assert!(
            desc.usage
                .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT)
        );

        let access_index = taa_pass
            .resources
            .iter()
            .position(|access| access.name == resource_name && access.access == access_kind)
            .expect("TAA history access");
        let access_id = graph.access_id_at(taa_pass.id, access_index).unwrap();
        let key = graph
            .versioned_access_key(access_id)
            .expect("TAA history versioned access key");
        assert_eq!(key.range, compiled_range);
        assert_eq!(key.intent, intent);
        let packet = graph
            .external_access_packet()
            .access(access_id)
            .expect("TAA history external access packet");
        assert!(matches!(
            &packet.desc,
            crate::render_graph::RenderGraphResourceDesc::Texture(_)
        ));
    }
}

#[test]
fn compile_preserves_exact_external_ssr_history_texture_lease() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .expect("default forward-plus pipeline compiles");
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "screen-space-reflection-resolve")
        .expect("SSR resolve pass");
    assert!(!pass.culled);
    let resource_name = PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION;
    let lifetime = graph
        .resource_lifetime_by_name(resource_name)
        .expect("previous SSR history lifetime");
    let desc = lifetime
        .external_texture_desc
        .as_ref()
        .expect("previous SSR history descriptor");
    assert_eq!(desc.format, crate::rhi::TextureFormat::Rgba16Float);
    assert_eq!(desc.mip_levels, 1);
    assert!(desc.usage.contains(crate::rhi::TextureUsage::SAMPLED));

    let access_index = pass
        .resources
        .iter()
        .position(|access| {
            access.name == resource_name && access.access == RenderGraphResourceAccessKind::Read
        })
        .expect("previous SSR history read");
    let access_id = graph.access_id_at(pass.id, access_index).unwrap();
    let key = graph
        .versioned_access_key(access_id)
        .expect("previous SSR history access key");
    assert_eq!(
        key.range,
        crate::render_graph::RenderGraphResourceAccessRange::Texture(
            crate::render_graph::RenderGraphTextureSubresourceRange {
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                aspect: crate::render_graph::RenderGraphTextureAspect::All,
            },
        )
    );
    assert_eq!(
        key.intent,
        crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
            crate::render_graph::RenderGraphShaderStages::FRAGMENT,
        )
    );
    assert!(graph.external_access_packet().access(access_id).is_some());
}

#[test]
fn compile_preserves_exact_external_hzb_history_texture_lease() {
    let extract = test_extract();
    let hzb_plan = crate::graphics::visibility::HzbBuilder::new(
        extract
            .view
            .view_family_pipeline()
            .output_target_for_phase(
                crate::core::framework::render::RenderPipelinePhase::SceneLinear,
            )
            .expect("SceneLinear output target")
            .allocation_extent(),
    )
    .build_plan();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .expect("default forward-plus pipeline compiles");
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "hzb-occlusion-cull")
        .expect("HZB occlusion cull pass");
    assert!(!pass.culled);
    let resource_name = PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST;
    let lifetime = graph
        .resource_lifetime_by_name(resource_name)
        .expect("previous HZB history lifetime");
    let desc = lifetime
        .external_texture_desc
        .as_ref()
        .expect("previous HZB history descriptor");
    assert_eq!(
        (desc.width, desc.height, desc.mip_levels),
        (hzb_plan.hzb_size.x, hzb_plan.hzb_size.y, hzb_plan.mip_count)
    );
    assert_eq!(desc.format, crate::rhi::TextureFormat::Rgba16Float);
    assert_eq!(desc.sample_count, 1);
    assert_eq!(desc.usage, crate::rhi::TextureUsage::SAMPLED);

    let access_index = pass
        .resources
        .iter()
        .position(|access| {
            access.name == resource_name && access.access == RenderGraphResourceAccessKind::Read
        })
        .expect("previous HZB history read");
    let access_id = graph.access_id_at(pass.id, access_index).unwrap();
    let key = graph
        .versioned_access_key(access_id)
        .expect("previous HZB history access key");
    assert_eq!(
        key.range,
        crate::render_graph::RenderGraphResourceAccessRange::Texture(
            crate::render_graph::RenderGraphTextureSubresourceRange {
                base_mip_level: 0,
                mip_level_count: Some(hzb_plan.mip_count),
                base_array_layer: 0,
                array_layer_count: Some(1),
                aspect: crate::render_graph::RenderGraphTextureAspect::All,
            },
        )
    );
    assert_eq!(
        key.intent,
        crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
            crate::render_graph::RenderGraphShaderStages::COMPUTE,
        )
    );
    assert!(graph.external_access_packet().access(access_id).is_some());
}

#[test]
fn compile_preserves_exact_external_uber_gi_history_texture_lease() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .expect("default forward-plus pipeline compiles");
    let graph = compiled.graph();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("uber pass");
    assert!(!pass.culled);
    let resource_name = PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI;
    let lifetime = graph
        .resource_lifetime_by_name(resource_name)
        .expect("previous GI history lifetime");
    let desc = lifetime
        .external_texture_desc
        .as_ref()
        .expect("previous GI history descriptor");
    assert_eq!(desc.format, crate::rhi::TextureFormat::Rgba16Float);
    assert_eq!(desc.mip_levels, 1);
    assert!(desc.usage.contains(crate::rhi::TextureUsage::SAMPLED));

    let access_index = pass
        .resources
        .iter()
        .position(|access| {
            access.name == resource_name && access.access == RenderGraphResourceAccessKind::Read
        })
        .expect("previous GI history read");
    let access_id = graph.access_id_at(pass.id, access_index).unwrap();
    let key = graph
        .versioned_access_key(access_id)
        .expect("previous GI history access key");
    assert_eq!(
        key.intent,
        crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
            crate::render_graph::RenderGraphShaderStages::FRAGMENT,
        )
    );
    assert!(graph.external_access_packet().access(access_id).is_some());
}
