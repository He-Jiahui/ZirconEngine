//! 平面反射的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::{
    planar_reflection_filter_compute_workload, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingPlanarReflectionsRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.planar_reflections";
pub const FEATURE_NAME: &str = "planar_reflections";
pub const FILTER_PASS: &str = zircon_runtime::graphics::PLANAR_FILTER_EXECUTOR_ID;

/// 仅为拥有平面探针目标的捕获相机添加过滤通道；写入宿主持有的持久 mip 链，以副作用保留通道。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec!["view".to_string(), "advanced_lighting".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            FILTER_PASS,
            QueueLane::AsyncCompute,
        )
        .with_executor_id(FILTER_PASS)
        .with_compute_workload(planar_reflection_filter_compute_workload())
        .with_side_effects()
        .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
        .write_storage_external_texture(
            zircon_runtime::graphics::PLANAR_REFLECTION_TEXTURE_RESOURCE,
        )],
    )
    .when_advanced_lighting_planar_capture_enabled()
}

/// 提供与特性图匹配的执行实现；宿主负责实际 GPU 资源与这些句柄的设备生命周期。
pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::planar_reflection_render_pass_executor_registrations()
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
