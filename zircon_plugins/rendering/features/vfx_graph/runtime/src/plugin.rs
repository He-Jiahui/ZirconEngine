//! 视效图的链接式注册入口；导出宿主按项目特性选择调用报告函数，再合并贡献。
use crate::{
    render_feature_descriptor, render_pass_executor_registrations, vfx_emitter_component_descriptor,
};

/// 可重复构造的特性提供者；实例无渲染设备状态，资源生命周期由登记的 executor 或宿主管理。
#[derive(Clone, Debug)]
pub struct RenderingVfxGraphRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for RenderingVfxGraphRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    // 向本次报告的暂存表提交组件描述符、图拓扑和 executor；错误保留在报告诊断中，由目录决定是否接纳。
    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_component(vfx_emitter_component_descriptor())?;
        registry.register_render_feature(render_feature_descriptor())?;
        for registration in render_pass_executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
        Ok(())
    }
}

/// 提供用于 trait 注册的无设备实例；调用后仍须生成并检查注册报告。
pub fn runtime_plugin_feature() -> RenderingVfxGraphRuntimeFeature {
    RenderingVfxGraphRuntimeFeature
}

/// 供生成的链接式导出宿主收集贡献；必须检查报告诊断，不能仅凭取得报告判定可用。
pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

/// 由渲染主包生成此特性的目标模式、owner 与依赖关系；注册内容须保持同一身份。
pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::VfxGraph,
    )
}
