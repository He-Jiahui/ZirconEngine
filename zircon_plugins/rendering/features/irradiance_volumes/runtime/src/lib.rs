//! 辐照度体积的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage, IRRADIANCE_VOLUME_BIND_EXECUTOR_ID, IRRADIANCE_VOLUME_RESOURCE,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingIrradianceVolumesRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.irradiance_volumes";
pub const FEATURE_NAME: &str = "irradiance_volumes";
pub const VOLUME_BIND_PASS: &str = "irradiance.volume_bind";

/// 声明本帧已准备辐照度体积的外部绑定，供不透明与延迟材质通道读取；无消费者时允许裁剪。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "advanced_lighting".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            VOLUME_BIND_PASS,
            QueueLane::Graphics,
        )
        .with_executor_id(IRRADIANCE_VOLUME_BIND_EXECUTOR_ID)
        .write_external_texture(IRRADIANCE_VOLUME_RESOURCE)],
    )
    .when_advanced_lighting_irradiance_volumes_enabled()
    .with_pass_read_external_texture("opaque-mesh", IRRADIANCE_VOLUME_RESOURCE)
    .with_pass_read_external_texture("alpha-mask-mesh", IRRADIANCE_VOLUME_RESOURCE)
    .with_pass_read_external_texture("deferred-gbuffer", IRRADIANCE_VOLUME_RESOURCE)
}

/// 提供与特性图匹配的执行实现；宿主负责实际 GPU 资源与这些句柄的设备生命周期。
pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::irradiance_volume_render_pass_executor_registrations()
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
