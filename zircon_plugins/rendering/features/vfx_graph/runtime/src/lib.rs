//! 视效图的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingVfxGraphRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.vfx_graph";
pub const FEATURE_NAME: &str = "vfx_graph";
pub const SIMULATION_EXECUTOR_ID: &str = "vfx-graph.simulate";
pub const TRANSPARENT_EXECUTOR_ID: &str = "vfx-graph.transparent";
pub const VFX_EMITTER_COMPONENT_TYPE: &str = "rendering.Component.VfxEmitter";
const VFX_GRAPH_SIMULATION_PIPELINE_LABEL: &str = "zircon-vfx-graph-simulate";
const VFX_GRAPH_SIMULATION_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
const VFX_GRAPH_SIMULATION_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];

#[derive(Clone, Debug, PartialEq)]
/// 作者侧的视效图配置；粒子容量是资源规划上限，不由当前固定工作量自动推导。
pub struct VfxGraphAsset {
    pub name: String,
    pub max_particles: u32,
    pub nodes: Vec<VfxGraphNode>,
}

#[derive(Clone, Debug, PartialEq)]
/// 描述生成、生命周期与材质依赖；编译报告当前只做必要节点检查。
pub enum VfxGraphNode {
    SpawnRate { particles_per_second: f32 },
    Lifetime { seconds: f32 },
    Velocity { value: [f32; 3] },
    ColorOverLife { start: [f32; 4], end: [f32; 4] },
    ShaderGraphMaterial { shader_graph: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 特性通道名称与诊断；此报告本身不包含可提交 GPU 的 shader 或粒子程序。
pub struct VfxGraphCompileReport {
    pub simulation_pass: String,
    pub render_pass: String,
    pub diagnostics: Vec<String>,
}

/// 在将资产映射到运行时前检查必需节点，供作者或导出诊断；没有执行程序生成步骤。
pub fn compile_vfx_graph(asset: &VfxGraphAsset) -> VfxGraphCompileReport {
    let mut diagnostics = Vec::new();
    if asset.max_particles == 0 {
        diagnostics.push(format!("vfx graph `{}` has zero max particles", asset.name));
    }
    if !asset
        .nodes
        .iter()
        .any(|node| matches!(node, VfxGraphNode::SpawnRate { .. }))
    {
        diagnostics.push(format!("vfx graph `{}` has no spawn node", asset.name));
    }
    if !asset
        .nodes
        .iter()
        .any(|node| matches!(node, VfxGraphNode::ShaderGraphMaterial { .. }))
    {
        diagnostics.push(format!(
            "vfx graph `{}` has no shader graph material",
            asset.name
        ));
    }
    VfxGraphCompileReport {
        simulation_pass: "vfx-graph-simulate".to_string(),
        render_pass: "vfx-graph-transparent".to_string(),
        diagnostics,
    }
}

/// 为场景反射声明视效图资产引用；实例提取与 GPU 粒子缓冲区由后续运行时链负责。
pub fn vfx_emitter_component_descriptor(
) -> zircon_runtime::core::framework::scene::ComponentTypeDescriptor {
    zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
        VFX_EMITTER_COMPONENT_TYPE,
        zircon_plugin_rendering_runtime::PLUGIN_ID,
        "VFX Emitter",
    )
    .with_property("graph", "asset:vfx_graph", true)
    .with_property("rate_multiplier", "float", true)
}

/// 声明粒子状态写入与透明绘制读取的依赖；固定工作量是当前图契约，不按资产粒子上限调整。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "vfx".to_string(),
            "particles".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent,
                "vfx-graph-simulate",
                QueueLane::AsyncCompute,
            )
            .with_executor_id(SIMULATION_EXECUTOR_ID)
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                VFX_GRAPH_SIMULATION_PIPELINE_LABEL,
                VFX_GRAPH_SIMULATION_WORKGROUP_SIZE,
                VFX_GRAPH_SIMULATION_DISPATCH_GROUPS,
            ))
            .write_buffer("vfx-particle-state"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent,
                "vfx-graph-transparent",
                QueueLane::Graphics,
            )
            .with_executor_id(TRANSPARENT_EXECUTOR_ID)
            .read_buffer("vfx-particle-state")
            .read_texture("scene-depth")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
}

/// 提供与特性图匹配的执行实现；宿主负责实际 GPU 资源与这些句柄的设备生命周期。
pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(SIMULATION_EXECUTOR_ID, noop_render_executor),
        RenderPassExecutorRegistration::new(TRANSPARENT_EXECUTOR_ID, noop_render_executor),
    ]
}

// TODO: [CR-PLUGIN-RENDERING-0006] 确认视效图的模拟与透明绘制是否由其它 owner 处理；两个已注册通道都绑定空 executor，且模拟工作量固定为一组；下一步核对组件提取与资产容量对应的产品行为。
fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
