//! 接触阴影的可选图契约与 WGPU 计算 executor；资源由图与设备宿主持有，executor 缓存按设备 epoch 更新。
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
#[path = "tests/wgpu_product_tests.rs"]
mod wgpu_product_tests;

/// 在环境遮蔽阶段读取深度、法线和最远 HZB，写入供后处理消费的短寿命可见度纹理。
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

/// 注册含设备 epoch 缓存的状态化计算 executor；每个注册句柄共享该实例直到目录撤销。
pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new_executor(
        EXECUTOR_ID,
        Arc::new(ContactShadowRenderPassExecutor::default()),
    )
}

#[derive(Default)]
// 同一 executor 可跨帧复用，但 native pipeline 只能用于创建它的设备 epoch。
struct ContactShadowRenderPassExecutor {
    pipeline: Mutex<Option<ContactShadowPipelineCache>>,
}

impl RenderPassExecutor for ContactShadowRenderPassExecutor {
    // 先核对图合同与设备 epoch，再取得绑定资源并录制计算；失败必须在提交前显式返回。
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
        // 设备更替后先释放旧 native 句柄，再用当前设备重建，避免复用上一设备的 WGPU 对象。
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

// 缓存键必须包含实际物化的设备 epoch，不能只按同名通道或 shader 标签复用。
struct ContactShadowPipelineCache {
    device_epoch: RenderPassDeviceEpoch,
    pipeline: ContactShadowPipeline,
}

// 绑定布局与管线共同从单一 GPU factory 生成；不能跨设备拆分或独立延长寿命。
struct ContactShadowPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl ContactShadowPipeline {
    // 与内嵌 WGSL 的四个绑定保持同一顺序与纹理格式，输出需匹配图分配的 RGBA8 存储纹理。
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
// 运行期拒绝编译图与 executor 的资源、队列或通道身份漂移，避免录制未声明的 GPU 访问。
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

// 每个资源的名称、种类与读写方向需与图描述符及 WGPU 绑定布局同步。
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

// 正常帧使用编译图的元数据，不应只用测试专门构造的静态合同验证成功路径。
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
    // BUG: [CR-PLUGIN-RENDERING-0008] 正常编译的接触阴影通道 has_side_effects=false，而静态 executor 合同写 true，故执行前必在此分支返回 flag mismatch；证据：同文件图测试断言 false，伪 context 测试直接从静态合同构造 true。
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

// 异步计算可以被宿主合并进图形队列；其余队列迁移仍视为图与 executor 合同不一致。
fn queue_is_compatible(actual: QueueLane, declared: QueueLane) -> bool {
    actual == declared || (declared != QueueLane::Graphics && actual == QueueLane::Graphics)
}

// 按资源身份匹配而不依赖图访问列表顺序，仍要求数量完全相等且不可重复消费一行。
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

// 单元测试分别检查声明图和伪造执行上下文；真实 WGPU 帧的合同一致性需由产品测试验证。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
