//! 贴花的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
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
    RenderingDecalsRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.decals";
pub const FEATURE_NAME: &str = "decals";
pub const EXECUTOR_ID: &str = "decals.projector-composite";
pub const DECAL_PROJECTOR_COMPONENT_TYPE: &str = "rendering.Component.DecalProjector";

/// 作者选择的投影路径；此枚举只描述意图，当前注册元数据未据此切换 executor。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecalProjectionMode {
    ScreenSpace,
    Deferred,
}

/// 作者侧的贴花参数载体；组件反射表只公开字段类型，不替调用方验证数值范围或图集引用。
#[derive(Clone, Debug, PartialEq)]
pub struct DecalProjectorDescriptor {
    pub mode: DecalProjectionMode,
    pub opacity: f32,
    pub normal_blend: f32,
    pub atlas_region: String,
}

impl Default for DecalProjectorDescriptor {
    fn default() -> Self {
        Self {
            mode: DecalProjectionMode::Deferred,
            opacity: 1.0,
            normal_blend: 0.0,
            atlas_region: "default".to_string(),
        }
    }
}

/// 将贴花字段交给场景组件反射注册；登记此描述符不会创建投影器实例或绘制内容。
pub fn decal_projector_component_descriptor(
) -> zircon_runtime::core::framework::scene::ComponentTypeDescriptor {
    zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
        DECAL_PROJECTOR_COMPONENT_TYPE,
        zircon_plugin_rendering_runtime::PLUGIN_ID,
        "Decal Projector",
    )
    .with_property("mode", "enum:screen_space|deferred", true)
    .with_property("opacity", "float", true)
    .with_property("normal_blend", "float", true)
    .with_property("atlas_region", "string", true)
}

/// 向图编译声明贴花对深度和场景颜色的依赖；组件元数据登记与实际合成执行是不同步骤。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "decals".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "decal-projector-composite",
            QueueLane::Graphics,
        )
        .with_executor_id(EXECUTOR_ID)
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

/// 为此特性的 executor 标识提供实现句柄；与图描述符一并安装到同一运行期目录。
pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

// TODO: [CR-PLUGIN-RENDERING-0002] 确认组件参数到贴花像素的消费链；此回调不读取组件或录制 GPU 命令，现有测试仅查组件与通道；下一步核对实际场景渲染并补投影效果证据。
fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
