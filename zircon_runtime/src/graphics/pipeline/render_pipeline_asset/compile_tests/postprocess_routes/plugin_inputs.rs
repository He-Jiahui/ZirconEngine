use super::*;

#[test]
fn compile_orders_plugin_scene_velocity_load_after_temporal_velocity_producer() {
    let mut extract = test_extract();
    extract.particles.emitters = vec![42];
    extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
        entity: 42,
        stable_sprite_key: 7,
        position: Vec3::new(0.0, 0.0, -2.0),
        size: 0.5,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        sort_order: 0,
        color: Vec4::ONE,
        intensity: 1.0,
        depth_test: true,
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        material: None,
        texture: None,
    }];
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
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 1.0,
                temporal_blend_factor: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::smaa(),
        true,
        false,
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_velocity_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::Temporal)
                .with_post_process_stack(stack),
        )
        .unwrap();

    assert_pass_writes(
        &compiled,
        "velocity-object",
        PostProcessGraphResourceNames::SCENE_VELOCITY,
    );
    assert_pass_writes(
        &compiled,
        "particle-velocity",
        PostProcessGraphResourceNames::SCENE_VELOCITY,
    );
    let velocity_object_index = graph_pass_index(&compiled, "velocity-object");
    let particle_velocity_index = graph_pass_index(&compiled, "particle-velocity");
    assert!(
        velocity_object_index < particle_velocity_index,
        "temporal velocity producer must execute before plugin particle velocity load/store writer"
    );
    let particle_velocity = graph_pass(&compiled, "particle-velocity");
    assert!(particle_velocity.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
            && resource.access == RenderGraphResourceAccessKind::Write
            && resource.attachment_ops == Some(RenderGraphAttachmentOps::load_store())
    }));
}

#[test]
fn compile_filters_plugin_scene_velocity_pass_without_post_process_stack() {
    let extract = particle_extract();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_velocity_descriptor()])
        .compile_with_options(&extract, &RenderPipelineCompileOptions::default())
        .unwrap();

    assert!(!graph_has_pass(&compiled, "particle-velocity"));
    assert!(graph_has_pass(&compiled, "particle-render"));
}

#[test]
fn compile_filters_plugin_scene_velocity_pass_when_stack_does_not_use_scene_velocity() {
    let extract = particle_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings::default(),
        &extract.post_process.color_grading,
        &RenderPostProcessEffectStackSettings::default(),
        false,
        false,
        &AntiAliasSettings::off(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_velocity_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    assert!(!graph_has_pass(&compiled, "particle-velocity"));
    assert!(graph_has_pass(&compiled, "particle-render"));
}

#[test]
fn compile_filters_hybrid_gi_lighting_from_uber_without_stack_input() {
    let extract = test_extract();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_lighting_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_post_process_stack(PostProcessStackDescriptor::default()),
        )
        .unwrap();

    assert_pass_does_not_read(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
    );
}

#[test]
fn compile_routes_hybrid_gi_lighting_into_uber_when_stack_requests_current_input() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::default().with_hybrid_gi_lighting_input();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_lighting_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();

    assert_pass_writes(
        &compiled,
        "hybrid-gi-resolve",
        PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
    );
    assert_pass_reads(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
    );
    assert!(graph_pass_index(&compiled, "hybrid-gi-resolve") < graph_pass_index(&compiled, "uber"));
}

#[test]
fn compile_keeps_hybrid_gi_lighting_single_sample_when_graph_msaa_is_enabled() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::default().with_hybrid_gi_lighting_input();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_lighting_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_graph_msaa_sample_count(4)
                .with_post_process_stack(stack),
        )
        .unwrap();

    assert_pass_writes(
        &compiled,
        "hybrid-gi-resolve",
        PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
    );
    assert_pass_reads(
        &compiled,
        "uber",
        PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
    );
    assert_eq!(
        texture_lifetime(&compiled, PostProcessGraphResourceNames::SCENE_COLOR).sample_count,
        4
    );
    assert_eq!(
        texture_lifetime(&compiled, PostProcessGraphResourceNames::HYBRID_GI_LIGHTING).sample_count,
        1
    );
}

fn particle_velocity_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-velocity",
                QueueLane::Graphics,
            )
            .with_executor_id("particle.velocity")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-render",
                QueueLane::Graphics,
            )
            .with_executor_id("particle.transparent")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}

fn hybrid_gi_lighting_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hybrid_gi",
        vec!["view".to_string(), "lighting".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .write_texture(PostProcessGraphResourceNames::HYBRID_GI_LIGHTING),
        ],
    )
}

fn particle_extract() -> RenderFrameExtract {
    let mut extract = test_extract();
    extract.particles.emitters = vec![42];
    extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
        entity: 42,
        stable_sprite_key: 7,
        position: Vec3::new(0.0, 0.0, -2.0),
        size: 0.5,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        sort_order: 0,
        color: Vec4::ONE,
        intensity: 1.0,
        depth_test: true,
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        material: None,
        texture: None,
    }];
    extract
}
