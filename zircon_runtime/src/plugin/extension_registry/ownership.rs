use crate::asset::AssetImporterDescriptor;

use super::typed_extension_point::ExtensionSlot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtensionOwnership {
    pub plugin_systems: Vec<ExtensionSlot>,
    pub plugin_runtime_systems: Vec<ExtensionSlot>,
    pub plugin_resources: Vec<ExtensionSlot>,
    pub plugin_events: Vec<ExtensionSlot>,
    pub plugin_interfaces: Vec<ExtensionSlot>,
    pub plugin_interface_imports: Vec<ExtensionSlot>,
    pub managers: Vec<ExtensionSlot>,
    pub modules: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub render_features: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub render_pass_executors: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub geometry_sources: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub shading_models: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub runtime_prepare_collectors: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub hybrid_gi_runtime_providers: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub solari_runtime_providers: Vec<ExtensionSlot>,
    #[cfg(feature = "graphics")]
    pub virtual_geometry_runtime_providers: Vec<ExtensionSlot>,
    pub components: Vec<ExtensionSlot>,
    #[cfg(feature = "ui")]
    pub ui_components: Vec<ExtensionSlot>,
    pub plugin_options: Vec<ExtensionSlot>,
    pub plugin_event_catalogs: Vec<ExtensionSlot>,
    pub asset_importers: Vec<AssetImporterDescriptor>,
}

impl ExtensionOwnership {
    pub fn is_empty(&self) -> bool {
        let empty = self.plugin_systems.is_empty()
            && self.plugin_runtime_systems.is_empty()
            && self.plugin_resources.is_empty()
            && self.plugin_events.is_empty()
            && self.plugin_interfaces.is_empty()
            && self.plugin_interface_imports.is_empty()
            && self.managers.is_empty()
            && self.modules.is_empty();
        #[cfg(feature = "graphics")]
        let empty = empty
            && self.render_features.is_empty()
            && self.render_pass_executors.is_empty()
            && self.geometry_sources.is_empty()
            && self.shading_models.is_empty()
            && self.runtime_prepare_collectors.is_empty()
            && self.hybrid_gi_runtime_providers.is_empty()
            && self.solari_runtime_providers.is_empty()
            && self.virtual_geometry_runtime_providers.is_empty();
        let empty = empty
            && self.components.is_empty()
            && self.plugin_options.is_empty()
            && self.plugin_event_catalogs.is_empty()
            && self.asset_importers.is_empty();
        #[cfg(feature = "ui")]
        let empty = empty && self.ui_components.is_empty();
        empty
    }
}
