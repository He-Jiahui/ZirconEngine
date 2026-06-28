mod builtin_catalog;
mod constructors;
mod plugin_dependency_manifest;
mod plugin_distribution_manifest;
mod plugin_event_manifest;
mod plugin_feature_bundle_manifest;
mod plugin_feature_dependency;
mod plugin_interface_manifest;
mod plugin_module_kind;
mod plugin_module_manifest;
mod plugin_option_manifest;
mod plugin_package_kind;
mod plugin_package_manifest;
mod plugin_shader_permutation_manifest;

pub use plugin_dependency_manifest::PluginDependencyManifest;
pub use plugin_distribution_manifest::PluginDistributionManifest;
pub use plugin_event_manifest::{PluginEventCatalogManifest, PluginEventManifest};
pub use plugin_feature_bundle_manifest::PluginFeatureBundleManifest;
pub use plugin_feature_dependency::PluginFeatureDependency;
pub use plugin_interface_manifest::{PluginInterfaceManifest, PluginInterfaceMethodManifest};
pub use plugin_module_kind::PluginModuleKind;
pub use plugin_module_manifest::PluginModuleManifest;
pub use plugin_option_manifest::PluginOptionManifest;
pub use plugin_package_kind::PluginPackageKind;
pub use plugin_package_manifest::PluginPackageManifest;
pub use plugin_shader_permutation_manifest::{
    PluginShaderPermutationIdManifest, PluginShaderPermutationManifest,
};
