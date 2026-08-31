use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassDeviceEpoch,
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
    RenderPassGpuResourceFactory, RenderPassStage,
};
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    RenderingContactShadowRuntimeFeature, feature_manifest, plugin_feature_registration,
    runtime_plugin_feature,
};

pub const FEATURE_ID: &str = "rendering.contact_shadow";
pub const FEATURE_NAME: &str = "contact_shadow";
pub const PASS_NAME: &str = "contact-shadow";
pub const EXECUTOR_ID: &str = "lighting.contact-shadow";
pub const CONTACT_SHADOW_PIPELINE_LABEL: &str = "zircon-contact-shadow-ray-march";
pub const CONTACT_SHADOW_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const CONTACT_SHADOW_SHADER_SOURCE: &str = include_str!("contact_shadow.wgsl");

#[cfg(test)]
mod wgpu_product_tests;

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
            "lighting".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AmbientOcclusion,
                PASS_NAME,
                QueueLane::AsyncCompute,
            )
            .with_executor_id(EXECUTOR_ID)
            .with_compute_workload(RenderGraphComputeWorkload::per_pixel(
                CONTACT_SHADOW_PIPELINE_LABEL,
                CONTACT_SHADOW_WORKGROUP_SIZE,
                PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
                [
                    CONTACT_SHADOW_WORKGROUP_SIZE[0],
                    CONTACT_SHADOW_WORKGROUP_SIZE[1],
                ],
            ))
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .write_storage_texture(PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION),
        ],
    )
}

pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new_executor(
        EXECUTOR_ID,
        Arc::new(ContactShadowRenderPassExecutor::default()),
    )
}

#[derive(Default)]
struct ContactShadowRenderPassExecutor {
    pipeline: Mutex<Option<ContactShadowPipelineCache>>,
}

impl RenderPassExecutor for ContactShadowRenderPassExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context)?;

        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let Some(device_epoch) = gpu.device_epoch() else {
            return Err(
                "contact shadow executor requires a materialized device epoch before pipeline recording"
                    .to_string(),
            );
        };
        let depth_view = gpu.require_texture_view(
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let normal_view = gpu.require_texture_view(
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            RenderGraphResourceAccessKind::Read,
        )?;
        let hzb_view = gpu.require_texture_view(
            PostProcessGraphResourceNames::HZB_FURTHEST,
            RenderGraphResourceAccessKind::Read,
        )?;
        let output_view = gpu.require_texture_view(
            PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
            RenderGraphResourceAccessKind::Write,
        )?;

        let mut pipeline_guard = self
            .pipeline
            .lock()
            .map_err(|_| "contact shadow pipeline cache lock poisoned".to_string())?;
        let cache_matches = pipeline_guard
            .as_ref()
            .is_some_and(|cached| cached.device_epoch == device_epoch);
        if !cache_matches {
            drop(pipeline_guard.take());
            let native = gpu.native_context();
            let pipeline = ContactShadowPipeline::new(&native);
            drop(native);
            *pipeline_guard = Some(ContactShadowPipelineCache {
                device_epoch,
                pipeline,
            });
        }
        let pipeline = &pipeline_guard
            .as_ref()
            .expect("contact shadow pipeline cache was initialized")
            .pipeline;
        let native = gpu.native_context();
        let bind_group = native.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-contact-shadow-bind-group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(hzb_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
            ],
        });
        drop(native);
        let viewport_size = gpu.viewport_size();
        let dispatch_groups = [
            viewport_size
                .x
                .max(1)
                .div_ceil(CONTACT_SHADOW_WORKGROUP_SIZE[0]),
            viewport_size
                .y
                .max(1)
                .div_ceil(CONTACT_SHADOW_WORKGROUP_SIZE[1]),
            1,
        ];
        {
            let mut native = gpu.native_context();
            let mut pass = native
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(PASS_NAME),
                    timestamp_writes: None,
                });
            pass.set_pipeline(&pipeline.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(dispatch_groups[0], dispatch_groups[1], dispatch_groups[2]);
        }
        gpu.record_compute_dispatch(
            pass_name,
            executor_id,
            CONTACT_SHADOW_PIPELINE_LABEL,
            CONTACT_SHADOW_WORKGROUP_SIZE,
            dispatch_groups,
            vec![PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION.to_string()],
        );
        Ok(())
    }
}

struct ContactShadowPipelineCache {
    device_epoch: RenderPassDeviceEpoch,
    pipeline: ContactShadowPipeline,
}

struct ContactShadowPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl ContactShadowPipeline {
    fn new(factory: &impl RenderPassGpuResourceFactory) -> Self {
        let bind_group_layout =
            factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-contact-shadow-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-contact-shadow-shader"),
            source: wgpu::ShaderSource::Wgsl(CONTACT_SHADOW_SHADER_SOURCE.into()),
        });
        let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-contact-shadow-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(CONTACT_SHADOW_PIPELINE_LABEL),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            bind_group_layout,
            pipeline,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RenderPassExecutorContract {
    pass_name: &'static str,
    executor_id: &'static str,
    declared_queue: QueueLane,
    flags: PassFlags,
    resources: &'static [ExpectedResource],
}

#[derive(Clone, Copy, Debug)]
struct ExpectedResource {
    name: &'static str,
    kind: ExpectedResourceKind,
    access: RenderGraphResourceAccessKind,
}

#[derive(Clone, Copy, Debug)]
enum ExpectedResourceKind {
    Exact(RenderGraphResourceKind),
    AnyOf(&'static [RenderGraphResourceKind]),
}

impl ExpectedResource {
    const fn new(
        name: &'static str,
        kind: RenderGraphResourceKind,
        access: RenderGraphResourceAccessKind,
    ) -> Self {
        Self {
            name,
            kind: ExpectedResourceKind::Exact(kind),
            access,
        }
    }

    const fn any_of(
        name: &'static str,
        kinds: &'static [RenderGraphResourceKind],
        access: RenderGraphResourceAccessKind,
    ) -> Self {
        Self {
            name,
            kind: ExpectedResourceKind::AnyOf(kinds),
            access,
        }
    }

    fn description(self) -> String {
        describe_expected_resource(self.name, self.kind, self.access)
    }

    fn matches(self, resource: &RenderGraphPassResourceAccess) -> bool {
        self.name == resource.name
            && self.access == resource.access
            && self.kind.matches(resource.kind)
    }
}

impl ExpectedResourceKind {
    fn matches(self, kind: RenderGraphResourceKind) -> bool {
        match self {
            Self::Exact(expected) => expected == kind,
            Self::AnyOf(expected) => expected.contains(&kind),
        }
    }
}

const READ_ONLY_TEXTURE_INPUT_KINDS: &[RenderGraphResourceKind] = &[
    RenderGraphResourceKind::External,
    RenderGraphResourceKind::TransientTexture,
];

const CONTACT_SHADOW_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::any_of(
        PostProcessGraphResourceNames::SCENE_DEPTH,
        READ_ONLY_TEXTURE_INPUT_KINDS,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::any_of(
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        READ_ONLY_TEXTURE_INPUT_KINDS,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        PostProcessGraphResourceNames::HZB_FURTHEST,
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Write,
    ),
];

const CONTACT_SHADOW_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: PASS_NAME,
    executor_id: EXECUTOR_ID,
    declared_queue: QueueLane::AsyncCompute,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: true,
    },
    resources: CONTACT_SHADOW_RESOURCES,
};

fn validate_context(context: &RenderPassExecutionContext<'_>) -> Result<(), String> {
    let contract = &CONTACT_SHADOW_CONTRACT;
    if context.executor_id.as_str() != contract.executor_id {
        return Err(format!(
            "contact shadow executor contract mismatch: pass `{}` expected executor `{}`, got `{}`",
            context.pass_name, contract.executor_id, context.executor_id
        ));
    }
    if context.pass_name != contract.pass_name {
        return Err(format!(
            "contact shadow executor `{}` received pass `{}`, expected `{}`",
            contract.executor_id, context.pass_name, contract.pass_name
        ));
    }
    if context.declared_queue != contract.declared_queue {
        return Err(format!(
            "contact shadow executor `{}` declared queue mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id,
            context.pass_name,
            contract.declared_queue,
            context.declared_queue
        ));
    }
    if !queue_is_compatible(context.queue, contract.declared_queue) {
        return Err(format!(
            "contact shadow executor `{}` ran on incompatible queue for pass `{}`: declared `{:?}`, actual `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.queue
        ));
    }
    if context.flags != contract.flags {
        return Err(format!(
            "contact shadow executor `{}` pass flag mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.flags, context.flags
        ));
    }
    if !resource_contract_matches(contract.resources, &context.resources) {
        return Err(format!(
            "contact shadow executor `{}` resource contract mismatch for pass `{}`: expected {:?}, got {:?}",
            contract.executor_id,
            context.pass_name,
            expected_resource_descriptions(contract.resources),
            actual_resource_descriptions(&context.resources)
        ));
    }

    Ok(())
}

fn queue_is_compatible(actual: QueueLane, declared: QueueLane) -> bool {
    actual == declared || (declared != QueueLane::Graphics && actual == QueueLane::Graphics)
}

fn resource_contract_matches(
    expected: &[ExpectedResource],
    actual: &[RenderGraphPassResourceAccess],
) -> bool {
    if expected.len() != actual.len() {
        return false;
    }

    let mut matched = vec![false; actual.len()];
    for expected_resource in expected {
        let Some(index) = actual
            .iter()
            .enumerate()
            .find(|(index, resource)| !matched[*index] && expected_resource.matches(resource))
            .map(|(index, _)| index)
        else {
            return false;
        };
        matched[index] = true;
    }

    true
}

fn expected_resource_descriptions(resources: &[ExpectedResource]) -> Vec<String> {
    let mut descriptions = resources
        .iter()
        .map(|resource| resource.description())
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions
}

fn actual_resource_descriptions(resources: &[RenderGraphPassResourceAccess]) -> Vec<String> {
    let mut descriptions = resources
        .iter()
        .map(|resource| describe_resource(&resource.name, resource.kind, resource.access))
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions
}

fn describe_expected_resource(
    name: &str,
    kind: ExpectedResourceKind,
    access: RenderGraphResourceAccessKind,
) -> String {
    match kind {
        ExpectedResourceKind::Exact(kind) => describe_resource(name, kind, access),
        ExpectedResourceKind::AnyOf(kinds) => {
            let kinds = kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect::<Vec<_>>()
                .join("|");
            format!("{access:?}:{kinds}:{name}")
        }
    }
}

fn describe_resource(
    name: &str,
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
) -> String {
    format!("{access:?}:{kind:?}:{name}")
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("lib.rs");
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
}
