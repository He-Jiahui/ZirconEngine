#[cfg(feature = "graphics")]
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::core::framework::scene::ComponentTypeDescriptor;
#[cfg(feature = "graphics")]
use crate::plugin::PluginModuleId;
#[cfg(feature = "ui")]
use crate::plugin::UiComponentDescriptor;
use crate::plugin::{PluginEventCatalogManifest, PluginOptionManifest};

use super::super::RuntimeExtensionRegistry;

// 只暴露描述符视图；调用方在合并或构建世界时读取，不应把这些切片当成可跨热重载的快照。
impl RuntimeExtensionRegistry {
    pub fn components(&self) -> &[ComponentTypeDescriptor] {
        self.components.values()
    }

    #[cfg(feature = "ui")]
    pub fn ui_components(&self) -> &[UiComponentDescriptor] {
        self.ui_components.values()
    }

    pub fn plugin_options(&self) -> &[PluginOptionManifest] {
        self.plugin_options.values()
    }

    pub fn plugin_event_catalogs(&self) -> &[PluginEventCatalogManifest] {
        self.plugin_event_catalogs.values()
    }

    #[cfg(feature = "graphics")]
    pub fn geometry_sources(&self) -> &[GeometrySourceDescriptor] {
        self.geometry_sources.values()
    }

    #[cfg(feature = "graphics")]
    // 目录合并必须同时保留 owner，才能按目标模块筛选并在卸载时撤销图形扩展。
    pub(in crate::plugin) fn geometry_source_entries(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &GeometrySourceDescriptor)> {
        self.geometry_sources
            .iter()
            .map(|(owner, _, descriptor)| (owner, descriptor))
    }

    #[cfg(feature = "graphics")]
    pub fn shading_models(&self) -> &[ShadingModelDescriptor] {
        self.shading_models.values()
    }

    #[cfg(feature = "graphics")]
    pub(in crate::plugin) fn shading_model_entries(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &ShadingModelDescriptor)> {
        self.shading_models
            .iter()
            .map(|(owner, _, descriptor)| (owner, descriptor))
    }
}
