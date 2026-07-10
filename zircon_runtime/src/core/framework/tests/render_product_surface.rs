use super::*;

#[test]
fn render_product_camera_projection_and_core_pipeline_are_independent() {
    let core3d = CameraRenderDescriptor::from_camera_payload(
        None,
        ViewportCameraSnapshot {
            core_pipeline: CorePipelineKind::Core3d,
            projection_mode: super::super::render::ProjectionMode::Orthographic,
            ..ViewportCameraSnapshot::default()
        },
    );
    let core2d = CameraRenderDescriptor::from_camera_payload(
        None,
        ViewportCameraSnapshot {
            core_pipeline: CorePipelineKind::Core2d,
            projection_mode: super::super::render::ProjectionMode::Orthographic,
            ..ViewportCameraSnapshot::default()
        },
    );

    assert_eq!(core3d.camera.core_pipeline, CorePipelineKind::Core3d);
    assert_eq!(core2d.camera.core_pipeline, CorePipelineKind::Core2d);
    assert_eq!(core3d.camera.projection_mode, core2d.camera.projection_mode);
    assert_eq!(
        core3d.camera.projection_mode,
        super::super::render::ProjectionMode::Orthographic
    );
}

#[test]
fn render_product_orthographic_projection_keeps_orthographic_matrix_in_core3d() {
    let camera = ViewportCameraSnapshot {
        core_pipeline: CorePipelineKind::Core3d,
        projection_mode: super::super::render::ProjectionMode::Orthographic,
        ..ViewportCameraSnapshot::default()
    };
    let projection =
        super::super::render::ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(1280, 720))
            .clip_from_world_unjittered;

    assert_eq!(camera.core_pipeline_kind(), CorePipelineKind::Core3d);
    assert_eq!(projection.w_axis.w, 1.0);
}

#[test]
fn render_product_post_process_graph_elides_disabled_effects() {
    let stack = PostProcessStackDescriptor::default();

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.skipped_node_count(), 2);
    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::ExposureResolve,
            PostProcessEffectKind::OutputTransfer,
        ]
    );
    assert_eq!(
        graph.output_transfer_node.as_deref(),
        Some("output-transfer")
    );
}

#[test]
fn render_product_post_process_stack_elides_history_until_history_is_available() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &Default::default(),
        true,
        false,
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(graph.node_count(), 2);
    assert!(!graph
        .nodes
        .iter()
        .any(|node| node.kind == PostProcessEffectKind::TaaResolve));
}

#[test]
fn render_product_post_process_stack_can_drop_history_from_validated_graph() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &Default::default(),
        true,
        true,
    );

    let stack = stack.without_history_resources();
    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(graph.node_count(), 2);
    assert!(!graph
        .nodes
        .iter()
        .any(|node| node.kind == PostProcessEffectKind::TaaResolve));
}

#[test]
fn render_product_post_process_graph_rejects_missing_scene_color() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_DEPTH.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::OutputTransfer)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::MissingRequiredInput {
            node: "output-transfer".to_string(),
            resource: PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
        })
    );
}

#[test]
fn render_product_post_process_graph_rejects_missing_taa_history_dependency() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
            PostProcessGraphResourceNames::SCENE_VELOCITY.to_string(),
        ],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::TaaResolve)
                .with_required_inputs([
                    PostProcessGraphResourceNames::SCENE_COLOR,
                    PostProcessGraphResourceNames::SCENE_DEPTH,
                    PostProcessGraphResourceNames::SCENE_VELOCITY,
                    PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
                ])
                .with_produced_outputs([
                    PostProcessGraphResourceNames::TAA_OUTPUT,
                    PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
                ]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::MissingRequiredInput {
            node: "taa-resolve".to_string(),
            resource: PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string(),
        })
    );
}

#[test]
fn render_product_post_process_graph_rejects_duplicate_output_resource() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string(),
        ],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM]),
            PostProcessEffectSettings::new(PostProcessEffectKind::ColorLutBake)
                .with_required_inputs([PostProcessGraphResourceNames::EXPOSURE_CURRENT])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::DuplicateOutputResource {
            node: "color-lut-bake".to_string(),
            resource: PostProcessGraphResourceNames::BLOOM.to_string(),
        })
    );
}

#[test]
fn render_product_post_process_graph_rejects_cycles() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string(),
        ],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM])
                .with_after([PostProcessEffectKind::ColorLutBake]),
            PostProcessEffectSettings::new(PostProcessEffectKind::ColorLutBake)
                .with_required_inputs([PostProcessGraphResourceNames::EXPOSURE_CURRENT])
                .with_produced_outputs([PostProcessGraphResourceNames::COLOR_LUT])
                .with_after([PostProcessEffectKind::Bloom]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::CycleDetected)
    );
}

#[test]
fn render_product_post_process_graph_rejects_missing_effect_dependency() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::OutputTransfer)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR])
                .with_after([PostProcessEffectKind::Bloom]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::MissingDependency {
            node: "output-transfer".to_string(),
            dependency: PostProcessEffectKind::Bloom,
        })
    );
}

#[test]
fn render_product_post_process_graph_allows_color_grading_without_bloom() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &super::super::render::RenderColorGradingSettings {
            exposure: 1.05,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        },
        false,
        false,
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::ExposureResolve,
            PostProcessEffectKind::ColorLutBake,
            PostProcessEffectKind::OutputTransfer,
        ]
    );
}

#[test]
fn render_product_post_process_graph_bakes_tonemap_and_user_lut_without_color_grading() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings::default(),
        &super::super::render::RenderColorGradingSettings::default(),
        &RenderPostProcessEffectStackSettings {
            tonemap: super::super::render::RenderTonemapSettings {
                operator: super::super::render::RenderTonemapOperator::Aces,
                ..Default::default()
            },
            color_lookup: super::super::render::RenderColorLookupSettings {
                intensity: 0.5,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &super::super::render::AntiAliasSettings::off(),
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();
    let color_lut_bake = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::ColorLutBake)
        .expect("tonemap and user LUT should enable the color LUT bake node");

    assert!(color_lut_bake
        .required_inputs
        .contains(&PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string()));
    assert!(color_lut_bake
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::COLOR_LUT.to_string()));
}

#[test]
fn render_product_post_process_effect_stack_runs_before_output_transfer_when_authored() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings {
            threshold: 1.0,
            intensity: 0.5,
            radius: 0.25,
        },
        &super::super::render::RenderColorGradingSettings {
            exposure: 1.05,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        },
        &RenderPostProcessEffectStackSettings {
            vignette: super::super::render::RenderVignetteSettings {
                intensity: 0.35,
                ..Default::default()
            },
            grain: super::super::render::RenderFilmGrainSettings {
                intensity: 0.2,
                ..Default::default()
            },
            chromatic_aberration: super::super::render::RenderChromaticAberrationSettings {
                intensity: 0.1,
                ..Default::default()
            },
            fog: super::super::render::RenderFogSettings {
                density: 0.05,
                color: Vec3::new(0.5, 0.6, 0.7),
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &super::super::render::AntiAliasSettings::off(),
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();
    let effect_stack = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::Uber)
        .expect("authored effect-stack settings should enable the graph node");
    let output_transfer = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("postprocess graph should still end in final composite");

    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::ExposureResolve,
            PostProcessEffectKind::Bloom,
            PostProcessEffectKind::ColorLutBake,
            PostProcessEffectKind::SceneComposite,
            PostProcessEffectKind::Uber,
            PostProcessEffectKind::OutputTransfer,
        ]
    );
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::BLOOM.to_string()));
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::COLOR_LUT.to_string()));
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert_eq!(
        effect_stack.produced_outputs,
        vec![
            PostProcessGraphResourceNames::EFFECT_STACKED.to_string(),
            PostProcessGraphResourceNames::TONEMAPPED.to_string(),
        ]
    );
    assert_eq!(
        output_transfer.required_inputs,
        vec![PostProcessGraphResourceNames::EFFECT_STACKED.to_string()]
    );
    assert_eq!(output_transfer.after, vec![PostProcessEffectKind::Uber]);
}

#[test]
fn render_product_post_process_extended_effect_stack_settings_enable_product_node() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings::default(),
        &super::super::render::RenderColorGradingSettings::default(),
        &RenderPostProcessEffectStackSettings {
            tonemap: super::super::render::RenderTonemapSettings {
                operator: super::super::render::RenderTonemapOperator::Filmic,
                ..Default::default()
            },
            dither: super::super::render::RenderDitherSettings {
                intensity: 0.2,
                ..Default::default()
            },
            screen_space_reflection: super::super::render::RenderScreenSpaceReflectionSettings {
                intensity: 0.4,
                max_steps: 24,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &super::super::render::AntiAliasSettings::off(),
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();
    let effect_stack = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::Uber)
        .expect("SSR settings should enable the effect-stack node");

    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::ExposureResolve,
            PostProcessEffectKind::ColorLutBake,
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
            PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion,
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse,
            PostProcessEffectKind::ScreenSpaceReflectionResolve,
            PostProcessEffectKind::SceneComposite,
            PostProcessEffectKind::Uber,
            PostProcessEffectKind::OutputTransfer,
        ]
    );
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::COLOR_LUT.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
}

#[test]
fn render_camera_contracts_cover_viewports_and_bevy_layer_intersection() {
    let viewport = RenderViewportRect::new(UVec2::new(600, 400), UVec2::new(100, 100))
        .clamped_to_size(UVec2::new(640, 480));
    assert_eq!(viewport.physical_position, UVec2::new(600, 400));
    assert_eq!(viewport.physical_size, UVec2::new(40, 80));

    let layers = RenderLayerSet::from_layers([0, 3, 70]);
    assert!(layers.contains(0));
    assert!(layers.contains(3));
    assert!(layers.contains(70));
    assert!(layers.intersects(&RenderLayerSet::layer(70)));
    assert!(!layers.intersects(&RenderLayerSet::layer(4)));
    assert!(!RenderLayerSet::none().intersects(&RenderLayerSet::none()));
    assert_eq!(
        RenderLayerSet::from_scene_schema_v1_mask(0b1010).to_scene_schema_v1_mask_lossy(),
        0b1010
    );

    let mut camera = CameraRenderDescriptor::from_camera_payload(
        None,
        ViewportCameraSnapshot {
            hdr: true,
            msaa_samples: 4,
            ..ViewportCameraSnapshot::default()
        },
    );
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(100, 0),
        UVec2::new(320, 160),
    ));
    camera.culling_mask = RenderLayerSet::from_layers([3]);
    camera.apply_target_size(UVec2::new(1920, 1080));

    assert_eq!(camera.camera.aspect_ratio, 2.0);
    assert!(camera.camera.hdr);
    assert_eq!(camera.camera.msaa_samples, 4);
    assert!(camera.culling_mask.intersects_scene_schema_v1_mask(0b1000));
    assert!(!camera.culling_mask.intersects_scene_schema_v1_mask(0b0010));

    camera.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    assert_eq!(
        camera.effective_viewport_size(UVec2::new(1920, 1080)),
        UVec2::new(320, 160),
        "dynamic resolution must not change the camera viewport/present size"
    );
    assert_eq!(
        camera.effective_render_size(UVec2::new(1920, 1080)),
        UVec2::new(160, 80),
        "dynamic resolution should scale only the internal render extent"
    );

    camera.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.0);
    assert_eq!(
        camera.effective_render_size(UVec2::new(1920, 1080)),
        UVec2::new(32, 16),
        "render scale is clamped so graph resources never collapse to zero"
    );

    camera.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(f32::NAN);
    assert_eq!(
        camera.effective_render_size(UVec2::new(1920, 1080)),
        UVec2::new(320, 160),
        "non-finite render scale falls back to unscaled viewport size"
    );
}

#[test]
fn render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index() {
    let texture_a = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://textures/camera-a.png",
    ));

    let report = super::super::render::sort_render_cameras([
        RenderCameraOrderInput::from_descriptor(
            40,
            camera_order_input(2, RenderCameraTarget::PrimarySurface),
        ),
        RenderCameraOrderInput::from_descriptor(
            30,
            hdr_camera_order_input(2, RenderCameraTarget::Texture(texture_a)),
        ),
        RenderCameraOrderInput::from_descriptor(
            10,
            camera_order_input(
                -1,
                RenderCameraTarget::Headless {
                    size: UVec2::new(640, 480),
                },
            ),
        ),
        RenderCameraOrderInput::from_descriptor(
            20,
            hdr_camera_order_input(0, RenderCameraTarget::Texture(texture_a)),
        ),
        RenderCameraOrderInput::from_descriptor(
            50,
            camera_order_input(0, RenderCameraTarget::PrimarySurface),
        ),
    ]);

    assert!(!report.has_ambiguities());
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![10, 50, 20, 40, 30]
    );
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.sorted_camera_index_for_target)
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1]
    );
}

#[test]
fn render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras() {
    let report = super::super::render::sort_render_cameras([
        RenderCameraOrderInput::from_descriptor(30, inactive_camera_order_input(1)),
        RenderCameraOrderInput::from_descriptor(
            20,
            camera_order_input(1, RenderCameraTarget::PrimarySurface),
        ),
        RenderCameraOrderInput::from_descriptor(
            40,
            camera_order_input(
                1,
                RenderCameraTarget::Headless {
                    size: UVec2::new(320, 240),
                },
            ),
        ),
        RenderCameraOrderInput::from_descriptor(
            10,
            camera_order_input(1, RenderCameraTarget::PrimarySurface),
        ),
    ]);

    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![10, 20, 40]
    );
    assert_eq!(
        report.ambiguities,
        vec![RenderCameraOrderAmbiguity {
            order: 1,
            target: RenderCameraTargetOrderKey::PrimarySurface,
        }]
    );
}

fn camera_order_input(order: i32, target: RenderCameraTarget) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        render_order: order,
        target,
        ..CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default())
    }
}

fn hdr_camera_order_input(order: i32, target: RenderCameraTarget) -> CameraRenderDescriptor {
    let mut camera = camera_order_input(order, target);
    camera.camera.hdr = true;
    camera
}

fn inactive_camera_order_input(order: i32) -> CameraRenderDescriptor {
    let mut camera = camera_order_input(order, RenderCameraTarget::PrimarySurface);
    camera.camera.is_active = false;
    camera
}
