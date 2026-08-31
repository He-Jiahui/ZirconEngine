use serde::{Deserialize, Serialize};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::{
    asset::AssetImporterDescriptor,
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportTargetPlatform,
    core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor},
    plugin::CapabilityStatusManifest,
    plugin::PluginMaturity,
    plugin::UiComponentDescriptor,
};

use super::{
    PluginDependencyManifest, PluginDistributionManifest, PluginEventCatalogManifest,
    PluginFeatureBundleManifest, PluginInterfaceManifest, PluginInterfaceMethodManifest,
    PluginModuleManifest, PluginOptionManifest, PluginPackageKind, PluginShaderPermutationManifest,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPackageManifest {
    pub id: String,
    pub version: String,
    #[serde(default = "default_sdk_api_version")]
    pub sdk_api_version: String,
    #[serde(default)]
    pub package_prefix: String,
    #[serde(default)]
    pub package_company: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub package_kind: PluginPackageKind,
    pub display_name: String,
    #[serde(default = "default_plugin_category")]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub supported_targets: Vec<RuntimeTargetMode>,
    #[serde(default)]
    pub supported_platforms: Vec<ExportTargetPlatform>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub capability_statuses: Vec<CapabilityStatusManifest>,
    #[serde(default)]
    pub maturity: PluginMaturity,
    #[serde(default)]
    pub asset_roots: Vec<String>,
    #[serde(default)]
    pub content_roots: Vec<String>,
    #[serde(default)]
    pub modules: Vec<PluginModuleManifest>,
    #[serde(default)]
    pub dependencies: Vec<PluginDependencyManifest>,
    #[serde(default)]
    pub provides_interfaces: Vec<PluginInterfaceManifest>,
    #[serde(default)]
    pub options: Vec<PluginOptionManifest>,
    #[serde(default)]
    pub event_catalogs: Vec<PluginEventCatalogManifest>,
    #[serde(default)]
    pub components: Vec<ComponentTypeDescriptor>,
    #[serde(default)]
    pub ui_components: Vec<UiComponentDescriptor>,
    #[serde(default)]
    pub asset_importers: Vec<AssetImporterDescriptor>,
    #[serde(default)]
    pub optional_features: Vec<PluginFeatureBundleManifest>,
    #[serde(default)]
    pub feature_extensions: Vec<PluginFeatureBundleManifest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geometry_sources: Vec<GeometrySourceDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shading_models: Vec<ShadingModelDescriptor>,
    #[serde(
        default,
        skip_serializing_if = "PluginShaderPermutationManifest::is_empty"
    )]
    pub shader_permutation: PluginShaderPermutationManifest,
    #[serde(default)]
    pub default_packaging: Vec<ExportPackagingStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<PluginDistributionManifest>,
}

fn default_plugin_category() -> String {
    "uncategorized".to_string()
}

fn default_sdk_api_version() -> String {
    "0.1.0".to_string()
}

impl PluginPackageManifest {
    pub fn package_id(&self) -> String {
        if self.package_prefix.is_empty()
            || self.package_company.is_empty()
            || self.package_name.is_empty()
        {
            return self.id.clone();
        }
        let capacity =
            self.package_prefix.len() + self.package_company.len() + self.package_name.len() + 2;
        let mut package_id = String::with_capacity(capacity);
        package_id.push_str(&self.package_prefix);
        package_id.push('.');
        package_id.push_str(&self.package_company);
        package_id.push('.');
        package_id.push_str(&self.package_name);
        package_id
    }

    pub fn asset_roots_or_default(&self) -> Vec<String> {
        if self.asset_roots.is_empty() {
            return vec!["assets".to_string()];
        }
        self.asset_roots.clone()
    }

    pub fn bridge_interface(&self, interface_id: &str) -> Option<&PluginInterfaceManifest> {
        self.provides_interfaces
            .iter()
            .find(|interface| interface.id == interface_id)
    }

    pub fn bridge_methods(
        &self,
    ) -> impl Iterator<Item = (&PluginInterfaceManifest, &PluginInterfaceMethodManifest)> {
        self.provides_interfaces.iter().flat_map(|interface| {
            interface
                .methods
                .iter()
                .map(move |method| (interface, method))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PluginPackageManifest;

    #[test]
    fn exact_package_coordinate_id_preserves_qualified_and_fallback_identity() {
        let qualified = PluginPackageManifest::new("legacy_weather", "Weather")
            .with_package_identity("org", "zircon", "weather");
        assert_eq!(qualified.package_id(), "org.zircon.weather");

        let fallback = PluginPackageManifest::new("legacy_weather", "Weather")
            .with_package_identity("", "zircon", "weather");
        assert_eq!(fallback.package_id(), "legacy_weather");
    }
}
