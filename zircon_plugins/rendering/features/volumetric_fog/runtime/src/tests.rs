use super::*;
use zircon_runtime::core::framework::render::{
    AdvancedLightingExtract, EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract,
    RenderFrameExtract, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ShaderQualityTier, ViewportCameraSnapshot, VolumetricFogSettings,
};
use zircon_runtime::core::math::Vec4;
use zircon_runtime::graphics::{
    CompiledRenderPipeline, RenderFeatureResourceAccess, RenderFeatureResourceKind,
    RenderPipelineAsset, RenderPipelineCompileOptions,
};
use zircon_runtime::render_graph::{
    RenderGraphComputeDispatchExtent, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphResourceDesc, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange,
};
use zircon_runtime::rhi::{TextureDimension, TextureFormat, TextureUsage};

#[test]
fn volumetric_fog_feature_declares_three_compute_passes_and_resources() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);

    let feature = &report.extensions.render_features()[0];
    assert_eq!(feature.name, FEATURE_NAME);
    assert_eq!(
        feature
            .stage_passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        vec![MEDIA_INJECT_PASS, LIGHT_SCATTER_PASS, INTEGRATE_PASS]
    );
    for pass in &feature.stage_passes {
        assert_eq!(pass.stage, RenderPassStage::Lighting);
        assert_eq!(pass.queue, QueueLane::AsyncCompute);
        assert!(pass.compute_workload.is_some());
    }

    let media = &feature.stage_passes[0];
    assert!(resource_matches(
        media,
        PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
        RenderFeatureResourceKind::Texture,
        RenderFeatureResourceAccess::Write,
    ));
    let scatter = &feature.stage_passes[1];
    for resource in [
        PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
        PostProcessGraphResourceNames::SCENE_LIGHT_DATA,
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        PostProcessGraphResourceNames::SHADOW_ATLAS,
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
    ] {
        assert!(scatter.resources.iter().any(|entry| entry.name == resource));
    }
    assert!(
        !scatter
            .resources
            .iter()
            .any(|entry| entry.name == PostProcessGraphResourceNames::LIGHT_LIST),
        "volumetric scattering must not reinterpret the clustered-lighting tile buffer as GpuLightData",
    );
    assert!(resource_matches(
        scatter,
        PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        RenderFeatureResourceKind::Texture,
        RenderFeatureResourceAccess::Write,
    ));
    let history = scatter
        .resources
        .iter()
        .find(|resource| {
            resource.name == PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING
        })
        .expect("volumetric history resource");
    assert!(history.usage.persistent);
    assert_eq!(history.schema, None);
    assert_eq!(
        history.access_metadata,
        Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full()),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        ))
    );
    assert!(scatter.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING
            && resource.usage.persistent
    }));
    let integrate = &feature.stage_passes[2];
    assert!(resource_matches(
        integrate,
        PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        RenderFeatureResourceKind::Texture,
        RenderFeatureResourceAccess::Read,
    ));
    assert!(resource_matches(
        integrate,
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
        RenderFeatureResourceKind::Texture,
        RenderFeatureResourceAccess::Write,
    ));
}

#[test]
fn volumetric_fog_passes_follow_light_grid_and_precede_scene_shading() {
    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline
            .with_plugin_render_features([render_feature_descriptor()])
            .compile(&test_extract())
            .unwrap();
        let passes = pass_names(&compiled);

        assert_before(&passes, "shadow-atlas", MEDIA_INJECT_PASS);
        assert_before(&passes, "light-grid-build", MEDIA_INJECT_PASS);
        assert_before(&passes, MEDIA_INJECT_PASS, LIGHT_SCATTER_PASS);
        assert_before(&passes, LIGHT_SCATTER_PASS, INTEGRATE_PASS);
        assert_before(&passes, INTEGRATE_PASS, "preview-sky");
        pass_reads_integrated(&compiled, "preview-sky");
        if passes.contains(&"deferred-lighting") {
            assert_before(&passes, INTEGRATE_PASS, "deferred-lighting");
            pass_reads_integrated(&compiled, "deferred-lighting");
        } else {
            assert_before(&passes, INTEGRATE_PASS, "opaque-mesh");
            for pass in ["opaque-mesh", "alpha-mask-mesh", "transparent-mesh"] {
                pass_reads_integrated(&compiled, pass);
            }
        }
    }
}

#[test]
fn render_advanced_extract_empty_keeps_graph_baseline_when_feature_is_disabled() {
    let extract = test_extract();
    assert!(extract.lighting.advanced_lighting.is_empty());

    let baseline = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();
    let disabled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled(FEATURE_NAME),
        )
        .unwrap();

    assert_eq!(
        baseline.graph().dump().to_text(),
        disabled.graph().dump().to_text()
    );
}

#[test]
fn volumetric_fog_graph_uses_rgba16f_d3_resources_and_two_physical_slots() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile(&volumetric_extract())
        .unwrap();

    for resource in [
        PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
        PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    ] {
        let desc = texture_desc(&compiled, resource);
        assert_eq!(desc.format, TextureFormat::Rgba16Float);
        assert_eq!(desc.dimension, TextureDimension::D3);
        assert_eq!([desc.width, desc.height, desc.depth], [160, 90, 64]);
        assert_eq!(desc.sample_count, 1);
    }

    let allocation = compiled.graph().transient_allocation_plan();
    assert_eq!(
        allocation.slot_for(PostProcessGraphResourceNames::VOLUMETRIC_MEDIA),
        allocation.slot_for(PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED),
        "media lifetime must alias integrated output so three logical products use two physical textures",
    );
    assert_eq!(
        allocation.slot_for(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING),
        None,
        "cross-frame volumetric history source must retain its own physical backing until extraction",
    );

    let media_dispatch = compute_dispatch(&compiled, MEDIA_INJECT_PASS);
    let scatter_dispatch = compute_dispatch(&compiled, LIGHT_SCATTER_PASS);
    let integrate_dispatch = compute_dispatch(&compiled, INTEGRATE_PASS);
    assert_eq!(
        media_dispatch,
        &RenderGraphComputeDispatchExtent::FroxelGrid
    );
    assert_eq!(
        scatter_dispatch,
        &RenderGraphComputeDispatchExtent::FroxelGrid
    );
    assert_eq!(
        integrate_dispatch,
        &RenderGraphComputeDispatchExtent::FroxelGridXy
    );
}

#[test]
fn volumetric_fog_graph_dimensions_follow_shader_quality_tier() {
    for (quality, expected_depth) in [
        (ShaderQualityTier::Low, 48),
        (ShaderQualityTier::Medium, 64),
        (ShaderQualityTier::High, 96),
        (ShaderQualityTier::Ultra, 96),
    ] {
        let compiled = RenderPipelineAsset::default_forward_plus()
            .with_plugin_render_features([render_feature_descriptor()])
            .compile_with_options(
                &volumetric_extract(),
                &RenderPipelineCompileOptions::default().with_shader_quality(quality),
            )
            .unwrap();
        let desc = texture_desc(
            &compiled,
            PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        );

        assert_eq!(
            [desc.width, desc.height, desc.depth],
            [160, 90, expected_depth]
        );
    }
}

#[test]
fn volumetric_fog_graph_preserves_exact_external_history_texture_lease() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()])
        .compile_with_options(
            &volumetric_extract(),
            &RenderPipelineCompileOptions::default().with_shader_quality(ShaderQualityTier::High),
        )
        .expect("high-quality volumetric graph compiles");
    let graph = compiled.graph();
    let resource_name = PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING;
    let desc = graph
        .resource_lifetime_by_name(resource_name)
        .and_then(|lifetime| lifetime.external_texture_desc.as_ref())
        .expect("volumetric external history descriptor");
    assert_eq!((desc.width, desc.height, desc.depth), (160, 90, 96));
    assert_eq!(desc.dimension, TextureDimension::D3);
    assert_eq!(desc.format, TextureFormat::Rgba16Float);
    assert_eq!(desc.mip_levels, 1);
    assert_eq!(desc.sample_count, 1);
    assert_eq!(desc.usage, TextureUsage::SAMPLED);

    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == LIGHT_SCATTER_PASS)
        .expect("volumetric light-scatter pass");
    let access_index = pass
        .resources
        .iter()
        .position(|access| {
            access.name == resource_name && access.access == RenderGraphResourceAccessKind::Read
        })
        .expect("volumetric history read");
    let access_id = graph
        .access_id_at(pass.id, access_index)
        .expect("volumetric history access ID");
    let key = graph
        .versioned_access_key(access_id)
        .expect("volumetric history access key");
    assert_eq!(
        key.range,
        RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
            aspect: zircon_runtime::render_graph::RenderGraphTextureAspect::All,
        })
    );
    assert_eq!(
        key.intent,
        RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE)
    );
    assert!(graph.external_access_packet().access(access_id).is_some());
}

fn resource_matches(
    pass: &zircon_runtime::graphics::RenderFeaturePassDescriptor,
    name: &str,
    kind: RenderFeatureResourceKind,
    access: RenderFeatureResourceAccess,
) -> bool {
    pass.resources
        .iter()
        .any(|resource| resource.name == name && resource.kind == kind && resource.access == access)
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: EnvironmentExtract::default(),
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

fn volumetric_extract() -> RenderFrameExtract {
    let mut extract = test_extract();
    extract.lighting.advanced_lighting = AdvancedLightingExtract {
        volumetric: Some(VolumetricFogSettings::default()),
        ..AdvancedLightingExtract::default()
    };
    extract
}

fn pass_names(compiled: &CompiledRenderPipeline) -> Vec<&str> {
    compiled
        .graph()
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect()
}

fn assert_before(passes: &[&str], before: &str, after: &str) {
    let before_index = passes
        .iter()
        .position(|pass| *pass == before)
        .unwrap_or_else(|| panic!("missing pass `{before}` in {passes:?}"));
    let after_index = passes
        .iter()
        .position(|pass| *pass == after)
        .unwrap_or_else(|| panic!("missing pass `{after}` in {passes:?}"));
    assert!(
        before_index < after_index,
        "expected `{before}` before `{after}` in {passes:?}"
    );
}

fn texture_desc<'a>(
    compiled: &'a CompiledRenderPipeline,
    resource_name: &str,
) -> &'a zircon_runtime::rhi::TextureDesc {
    let lifetime = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap_or_else(|| panic!("missing resource `{resource_name}`"));
    match &lifetime.desc {
        RenderGraphResourceDesc::Texture(desc) => desc,
        other => panic!("expected texture `{resource_name}`, got {other:?}"),
    }
}

fn compute_dispatch<'a>(
    compiled: &'a CompiledRenderPipeline,
    pass_name: &str,
) -> &'a RenderGraphComputeDispatchExtent {
    &compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing pass `{pass_name}`"))
        .compute_workload
        .as_ref()
        .expect("volumetric pass compute workload")
        .dispatch_extent
}

fn pass_reads_integrated(compiled: &CompiledRenderPipeline, pass_name: &str) {
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("missing pass `{pass_name}`"));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
}
