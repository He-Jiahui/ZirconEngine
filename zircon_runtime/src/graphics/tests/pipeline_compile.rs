use crate::core::framework::render::{
    FallbackSkyboxKind, PostProcessGraphResourceNames, PreviewEnvironmentExtract, ProjectionMode,
    RenderCameraTarget, RenderDynamicResolutionSettings, RenderFrameExtract, RenderPhase,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::graphics::tests::plugin_render_feature_fixtures::{
    hybrid_gi_render_feature_descriptor, virtual_geometry_render_feature_descriptor,
};
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
};
use crate::rhi::TextureFormat;

use crate::{
    BuiltinRenderFeature, FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};

const ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS: &[(
    BuiltinRenderFeature,
    &str,
    RenderFeatureCapabilityRequirement,
)] = &[(
    BuiltinRenderFeature::SparseTexture,
    "sparse_texture",
    RenderFeatureCapabilityRequirement::SparseTexture,
)];

const ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS: &[(BuiltinRenderFeature, &str, &str)] = &[
    (BuiltinRenderFeature::Particle, "particle", "particles"),
    (BuiltinRenderFeature::Terrain, "terrain", "terrain"),
    (BuiltinRenderFeature::Tree, "tree", "tree"),
    (BuiltinRenderFeature::Projector, "projector", "projector"),
    (BuiltinRenderFeature::Halo, "halo", "halo"),
    (BuiltinRenderFeature::LensFlare, "lens_flare", "lens_flare"),
    (BuiltinRenderFeature::Trail, "trail", "trail"),
    (BuiltinRenderFeature::Billboard, "billboard", "billboard"),
    (BuiltinRenderFeature::Tilemap, "tilemap", "tilemap"),
    (
        BuiltinRenderFeature::TextShaping,
        "text_shaping",
        "text_shaping",
    ),
    (BuiltinRenderFeature::Skybox, "skybox", "skybox"),
    (BuiltinRenderFeature::Cubemap, "cubemap", "cubemap"),
    (
        BuiltinRenderFeature::Texture2dArray,
        "texture_2d_array",
        "texture_2d_array",
    ),
    (BuiltinRenderFeature::NormalMap, "normal_map", "normal_map"),
    (BuiltinRenderFeature::Mipmap, "mipmap", "mipmap"),
    (
        BuiltinRenderFeature::ColorSpace,
        "color_space",
        "color_space",
    ),
];

#[test]
fn default_forward_plus_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert!(pipeline.phase_mapping.contains(&RenderPhase::AlphaMask3d));

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Opaque3d,
            RenderPassStage::AlphaMask3d,
            RenderPassStage::Transparent3d,
            RenderPassStage::PostProcess,
            RenderPassStage::Ui,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "preview-sky",
            "depth-prepass",
            "shadow-map",
            "clustered-light-culling",
            "opaque-mesh",
            "alpha-mask-mesh",
            "transparent-mesh",
            "depth-of-field-prepare",
            "post-process",
            "bloom-extract",
            "color-grade",
            "fxaa",
            "runtime-ui",
            "overlay-gizmo",
        ]
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "depth-of-field-prepare",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "depth-of-field-prepare",
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert_eq!(
        pass_resource_access(
            &compiled,
            "depth-of-field-prepare",
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Write,
        )
        .attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
            "ui".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(compiled.history_bindings, Vec::<FrameHistoryBinding>::new());
}

#[test]
fn default_deferred_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_deferred();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert!(pipeline.phase_mapping.contains(&RenderPhase::AlphaMask3d));

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::Deferred,
            RenderPassStage::AlphaMask3d,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Transparent3d,
            RenderPassStage::PostProcess,
            RenderPassStage::Ui,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "preview-sky",
            "depth-prepass",
            "shadow-map",
            "gbuffer-mesh",
            "clustered-light-culling",
            "deferred-lighting",
            "transparent-mesh",
            "depth-of-field-prepare",
            "post-process",
            "bloom-extract",
            "color-grade",
            "fxaa",
            "runtime-ui",
            "overlay-gizmo",
        ]
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "gbuffer-mesh",
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "gbuffer-mesh",
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        RenderGraphResourceAccessKind::Write,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::FINAL_COLOR,
        RenderGraphResourceAccessKind::Read,
    );
    assert!(
        compiled
            .graph
            .passes()
            .iter()
            .flat_map(|pass| pass.resources.iter())
            .any(|resource| resource.name == PostProcessGraphResourceNames::GBUFFER_MATERIAL),
        "default deferred graph should declare its backed material G-buffer"
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
            "ui".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(compiled.history_bindings, Vec::<FrameHistoryBinding>::new());
}

#[test]
fn deferred_material_gbuffer_shaders_encode_and_decode_material_channels() {
    let geometry_shader =
        include_str!("../scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl");
    let lighting_shader =
        include_str!("../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl");
    for (name, shader) in [
        ("deferred_geometry.wgsl", geometry_shader),
        ("deferred_lighting.wgsl", lighting_shader),
    ] {
        let module = naga::front::wgsl::parse_str(shader)
            .unwrap_or_else(|error| panic!("{name}: {}", error.emit_to_string(shader)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }

    assert!(
        geometry_shader.contains("@location(1) material: vec4<f32>"),
        "deferred geometry should emit a second material G-buffer target"
    );
    assert!(
        geometry_shader.contains("material_properties.data0.x")
            && geometry_shader.contains("material_properties.data0.y"),
        "deferred geometry should encode metallic and roughness from material uniform channels"
    );
    assert!(
        lighting_shader.contains("var gbuffer_material_tex: texture_2d<f32>")
            && lighting_shader.contains("textureLoad(gbuffer_material_tex"),
        "deferred lighting should read the material G-buffer"
    );
    assert!(
        lighting_shader.contains("let roughness =") && lighting_shader.contains("let metallic ="),
        "deferred lighting should decode material G-buffer channels"
    );
}

#[test]
fn dynamic_resolution_scales_internal_graph_resources_without_resizing_viewport_output() {
    let mut extract = test_extract();
    extract.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    extract.apply_viewport_size(UVec2::new(320, 240));

    assert_eq!(extract.view.effective_view_size(), UVec2::new(320, 240));
    assert_eq!(extract.view.effective_render_size(), UVec2::new(160, 120));

    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    let scene_color =
        graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::SCENE_COLOR);
    assert!(matches!(
        &scene_color.desc,
        RenderGraphResourceDesc::Texture(desc)
            if scene_color.kind == RenderGraphResourceKind::TransientTexture
                && desc.width == 160
                && desc.height == 120
                && desc.format == TextureFormat::Rgba8UnormSrgb
    ));

    let scene_depth =
        graph_resource_lifetime(&compiled, PostProcessGraphResourceNames::SCENE_DEPTH);
    assert!(matches!(
        &scene_depth.desc,
        RenderGraphResourceDesc::Texture(desc)
            if scene_depth.kind == RenderGraphResourceKind::TransientTexture
                && desc.width == 160
                && desc.height == 120
                && desc.format == TextureFormat::Depth32Float
    ));

    let viewport_output = graph_resource_lifetime(&compiled, "viewport-output");
    assert_eq!(viewport_output.kind, RenderGraphResourceKind::External);
    assert_eq!(viewport_output.desc, RenderGraphResourceDesc::External);
}

#[test]
fn default_core2d_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_core2d();
    let compiled = pipeline.compile(&orthographic_extract()).unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::Opaque2d,
            RenderPassStage::AlphaMask2d,
            RenderPassStage::Transparent2d,
            RenderPassStage::PostProcess,
            RenderPassStage::Ui,
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| (pass.name.as_str(), pass.executor_id.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("opaque-sprite", Some("sprite.opaque")),
            ("alpha-mask-sprite", Some("sprite.alpha-mask")),
            ("transparent-sprite", Some("sprite.transparent")),
            (
                "depth-of-field-prepare",
                Some("post.depth-of-field-prepare"),
            ),
            ("post-process", Some("post.stack")),
            ("runtime-ui", Some("ui.screen-space")),
            ("overlay-gizmo", Some("overlay.gizmo")),
        ]
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "post_process".to_string(),
            "sprites".to_string(),
            "ui".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(compiled.history_bindings, Vec::<FrameHistoryBinding>::new());
}

#[test]
fn default_pipeline_assets_do_not_embed_pluginized_advanced_builtin_features() {
    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        for feature in [
            BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
            BuiltinRenderFeature::ReflectionProbes,
            BuiltinRenderFeature::BakedLighting,
        ] {
            assert!(
                !pipeline
                    .renderer
                    .features
                    .iter()
                    .any(|asset| asset.is_builtin(feature)),
                "{} should receive {:?} from rendering plugin descriptors",
                pipeline.name,
                feature
            );
        }
        for feature in [
            BuiltinRenderFeature::PostProcess,
            BuiltinRenderFeature::AntiAlias,
            BuiltinRenderFeature::Ui,
        ] {
            assert!(
                pipeline
                    .renderer
                    .features
                    .iter()
                    .any(|asset| asset.is_builtin(feature)),
                "{} should keep {:?} in the product render graph",
                pipeline.name,
                feature
            );
        }
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::VirtualGeometry)),
            "{} should receive virtual geometry from plugin descriptors",
            pipeline.name
        );
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::GlobalIllumination)),
            "{} should receive hybrid GI from plugin descriptors",
            pipeline.name
        );
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::Particle)),
            "{} should receive particles from plugin descriptors",
            pipeline.name
        );
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::NeuralCompute)),
            "{} should receive neural compute work from plugin descriptors",
            pipeline.name
        );
        for (feature, _, _) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
            assert!(
                !pipeline
                    .renderer
                    .features
                    .iter()
                    .any(|asset| asset.is_builtin(*feature)),
                "{} should receive {:?} from later render feature plans",
                pipeline.name,
                feature
            );
        }
    }
}

#[test]
fn rendering_plugin_default_features_restore_legacy_forward_plus_pass_order() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "preview-sky",
            "depth-prepass",
            "shadow-map",
            "ssao-evaluate",
            "clustered-light-culling",
            "opaque-mesh",
            "alpha-mask-mesh",
            "transparent-mesh",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "depth-of-field-prepare",
            "post-process",
            "color-grade",
            "fxaa",
            "runtime-ui",
            "overlay-gizmo",
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion
        )]
    );
}

#[test]
fn rendering_plugin_default_features_restore_legacy_deferred_pass_order() {
    let pipeline = RenderPipelineAsset::default_deferred()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "preview-sky",
            "depth-prepass",
            "shadow-map",
            "gbuffer-mesh",
            "ssao-evaluate",
            "clustered-light-culling",
            "deferred-lighting",
            "transparent-mesh",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "depth-of-field-prepare",
            "post-process",
            "color-grade",
            "fxaa",
            "runtime-ui",
            "overlay-gizmo",
        ]
    );
}

#[test]
fn history_resolve_compiles_only_with_explicit_feature_opt_in() {
    let pipeline = RenderPipelineAsset::default_forward_plus();

    let default_compiled = pipeline.compile(&test_extract()).unwrap();
    let default_pass_names = default_compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    assert!(!default_pass_names.contains(&"history-resolve"));
    assert!(!default_compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::SceneColor
        )));

    let enabled_compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::HistoryResolve),
        )
        .unwrap();
    let enabled_pass_names = enabled_compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    assert!(enabled_pass_names.contains(&"history-resolve"));
    assert!(enabled_compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::SceneColor
        )));

    let history_pass = enabled_compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "history-resolve")
        .expect("history resolve pass should be compiled after opt in");
    assert!(
        history_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR
                && resource.kind == RenderGraphResourceKind::External
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "history resolve must read the previous history slot"
    );
    assert!(
        history_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR
                && resource.kind == RenderGraphResourceKind::External
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "history resolve must write a separate output slot"
    );
    assert!(
        !history_pass
            .resources
            .iter()
            .any(|resource| resource.name == "history-scene-color"),
        "legacy single history resource name must not survive in the SRP graph"
    );
}

#[test]
fn pipeline_compile_assigns_attachment_ops_from_resource_write_order() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    let preview_sky_depth = pass_resource_access(
        &compiled,
        "preview-sky",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_depth.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "preview sky should clear scene depth before depth prepass writes geometry"
    );

    let preview_sky_scene_color = pass_resource_access(
        &compiled,
        "preview-sky",
        PostProcessGraphResourceNames::SCENE_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "preview sky should clear scene color before drawing the background"
    );

    let prepass_depth = pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        prepass_depth.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "depth prepass should load depth after preview sky initialized the target"
    );

    let prepass_normal = pass_resource_access(
        &compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        prepass_normal.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "depth prepass should clear the graph-owned normal target before writing normals"
    );

    let opaque_scene_color = pass_resource_access(
        &compiled,
        "opaque-mesh",
        "scene-color",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        opaque_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "opaque scene color should load the preview sky background"
    );

    let transparent_scene_color = pass_resource_access(
        &compiled,
        "transparent-mesh",
        "scene-color",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        transparent_scene_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "later scene-color producers must load existing opaque output"
    );

    let runtime_ui_output = pass_resource_access(
        &compiled,
        "runtime-ui",
        "viewport-output",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        runtime_ui_output.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "external outputs default to load/store because ownership is imported"
    );

    let overlay_output = pass_resource_access(
        &compiled,
        "overlay-gizmo",
        "viewport-output",
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        overlay_output.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "overlay must load the UI/postprocess output before adding debug draws"
    );

    let deferred_compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();
    let preview_sky_final_color = pass_resource_access(
        &deferred_compiled,
        "preview-sky",
        PostProcessGraphResourceNames::FINAL_COLOR,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        preview_sky_final_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred preview sky should explicitly clear the imported final-color background target"
    );

    let deferred_prepass_depth = pass_resource_access(
        &deferred_compiled,
        "depth-prepass",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        deferred_prepass_depth.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        }),
        "deferred depth prepass should load depth after preview sky initialized the target"
    );
}

#[test]
fn particle_plugin_render_feature_adds_transparent_pass_to_default_pipeline() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_render_feature_descriptor()]);
    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"particle-render"));
    assert!(compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
    let particle_feature = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.feature_name() == "particle")
        .expect("particle plugin feature should remain in compiled pipeline");
    assert!(
        particle_feature.builtin_feature().is_none(),
        "particle plugin feature should not reintroduce built-in feature identity"
    );
}

#[test]
fn compile_options_can_disable_particle_plugin_feature_by_name() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_render_feature_descriptor()]);
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled("particle"),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"particle-render"));
    assert!(!compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
}

#[test]
fn forward_plus_pipeline_compilation_is_deterministic() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let extract = test_extract();

    let first = pipeline.compile(&extract).unwrap();
    let second = pipeline.compile(&extract).unwrap();

    assert_eq!(first, second);
}

#[test]
fn builtin_pipeline_lookup_exposes_deferred_pipeline_handle() {
    let builtin =
        RenderPipelineAsset::builtin(crate::core::framework::render::RenderPipelineHandle::new(2))
            .expect("handle 2 should map to the built-in deferred pipeline");

    assert_eq!(builtin, RenderPipelineAsset::default_deferred());
}

#[test]
fn history_binding_accessors_construct_expected_bindings() {
    assert_eq!(
        FrameHistoryBinding::read(FrameHistorySlot::AmbientOcclusion),
        FrameHistoryBinding {
            slot: FrameHistorySlot::AmbientOcclusion,
            access: FrameHistoryAccess::Read,
        }
    );
    assert_eq!(
        FrameHistoryBinding::write(FrameHistorySlot::SceneColor),
        FrameHistoryBinding {
            slot: FrameHistorySlot::SceneColor,
            access: FrameHistoryAccess::Write,
        }
    );
}

#[test]
fn compile_options_can_disable_clustered_history_and_rendering_plugin_features() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
        .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
        .with_plugin_feature_disabled("screen_space_ambient_occlusion");

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"ssao-evaluate"));
    assert!(!pass_names.contains(&"clustered-light-culling"));
    assert!(!pass_names.contains(&"history-resolve"));
    assert!(!compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion
        )));
}

#[test]
fn flagship_feature_descriptors_declare_backend_capability_requirements() {
    assert_eq!(
        BuiltinRenderFeature::VirtualGeometry
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
    );
    assert_eq!(
        BuiltinRenderFeature::GlobalIllumination
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::HybridGlobalIllumination]
    );
    assert_eq!(
        BuiltinRenderFeature::RayTracing
            .descriptor()
            .capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::AccelerationStructures,
            RenderFeatureCapabilityRequirement::RayTracingPipeline,
        ]
    );
    assert_eq!(
        BuiltinRenderFeature::NeuralCompute
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::NeuralCompute]
    );
    assert_eq!(
        BuiltinRenderFeature::SparseTexture
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::SparseTexture]
    );
}

#[test]
fn advanced_followup_feature_slots_reserve_extract_sections_without_runtime_passes() {
    for (feature, extract_section, requirement) in
        ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS
    {
        let descriptor = feature.descriptor();
        assert_eq!(descriptor.name, *extract_section);
        assert_eq!(
            descriptor.required_extract_sections,
            vec![extract_section.to_string()]
        );
        assert_eq!(descriptor.capability_requirements, vec![*requirement]);
        assert!(descriptor.history_bindings.is_empty());
        assert!(
            descriptor.stage_passes.is_empty(),
            "{feature:?} should stay descriptor-only until its dedicated render plan registers passes"
        );
    }

    for (feature, descriptor_name, extract_section) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        let descriptor = feature.descriptor();
        assert_eq!(descriptor.name, *descriptor_name);
        assert_eq!(
            descriptor.required_extract_sections,
            vec![extract_section.to_string()]
        );
        assert!(descriptor.capability_requirements.is_empty());
        assert!(descriptor.history_bindings.is_empty());
        assert!(
            descriptor.stage_passes.is_empty(),
            "{feature:?} should stay descriptor-only until its dedicated render plan registers passes"
        );
    }
}

#[test]
fn neural_compute_builtin_slot_compiles_only_with_explicit_feature_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::NeuralCompute,
        ));

    let default_compiled = pipeline.compile(&test_extract()).unwrap();
    assert!(
        !default_compiled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::NeuralCompute),
        "neural compute should not declare backend requirements until the slot is opted in"
    );

    let enabled_compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::NeuralCompute),
        )
        .unwrap();

    assert!(enabled_compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::NeuralCompute));
    assert!(
        !enabled_compiled
            .graph
            .passes()
            .iter()
            .any(|pass| pass.name.contains("neural")),
        "the runtime slot should only declare the capability; plugin descriptors own executable neural passes"
    );
}

#[test]
fn advanced_followup_builtin_slots_compile_only_with_explicit_feature_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    for (feature, _, _) in ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::builtin(*feature));
    }
    for (feature, _, _) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::builtin(*feature));
    }

    let default_compiled = pipeline.compile(&test_extract()).unwrap();
    for (feature, extract_section, requirement) in
        ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS
    {
        assert!(
            !default_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should not compile until explicitly opted in"
        );
        assert!(
            !default_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should not request extract data until explicitly opted in"
        );
        assert!(
            !default_compiled
                .capability_requirements
                .contains(requirement),
            "{feature:?} should not require backend capability until explicitly opted in"
        );
    }
    for (feature, _, extract_section) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        assert!(
            !default_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should not compile until explicitly opted in"
        );
        assert!(
            !default_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should not request extract data until explicitly opted in"
        );
    }

    let mut options = RenderPipelineCompileOptions::default();
    for (feature, _, requirement) in ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        options = options
            .with_feature_enabled(*feature)
            .with_capability_enabled(*requirement);
    }
    for (feature, _, _) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        options = options.with_feature_enabled(*feature);
    }
    let enabled_compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    for (feature, extract_section, requirement) in
        ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS
    {
        assert!(
            enabled_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should compile when explicitly opted in"
        );
        assert!(
            enabled_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should reserve its neutral extract section"
        );
        assert!(
            enabled_compiled
                .capability_requirements
                .contains(requirement),
            "{feature:?} should declare its backend capability requirement"
        );
    }
    for (feature, _, extract_section) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        assert!(
            enabled_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should compile when explicitly opted in"
        );
        assert!(
            enabled_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should reserve its neutral extract section"
        );
    }
    assert_eq!(
        enabled_compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        default_compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        "descriptor-only slots must not add executable graph passes"
    );
    let added_requirements = enabled_compiled
        .capability_requirements
        .iter()
        .filter(|requirement| {
            !default_compiled
                .capability_requirements
                .contains(requirement)
        })
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        added_requirements,
        vec![RenderFeatureCapabilityRequirement::SparseTexture],
        "only sparse texture should add a backend capability requirement in this follow-up slot set"
    );
}

#[test]
fn sparse_texture_builtin_slot_requires_feature_and_capability_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::SparseTexture,
        ));

    let feature_only = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::SparseTexture),
        )
        .unwrap();
    assert!(
        !feature_only
            .enabled_features
            .iter()
            .any(|feature| feature.is_builtin(BuiltinRenderFeature::SparseTexture)),
        "feature opt-in without the sparse texture capability should keep the slot out of the graph"
    );
    assert!(!feature_only
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::SparseTexture));

    let capability_enabled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::SparseTexture)
                .with_capability_enabled(RenderFeatureCapabilityRequirement::SparseTexture),
        )
        .unwrap();
    assert!(capability_enabled
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::SparseTexture)));
    assert!(capability_enabled
        .required_extract_sections
        .contains(&"sparse_texture".to_string()));
    assert!(capability_enabled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::SparseTexture));
    assert!(
        !capability_enabled
            .graph
            .passes()
            .iter()
            .any(|pass| pass.name.contains("sparse")),
        "the runtime slot should only reserve extract/capability; executable sparse passes are follow-up work"
    );
}

#[test]
fn compiled_pipeline_collects_enabled_plugin_feature_capability_requirements() {
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        virtual_geometry_render_feature_descriptor(),
        hybrid_gi_render_feature_descriptor(),
    ]);
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::AntiAlias)
        .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
        .with_capability_enabled(RenderFeatureCapabilityRequirement::HybridGlobalIllumination);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled.capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::VirtualGeometry,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ]
    );
}

#[test]
fn compile_options_fallback_async_compute_passes_to_graphics_queue() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default().with_async_compute(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .filter(|pass| pass.queue == QueueLane::AsyncCompute)
            .count(),
        0
    );
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "ssao-evaluate"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "clustered-light-culling"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    let ssao_output = pass_resource_access(
        &compiled,
        "ssao-evaluate",
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(ssao_output.kind, RenderGraphResourceKind::External);
    assert_eq!(
        ssao_output.attachment_ops, None,
        "compute storage writes must not inherit render attachment load/store ops"
    );
    assert_eq!(compiled.graph.stats().queue_fallback_pass_count, 2);
}

#[test]
fn feature_pass_descriptors_drive_executor_ids_and_resource_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    let depth_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "depth-prepass")
        .expect("default forward pipeline should include depth prepass");
    assert_eq!(
        depth_pass.executor_id.as_deref(),
        Some("mesh.depth-prepass")
    );
    assert!(
        depth_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::GBUFFER_NORMAL
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "depth prepass should declare the normal target it writes"
    );

    let preview_sky_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "preview-sky")
        .expect("default forward pipeline should include preview sky pass");
    assert_eq!(
        preview_sky_pass.executor_id.as_deref(),
        Some("sky.preview-scene-color")
    );
    assert!(
        preview_sky_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_COLOR
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "preview sky should initialize scene color through the render graph"
    );
    assert!(
        preview_sky_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "preview sky should initialize scene depth through the render graph"
    );

    let opaque_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque-mesh")
        .expect("default forward pipeline should include opaque mesh pass");
    assert_eq!(opaque_pass.executor_id.as_deref(), Some("mesh.opaque"));
    let overlay_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "overlay-gizmo")
        .expect("default forward pipeline should include overlay pass");
    assert!(
        overlay_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "overlay executor should declare its depth read instead of borrowing the target privately"
    );

    let lifetimes = compiled.graph.resource_lifetimes();
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-depth" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-color" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors() {
    let extract = extract_with_camera(ViewportCameraSnapshot {
        target: RenderCameraTarget::Headless {
            size: UVec2::new(1280, 720),
        },
        hdr: true,
        msaa_samples: 4,
        ..ViewportCameraSnapshot::default()
    });
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    let scene_color = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-color")
        .expect("scene-color should be a graph resource");
    assert!(matches!(
        &scene_color.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 1280
                && desc.height == 720
                && desc.format == TextureFormat::Rgba16Float
                && desc.sample_count == 4
    ));

    let scene_depth = compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-depth")
        .expect("scene-depth should be a graph resource");
    assert!(matches!(
        &scene_depth.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 1280
                && desc.height == 720
                && desc.format == TextureFormat::Depth32Float
                && desc.sample_count == 4
    ));
}

#[test]
fn gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        virtual_geometry_render_feature_descriptor(),
        hybrid_gi_render_feature_descriptor(),
    ]);
    let disabled = pipeline.compile(&test_extract()).unwrap();
    let disabled_pass_names = disabled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    assert!(!disabled_pass_names.contains(&"hybrid-gi-resolve"));
    assert!(!disabled_pass_names.contains(&"virtual-geometry-node-cluster-cull"));

    let options = RenderPipelineCompileOptions::default()
        .with_capability_enabled(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
        .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
    let enabled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let enabled_pass_names = enabled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    for pass_name in [
        "hybrid-gi-scene-prepare",
        "hybrid-gi-trace-schedule",
        "hybrid-gi-resolve",
        "hybrid-gi-history",
        "virtual-geometry-prepare",
        "virtual-geometry-node-cluster-cull",
        "virtual-geometry-page-feedback",
        "virtual-geometry-visbuffer",
        "virtual-geometry-debug-overlay",
    ] {
        assert!(
            enabled_pass_names.contains(&pass_name),
            "enabled graph should contain {pass_name}"
        );
    }
    assert!(enabled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination
        )));
}

#[test]
fn plugin_render_feature_asset_compiles_descriptor_without_builtin_feature_identity() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            plugin_virtual_geometry_descriptor(),
        ));

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        pass_names.contains(&"plugin-virtual-geometry-prepare"),
        "plugin descriptor pass should be compiled into the render graph"
    );
    let plugin_feature = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.feature_name() == "plugin.virtual_geometry")
        .expect("compiled pipeline should retain the plugin feature name");
    assert!(
        plugin_feature.builtin_feature().is_none(),
        "plugin renderer feature should not masquerade as a built-in feature"
    );
    assert!(compiled
        .required_extract_sections
        .contains(&"plugin_virtual_geometry".to_string()));
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::VirtualGeometry));
}

#[test]
fn plugin_render_feature_asset_respects_capability_opt_in_gate() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            plugin_virtual_geometry_descriptor(),
        ));

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !pass_names.contains(&"plugin-virtual-geometry-prepare"),
        "advanced plugin descriptor passes should not compile until their capability is enabled"
    );
    assert!(
        !compiled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::VirtualGeometry),
        "disabled plugin descriptors should not add runtime capability requirements"
    );
}

#[test]
fn plugin_neural_compute_feature_respects_capability_opt_in_gate() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            plugin_neural_compute_descriptor(),
        ));

    let disabled = pipeline.compile(&test_extract()).unwrap();
    assert!(
        !disabled
            .graph
            .passes()
            .iter()
            .any(|pass| pass.name == "plugin-neural-inference"),
        "neural compute plugin passes should not compile until the capability is enabled"
    );
    assert!(
        !disabled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::NeuralCompute),
        "disabled neural plugin descriptors should not add runtime capability requirements"
    );

    let enabled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::NeuralCompute),
        )
        .unwrap();

    let neural_pass = enabled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "plugin-neural-inference")
        .expect("enabled neural compute plugin pass should compile into the render graph");
    assert_eq!(neural_pass.queue, QueueLane::AsyncCompute);
    assert!(enabled
        .required_extract_sections
        .contains(&"plugin_neural_compute".to_string()));
    assert!(enabled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::NeuralCompute));
}

#[test]
fn plugin_render_feature_descriptors_replace_advanced_builtin_slots() {
    let pipeline = legacy_advanced_builtin_pipeline().with_plugin_render_features([
        replacement_virtual_geometry_descriptor(),
        replacement_hybrid_gi_descriptor(),
    ]);

    assert!(!pipeline
        .renderer
        .features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::VirtualGeometry)));
    assert!(!pipeline
        .renderer
        .features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::GlobalIllumination)));
    assert!(pipeline.renderer.features.iter().any(|feature| {
        feature.feature_name() == "virtual_geometry" && feature.builtin_feature().is_none()
    }));
    assert!(pipeline.renderer.features.iter().any(|feature| {
        feature.feature_name() == "hybrid_gi" && feature.builtin_feature().is_none()
    }));

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                .with_capability_enabled(
                    RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                ),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"plugin-virtual-geometry-replacement"));
    assert!(pass_names.contains(&"plugin-hybrid-gi-replacement"));
    assert!(
        !pass_names.contains(&"virtual-geometry-prepare"),
        "built-in virtual geometry pass should be removed when plugin descriptor replaces the capability"
    );
    assert!(
        !pass_names.contains(&"hybrid-gi-resolve"),
        "built-in hybrid GI pass should be removed when plugin descriptor replaces the capability"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_plugin_render_feature_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
            "plugin.duplicate_feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "plugin-duplicate-feature-a",
                QueueLane::Graphics,
            )
            .with_executor_id("plugin.duplicate.a")
            .with_side_effects()],
        )));
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
            "plugin.duplicate_feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "plugin-duplicate-feature-b",
                QueueLane::Graphics,
            )
            .with_executor_id("plugin.duplicate.b")
            .with_side_effects()],
        )));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate feature `plugin.duplicate_feature`"),
        "unexpected error: {error}"
    );
}

fn plugin_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plugin.virtual_geometry",
        vec!["plugin_virtual_geometry".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            "plugin-virtual-geometry-prepare",
            QueueLane::Graphics,
        )
        .with_executor_id("plugin.virtual-geometry.prepare")
        .write_buffer("plugin-virtual-geometry-page-requests")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

fn plugin_neural_compute_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plugin.neural_compute",
        vec!["plugin_neural_compute".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "plugin-neural-inference",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("plugin.neural.inference")
        .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
        .write_buffer("plugin-neural-output")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute)
}

fn replacement_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "virtual_geometry",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            "plugin-virtual-geometry-replacement",
            QueueLane::Graphics,
        )
        .with_executor_id("plugin.virtual-geometry.replacement")
        .write_buffer("plugin-virtual-geometry-replacement")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

fn replacement_hybrid_gi_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hybrid_gi",
        Vec::new(),
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "plugin-hybrid-gi-replacement",
            QueueLane::Graphics,
        )
        .with_executor_id("plugin.hybrid-gi.replacement")
        .write_texture("plugin-hybrid-gi-lighting")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

fn particle_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent3d,
            "particle-render",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.transparent")
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn default_rendering_feature_descriptors() -> Vec<RenderFeatureDescriptor> {
    vec![
        rendering_ssao_descriptor(),
        rendering_reflection_probes_descriptor(),
        rendering_baked_lighting_descriptor(),
        rendering_post_process_descriptor(),
    ]
}

fn rendering_ssao_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("ao.ssao-evaluate")
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .write_storage_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)],
    )
}

fn rendering_reflection_probes_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "reflection_probes",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_baked_lighting_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "baked_lighting",
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.baked-composite")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_post_process_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "post_process",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "depth-of-field-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("post.depth-of-field-prepare")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_external_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
                RenderGraphAttachmentOps::clear_store(),
            )
            .write_external_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "post-process",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .read_external(PostProcessGraphResourceNames::BLOOM)
            .write_external(PostProcessGraphResourceNames::FINAL_COMPOSITED)
            .write_external(PostProcessGraphResourceNames::FINAL_COLOR)
            .write_external(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
        ],
    )
}

fn legacy_advanced_builtin_pipeline() -> RenderPipelineAsset {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::VirtualGeometry,
        ));
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::GlobalIllumination,
        ));
    pipeline
}

#[test]
fn pipeline_compile_rejects_duplicate_stage_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline.renderer.stages.push(RenderPassStage::Opaque3d);

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_feature_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate feature"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_core_pipeline_mismatch_from_extract() {
    let error = RenderPipelineAsset::default_forward_plus()
        .compile(&orthographic_extract())
        .unwrap_err();

    assert!(
        error.contains("core pipeline mismatch")
            && error.contains("Core3d")
            && error.contains("Core2d"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_declared_renderer_stage_missing_product_phase_mapping() {
    let mut pipeline = RenderPipelineAsset::default_core2d();
    pipeline
        .phase_mapping
        .retain(|phase| *phase != RenderPhase::Transparent2d);

    let error = pipeline.compile(&orthographic_extract()).unwrap_err();

    assert!(
        error.contains("missing product phase") && error.contains("Transparent2d"),
        "unexpected error: {error}"
    );
}

#[test]
fn disabling_post_process_keeps_overlay_extract_requirements_for_debug_gizmos() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled("post_process"),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !pass_names.contains(&"post-process"),
        "post-process pass should be removed when the feature is disabled"
    );
    assert!(
        !pass_names.contains(&"depth-of-field-prepare"),
        "DoF scratch preparation should be removed with the post-process feature"
    );
    assert!(
        pass_names.contains(&"overlay-gizmo"),
        "overlay stage should remain available for debug and gizmo rendering"
    );
    assert!(
        compiled
            .required_extract_sections
            .contains(&"debug".to_string()),
        "overlay feature should keep requiring debug extract data"
    );
}

#[test]
fn renderer_feature_asset_quality_gate_controls_compiled_passes() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom")
        .quality_gate = Some(BuiltinRenderFeature::RayTracing);

    let without_gate = pipeline.compile(&test_extract()).unwrap();
    assert!(!without_gate
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));

    let with_gate = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::RayTracing),
        )
        .unwrap();
    assert!(with_gate
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));
}

#[test]
fn pipeline_compile_validates_quality_gated_descriptor_overrides_even_when_gate_is_closed() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_quality_gate(BuiltinRenderFeature::VirtualGeometry)
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-gated-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque,
                "bad-gated-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("bad-gated-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn renderer_feature_asset_local_config_and_capabilities_survive_compile() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let color_grading = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::ColorGrading))
        .expect("default pipeline should include color grading");
    *color_grading = color_grading
        .clone()
        .with_local_config("variant", "cinematic")
        .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline);

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let compiled_color_grading = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::ColorGrading))
        .expect("color grading should remain enabled");

    assert_eq!(
        compiled_color_grading.local_config.get("variant"),
        Some(&"cinematic".to_string())
    );
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::RayTracingPipeline));
}

#[test]
fn renderer_feature_asset_descriptor_override_changes_compiled_graph() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "custom-bloom",
            vec!["custom_post".to_string()],
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "custom-bloom-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .read_texture("scene-color")
            .write_external("viewport-output")],
        ));

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"bloom-extract"));
    assert!(pass_names.contains(&"custom-bloom-pass"));
    assert!(compiled
        .required_extract_sections
        .contains(&"custom_post".to_string()));
    assert!(compiled.graph.resource_lifetimes().iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn pipeline_compile_rejects_descriptor_passes_for_undeclared_stages() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-stage-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque,
                "custom-gbuffer-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .with_side_effects()],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("custom-gbuffer-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_descriptor_pass_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "duplicate-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "color-grade",
                QueueLane::Graphics,
            )
            .with_executor_id("post.color-grade")
            .with_side_effects()],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate render graph pass name `color-grade`"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_conflicting_descriptor_resource_kinds() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_buffer("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("conflicting resource kind"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_explicit_external_resource_name_conflicts() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-external-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-external-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_external("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("explicit external resource"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("feature descriptor name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_names_after_descriptor_name_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("pass name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("executor id must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_resource_names_after_executor_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_storage_write_mode_on_read_access() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    let mut invalid_pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "bad-storage-mode-pass",
        QueueLane::Graphics,
    )
    .with_executor_id("post.stack");
    invalid_pass
        .resources
        .push(RenderFeatureResourceDescriptor {
            name: "scene-color".to_string(),
            kind: RenderFeatureResourceKind::Texture,
            access: RenderFeatureResourceAccess::Read,
            attachment_ops: None,
            write_mode: RenderFeatureResourceWriteMode::Storage,
        });
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-storage-mode-feature",
            Vec::new(),
            Vec::new(),
            vec![invalid_pass],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("cannot declare storage write mode for a read access"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_extract_section_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-extract-section-feature",
            vec!["post".to_string(), " ".to_string()],
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "post-stack-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .read_texture("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("extract section name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_history_bindings_in_one_descriptor() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "duplicate-history-feature",
            Vec::new(),
            vec![
                FrameHistoryBinding::read(FrameHistorySlot::SceneColor),
                FrameHistoryBinding::write(FrameHistorySlot::SceneColor),
            ],
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "history-using-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .read_texture("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate history binding for slot `SceneColor`"),
        "unexpected error: {error}"
    );
}

fn test_extract() -> RenderFrameExtract {
    extract_with_camera(ViewportCameraSnapshot::default())
}

fn extract_with_camera(camera: ViewportCameraSnapshot) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn pass_resource_access<'a>(
    compiled: &'a crate::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> &'a crate::render_graph::RenderGraphPassResourceAccess {
    compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .and_then(|pass| {
            pass.resources
                .iter()
                .find(|resource| resource.name == resource_name && resource.access == access)
        })
        .unwrap_or_else(|| panic!("pass `{pass_name}` should {access:?} `{resource_name}`"))
}

fn graph_resource_lifetime<'a>(
    compiled: &'a crate::CompiledRenderPipeline,
    resource_name: &str,
) -> &'a crate::render_graph::RenderGraphResourceLifetime {
    compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap_or_else(|| panic!("compiled graph should contain resource `{resource_name}`"))
}

fn orthographic_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(2),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    projection_mode: ProjectionMode::Orthographic,
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}
