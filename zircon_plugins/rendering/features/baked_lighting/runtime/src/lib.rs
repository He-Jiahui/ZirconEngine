//! 烘焙光照的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingBakedLightingRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.baked_lighting";
pub const FEATURE_NAME: &str = "baked_lighting";
pub const EXECUTOR_ID: &str = "lighting.baked-composite";

/// 声明烘焙光照合成槽位，读取并续写帧内场景颜色；注册成功不代表已有光照合成实现。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id(EXECUTOR_ID)
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

/// 为此特性的 executor 标识提供实现句柄；与图描述符一并安装到同一运行期目录。
pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

// TODO: [CR-PLUGIN-RENDERING-0001] 确认默认启用的合成槽位由谁完成烘焙光照像素处理；此回调只返回成功，现有特性测试仅查注册；下一步追踪产品渲染消费并补效果证据。
fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
