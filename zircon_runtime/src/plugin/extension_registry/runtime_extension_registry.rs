use crate::asset::AssetImporterRegistry;
use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::{
    plugin::ComponentTypeDescriptor, plugin::PluginEventCatalogManifest,
    plugin::PluginOptionManifest, plugin::SceneRuntimeHookRegistration,
    plugin::UiComponentDescriptor,
};

#[derive(Clone, Debug, Default)]
pub struct RuntimeExtensionRegistry {
    pub(super) managers: Vec<ManagerDescriptor>,
    pub(super) modules: Vec<ModuleDescriptor>,
    pub(super) render_features: Vec<RenderFeatureDescriptor>,
    pub(super) render_pass_executors: Vec<RenderPassExecutorRegistration>,
    pub(super) hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    pub(super) virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
    pub(super) components: Vec<ComponentTypeDescriptor>,
    pub(super) ui_components: Vec<UiComponentDescriptor>,
    pub(super) plugin_options: Vec<PluginOptionManifest>,
    pub(super) plugin_event_catalogs: Vec<PluginEventCatalogManifest>,
    pub(super) asset_importers: AssetImporterRegistry,
    pub(super) scene_hooks: Vec<SceneRuntimeHookRegistration>,
}
