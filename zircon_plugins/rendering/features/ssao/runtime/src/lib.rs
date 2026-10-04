//! 屏幕空间环境遮蔽的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
use zircon_runtime::graphics::RenderFeatureDescriptor;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingSsaoRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.ssao";
pub const FEATURE_NAME: &str = "screen_space_ambient_occlusion";
pub const EXECUTOR_ID: &str = "compute.generic";

/// 复用图形宿主的计算与历史纹理契约；通用计算 executor 已由宿主提供，插件不重复登记。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    zircon_runtime::graphics::screen_space_ambient_occlusion_render_feature_descriptor()
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
