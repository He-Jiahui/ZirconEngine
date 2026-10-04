use super::*;
use zircon_runtime::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::math::Vec4;
use zircon_runtime::graphics::{
    CompiledRenderPipeline, RenderFeatureResourceAccess, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderPassExecutorId, RenderPipelineAsset,
    RenderPipelineCompileOptions,
};
use zircon_runtime::render_graph::{
    RenderGraphComputeDispatchExtent, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
};
use zircon_runtime::rhi::TextureFormat;

#[test]
fn contact_shadow_feature_registers_hzb_ray_march_pass() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);

    let feature = &report.extensions.render_features()[0];
    assert_eq!(feature.name, FEATURE_NAME);
    assert!(
        feature
            .required_extract_sections
            .contains(&"visibility".to_string())
    );

    let pass = &feature.stage_passes[0];
    assert_eq!(pass.stage, RenderPassStage::AmbientOcclusion);
    assert_eq!(pass.pass_name, PASS_NAME);
    assert_eq!(pass.queue, QueueLane::AsyncCompute);
    assert_eq!(pass.executor_id.as_str(), EXECUTOR_ID);
    assert!(pass.flags.has_side_effects);
    let workload = pass
        .compute_workload
        .as_ref()
        .expect("contact shadow pass should declare a compute workload");
    assert_eq!(workload.pipeline_label, CONTACT_SHADOW_PIPELINE_LABEL);
    assert_eq!(workload.workgroup_size, CONTACT_SHADOW_WORKGROUP_SIZE);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION.to_string(),
            local_size: [
                CONTACT_SHADOW_WORKGROUP_SIZE[0],
                CONTACT_SHADOW_WORKGROUP_SIZE[1],
            ],
        }
    );

    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::HZB_FURTHEST
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Read
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Write
            && resource.write_mode == RenderFeatureResourceWriteMode::Storage
    }));
}

#[test]
fn contact_shadow_graph_pass_is_absent_when_plugin_feature_is_disabled() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([render_feature_descriptor()]);
    let disabled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled(FEATURE_NAME),
        )
        .unwrap();
    let disabled_passes = pass_names(&disabled);

    assert!(!disabled_passes.contains(&PASS_NAME));

    let enabled = pipeline.compile(&test_extract()).unwrap();
    let enabled_passes = pass_names(&enabled);
    let hzb_index = enabled_passes
        .iter()
        .position(|name| *name == "hzb-build")
        .expect("default graph should build HZB before contact shadows");
    let contact_index = enabled_passes
        .iter()
        .position(|name| *name == PASS_NAME)
        .expect("enabled contact shadow feature should add its graph pass");

    assert!(hzb_index < contact_index);
    let pass = enabled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == PASS_NAME)
        .expect("contact shadow pass should compile");
    assert!(!pass.culled);
    assert!(
        !pass.flags.has_side_effects,
        "uber consumes contact shadow occlusion through the graph"
    );
    assert_eq!(pass.queue, QueueLane::AsyncCompute);
    assert_eq!(
        pass.compute_workload
            .as_ref()
            .map(|workload| &workload.dispatch_extent),
        Some(&RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION.to_string(),
            local_size: [
                CONTACT_SHADOW_WORKGROUP_SIZE[0],
                CONTACT_SHADOW_WORKGROUP_SIZE[1],
            ],
        })
    );
    pass_resource_access(
        &enabled,
        PASS_NAME,
        PostProcessGraphResourceNames::HZB_FURTHEST,
        RenderGraphResourceAccessKind::Read,
    );
    let contact_write = pass_resource_access(
        &enabled,
        PASS_NAME,
        PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        contact_write.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        texture_format(
            &enabled,
            PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
        ),
        TextureFormat::Rgba8Unorm
    );
    let contact_post_read = pass_resource_access(
        &enabled,
        "uber",
        PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
        RenderGraphResourceAccessKind::Read,
    );
    assert_eq!(
        contact_post_read.kind,
        RenderGraphResourceKind::TransientTexture,
        "post-process must keep contact shadow in the graph lifetime instead of relying on an undeclared optional lookup"
    );
}

#[test]
fn contact_shadow_executor_accepts_declared_pass_contract() {
    validate_context(&context_for_contract())
        .unwrap_or_else(|error| panic!("contact shadow contract failed: {error}"));
}

#[test]
fn contact_shadow_executor_requires_gpu_after_contract_validation() {
    let mut context = context_for_contract();
    let error = render_pass_executor_registration()
        .execute(&mut context)
        .unwrap_err();

    assert!(error.contains("requires renderer GPU context"), "{error}");
}

#[test]
fn contact_shadow_executor_rejects_resource_contract_drift() {
    let mut context = context_for_contract();
    context.resources.retain(|resource| {
        resource.name != PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
    });

    let error = validate_context(&context).unwrap_err();

    assert!(error.contains("resource contract mismatch"), "{error}");
    assert!(
        error.contains(PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION),
        "{error}"
    );
}

#[test]
fn contact_shadow_shader_declares_expected_compute_bindings() {
    assert!(
        CONTACT_SHADOW_SHADER_SOURCE
            .contains("@group(0) @binding(0) var depth_tex: texture_depth_2d")
    );
    assert!(
        CONTACT_SHADOW_SHADER_SOURCE
            .contains("@group(0) @binding(1) var normal_tex: texture_2d<f32>")
    );
    assert!(
        CONTACT_SHADOW_SHADER_SOURCE
            .contains("@group(0) @binding(2) var hzb_furthest_tex: texture_2d<f32>")
    );
    assert!(CONTACT_SHADOW_SHADER_SOURCE.contains(
        "@group(0) @binding(3) var contact_shadow_out: texture_storage_2d<rgba8unorm, write>"
    ));
    assert!(CONTACT_SHADOW_SHADER_SOURCE.contains("@compute @workgroup_size(8, 8, 1)"));
    assert!(CONTACT_SHADOW_SHADER_SOURCE.contains("textureStore(contact_shadow_out"));
}

#[test]
fn contact_shadow_pipeline_cache_is_device_epoch_qualified_and_fail_closed() {
    let source = include_str!("../lib.rs");
    let production_end = source
        .rfind("mod tests {")
        .expect("contact shadow production source must precede its tests");
    let production = &source[..production_end];
    let epoch_gate = production
        .find("let Some(device_epoch) = gpu.device_epoch()")
        .expect("executor must require a materialized device epoch");
    let resource_lookup = production
        .find("gpu.require_texture_view(")
        .expect("executor must resolve graph textures");
    let cache_lock = production
        .find(".lock()")
        .expect("executor must synchronize its persistent pipeline cache");
    let pipeline_create = production
        .find("let pipeline = ContactShadowPipeline::new(&native)")
        .expect("executor must rebuild the pipeline on cache miss");
    let cache_release = production
        .find("drop(pipeline_guard.take())")
        .expect("executor must release the old native cache before rebuilding");

    assert!(production.contains("pipeline: Mutex<Option<ContactShadowPipelineCache>>"));
    assert!(production.contains("device_epoch: RenderPassDeviceEpoch"));
    assert!(production.contains("cached.device_epoch == device_epoch"));
    assert!(production.contains("RenderPassGpuResourceFactory"));
    assert!(!production.contains("native.device"));
    assert!(production.contains(
        "contact shadow executor requires a materialized device epoch before pipeline recording"
    ));
    assert!(epoch_gate < resource_lookup);
    assert!(epoch_gate < cache_lock);
    assert!(cache_lock < cache_release);
    assert!(cache_release < pipeline_create);
    assert!(epoch_gate < pipeline_create);
    assert!(!production.contains("Mutex<Option<ContactShadowPipeline>>"));
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

fn pass_names(compiled: &CompiledRenderPipeline) -> Vec<&str> {
    compiled
        .graph()
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect()
}

fn pass_resource_access<'a>(
    compiled: &'a CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> &'a zircon_runtime::render_graph::RenderGraphPassResourceAccess {
    compiled
        .graph()
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

fn texture_format(compiled: &CompiledRenderPipeline, resource_name: &str) -> TextureFormat {
    let lifetime = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap_or_else(|| panic!("compiled graph should contain `{resource_name}`"));
    match &lifetime.desc {
        RenderGraphResourceDesc::Texture(desc) => desc.format,
        other => panic!("expected texture resource for `{resource_name}`, got {other:?}"),
    }
}

fn context_for_contract() -> RenderPassExecutionContext<'static> {
    let contract = &CONTACT_SHADOW_CONTRACT;
    RenderPassExecutionContext::with_declared_graph_metadata_and_resources(
        contract.pass_name,
        RenderPassExecutorId::new(contract.executor_id),
        contract.declared_queue,
        contract.declared_queue,
        contract.flags,
        contract
            .resources
            .iter()
            .map(|resource| RenderGraphPassResourceAccess {
                name: resource.name.to_string(),
                kind: match resource.kind {
                    ExpectedResourceKind::Exact(kind) => kind,
                    ExpectedResourceKind::AnyOf(kinds) => kinds[0],
                },
                access: resource.access,
                attachment_ops: None,
            })
            .collect(),
    )
}
