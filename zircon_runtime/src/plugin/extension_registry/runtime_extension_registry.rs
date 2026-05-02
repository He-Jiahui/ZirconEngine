use crate::asset::AssetImporterRegistry;
use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::{
    plugin::ComponentTypeDescriptor, plugin::PluginEventCatalogManifest, plugin::PluginOptionManifest,
    plugin::UiComponentDescriptor,
};

#[derive(Clone, Debug, Default)]
pub struct RuntimeExtensionRegistry {
    pub(super) managers: Vec<ManagerDescriptor>,
    pub(super) modules: Vec<ModuleDescriptor>,
    pub(super) render_features: Vec<RenderFeatureDescriptor>,
    pub(super) render_pass_executors: Vec<RenderPassExecutorRegistration>,
    pub(super) virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
    pub(super) components: Vec<ComponentTypeDescriptor>,
    pub(super) ui_components: Vec<UiComponentDescriptor>,
    pub(super) plugin_options: Vec<PluginOptionManifest>,
    pub(super) plugin_event_catalogs: Vec<PluginEventCatalogManifest>,
    pub(super) asset_importers: AssetImporterRegistry,
}
