use super::*;

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
            .graph
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
}
