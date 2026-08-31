use super::*;

fn with_builtin_ssao(mut pipeline: RenderPipelineAsset) -> RenderPipelineAsset {
    pipeline
        .renderer
        .features
        .push(crate::graphics::pipeline::RendererFeatureAsset::builtin(
            BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
        ));
    pipeline
}

#[test]
fn default_pipelines_keep_unqualified_ssao_out_of_the_product_graph() {
    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline.compile(&test_extract()).unwrap();

        assert!(!compiled.runtime_feature_flags().ssao_enabled);
        assert!(
            compiled
                .graph()
                .passes()
                .iter()
                .all(|pass| pass.name != "ssao-evaluate")
        );
    }
}

#[test]
fn forward_plus_rejects_ssao_without_a_normal_producer() {
    let pipeline = with_builtin_ssao(RenderPipelineAsset::default_forward_plus());
    let Err(error) = pipeline.compile(&test_extract()) else {
        panic!("Forward+ must reject SSAO without a qualified normal producer");
    };

    assert!(error.contains("screen_space_ambient_occlusion input qualification failed"));
    assert!(error.contains(PostProcessGraphResourceNames::GBUFFER_NORMAL));
    assert!(error.contains("enabled writer before `ssao-evaluate`"));
}

#[test]
fn deferred_ssao_runs_after_qualified_inputs_without_terminal_scene_darkening() {
    let pipeline = with_builtin_ssao(RenderPipelineAsset::default_deferred());
    let compiled = pipeline
        .compile(&test_extract())
        .expect("deferred SSAO has scene depth, normal, and HZB producers");
    let graph = compiled.graph();
    let profile = compiled
        .ambient_occlusion_profile()
        .expect("qualified SSAO must persist its compiled AO profile");

    assert_ne!(profile.pipeline_generation(), 0);
    assert_eq!(profile.input_qualification().inputs().len(), 3);
    assert!(
        profile
            .input_qualification()
            .inputs()
            .iter()
            .all(|input| input.producer_generation() == profile.pipeline_generation())
    );

    assert!(
        graph_pass_index(&compiled, "depth-prepass") < graph_pass_index(&compiled, "ssao-evaluate")
    );
    assert!(
        graph_pass_index(&compiled, "gbuffer-mesh") < graph_pass_index(&compiled, "ssao-evaluate")
    );
    assert!(
        graph_pass_index(&compiled, "hzb-build") < graph_pass_index(&compiled, "ssao-evaluate")
    );
    assert!(
        graph_pass_index(&compiled, "ssao-evaluate")
            < graph_pass_index(&compiled, "deferred-lighting")
    );

    let uber = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("deferred post-process uber pass");
    assert!(
        uber.resources
            .iter()
            .all(|resource| resource.name != PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
    );

    let normal = graph
        .resource_lifetime_by_name(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .expect("qualified GBuffer normal lifetime")
        .physical_texture_desc
        .as_ref()
        .expect("qualified GBuffer normal descriptor");
    assert_eq!(normal.format, crate::rhi::TextureFormat::Rgba8Unorm);
    assert_eq!(normal.sample_count, 1);
}

#[test]
fn deferred_half_resolution_ssao_compiles_two_half_stages_and_a_full_upsample() {
    let mut extract = test_extract();
    extract.post_process.ambient_occlusion.half_resolution = true;
    let compiled = with_builtin_ssao(RenderPipelineAsset::default_deferred())
        .compile(&extract)
        .expect("qualified half-resolution SSAO should compile its bilateral chain");
    let profile = compiled
        .ambient_occlusion_profile()
        .expect("half-resolution AO profile");
    let graph = compiled.graph();

    assert_eq!(profile.resolution_divisor(), 2);
    assert!(
        graph_pass_index(&compiled, "ssao-evaluate")
            < graph_pass_index(&compiled, "ssao-spatial-denoise")
    );
    assert!(
        graph_pass_index(&compiled, "ssao-spatial-denoise")
            < graph_pass_index(&compiled, "ssao-bilateral-upsample")
    );
    assert!(
        graph_pass_index(&compiled, "ssao-bilateral-upsample")
            < graph_pass_index(&compiled, "deferred-lighting")
    );

    for resource_name in [
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
    ] {
        let texture = graph
            .resource_lifetime_by_name(resource_name)
            .and_then(|lifetime| lifetime.physical_texture_desc.as_ref())
            .expect("half-resolution AO transient texture");
        let expected = profile.work_extent();
        assert_eq!((texture.width, texture.height), (expected.x, expected.y));
    }
    assert_eq!(
        compiled
            .ambient_occlusion_outputs()
            .expect("compiled AO output receipt")
            .indirect_diffuse_ao()
            .producer_pass_name(),
        "ssao-bilateral-upsample"
    );
}

#[test]
fn deferred_ssao_rejects_temporal_authoring_until_history_is_motion_qualified() {
    let mut extract = test_extract();
    extract.post_process.ambient_occlusion.temporal = true;

    let error = with_builtin_ssao(RenderPipelineAsset::default_deferred())
        .compile(&extract)
        .expect_err("unqualified AO history must fail at profile compilation");

    assert!(error.contains("temporal accumulation"));
    assert!(error.contains("motion/depth/normal history qualification"));
}

#[test]
fn deferred_ssao_rejects_multisampled_depth_and_normal_inputs() {
    let pipeline = with_builtin_ssao(RenderPipelineAsset::default_deferred());
    let error = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_graph_msaa_sample_count(4),
        )
        .expect_err("the current SSAO bindings accept only single-sampled depth and normal");

    assert!(error.contains("screen_space_ambient_occlusion input qualification failed"));
    assert!(error.contains(PostProcessGraphResourceNames::SCENE_DEPTH));
    assert!(error.contains("sample count 4"));
    assert!(error.contains("requires 1"));
}

#[test]
fn deferred_ssao_normal_contract_is_linear_and_independent_of_camera_hdr() {
    for hdr in [false, true] {
        let mut extract = test_extract();
        extract.view.camera.hdr = hdr;
        let compiled = with_builtin_ssao(RenderPipelineAsset::default_deferred())
            .compile(&extract)
            .expect("deferred SSAO has a qualified normal contract");
        let normal = compiled
            .graph()
            .resource_lifetime_by_name(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .expect("GBuffer normal lifetime")
            .physical_texture_desc
            .as_ref()
            .expect("GBuffer normal descriptor");

        assert_eq!(normal.format, crate::rhi::TextureFormat::Rgba8Unorm);
    }
}

#[test]
fn deferred_ssao_rejects_a_viewport_origin_the_kernel_cannot_address() {
    let mut extract = test_extract();
    let display_extent = crate::core::math::UVec2::new(320, 180);
    extract.apply_viewport_size(display_extent);
    extract.view.apply_view_family_pipeline(
        crate::core::framework::render::RenderViewFamilyPipeline::resolve_for_viewport(
            display_extent,
            crate::core::framework::render::RenderViewportRect::new(
                crate::core::math::UVec2::new(32, 16),
                crate::core::math::UVec2::new(240, 128),
            ),
            crate::core::framework::render::RenderResolutionPolicy::default(),
            crate::core::framework::render::RenderUpscalerKind::Spatial,
        ),
    );

    let error = with_builtin_ssao(RenderPipelineAsset::default_deferred())
        .compile(&extract)
        .expect_err("SSAO must not silently sample from allocation origin for a sub-rect view");

    assert!(error.contains("screen_space_ambient_occlusion input qualification failed"));
    assert!(error.contains("viewport origin"));
    assert!(error.contains("32x16"));
}
