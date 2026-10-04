//! 后处理的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
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
    RenderingPostProcessRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.post_process";
pub const FEATURE_NAME: &str = "post_process";
pub const EXECUTOR_ID: &str = "post.stack";

/// 保留旧后处理槽位的图契约，续写当前帧场景颜色；实际效果执行仍取决于登记的 executor。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "post-process",
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

// TODO: [CR-PLUGIN-RENDERING-0007] 确认默认开启的旧后处理槽位是否仅用于兼容图拓扑；executor 不消费 scene-color，也不录制图形命令；下一步核对图资源初始化和内建后处理路径。
fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
