//! 光线追踪策略的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::graphics::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingRayTracingPolicyRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.ray_tracing_policy";
pub const FEATURE_NAME: &str = "ray_tracing_policy";

/// 后端能力的静态快照；路径选择需同时考虑图形宿主实际创建的设备与目标 profile。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RayTracingBackendCapabilities {
    pub acceleration_structures: bool,
    pub inline_ray_query: bool,
    pub ray_tracing_pipeline: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 用户请求的射线执行方式；Disabled 不要求硬件能力，另两类具有不同能力门槛。
pub enum RayTracingPath {
    Disabled,
    InlineQuery,
    Pipeline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 路径与后端能力的比对结果；仅是策略诊断，不创建加速结构或 shader 管线。
pub struct RayTracingPolicyReport {
    pub requested_path: RayTracingPath,
    pub supported: bool,
    pub missing_gates: Vec<RenderFeatureCapabilityRequirement>,
}

impl RayTracingPolicyReport {
    /// 在选择渲染路径前列出该路径真正缺失的能力，供 profile 回退或诊断使用。
    pub fn from_backend(
        requested_path: RayTracingPath,
        backend: RayTracingBackendCapabilities,
    ) -> Self {
        let mut missing_gates = Vec::new();
        if requested_path != RayTracingPath::Disabled && !backend.acceleration_structures {
            missing_gates.push(RenderFeatureCapabilityRequirement::AccelerationStructures);
        }
        if requested_path == RayTracingPath::InlineQuery && !backend.inline_ray_query {
            missing_gates.push(RenderFeatureCapabilityRequirement::InlineRayQuery);
        }
        if requested_path == RayTracingPath::Pipeline && !backend.ray_tracing_pipeline {
            missing_gates.push(RenderFeatureCapabilityRequirement::RayTracingPipeline);
        }
        Self {
            requested_path,
            supported: missing_gates.is_empty(),
            missing_gates,
        }
    }
}

/// 提供能力需求声明，不产生渲染通道；图形宿主会校验全部声明的硬件能力。
// TODO: [CR-PLUGIN-RENDERING-0003] 确认图描述符为何同时要求 InlineRayQuery 与 RayTracingPipeline；from_backend 对两种请求路径只查各自门槛，图能力聚合却会要求全部三项；下一步以仅支持单一路径的后端验证 profile。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(FEATURE_NAME, Vec::new(), Vec::new(), Vec::new())
        .with_capability_requirement(RenderFeatureCapabilityRequirement::AccelerationStructures)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::InlineRayQuery)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline)
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
