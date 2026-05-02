mod builtin_catalog;
mod package_manifest;
mod project_selection;
mod runtime_plugin;
mod runtime_plugin_catalog;
mod runtime_plugin_descriptor;
mod runtime_plugin_descriptor_builder;
mod runtime_plugin_feature_registration_report;
mod runtime_plugin_registration_report;

pub use runtime_plugin::RuntimePlugin;
pub use runtime_plugin::RuntimePluginFeature;
pub use runtime_plugin_catalog::{
    RuntimeExtensionCatalogReport, RuntimePluginCatalog, RuntimePluginFeatureBlock,
    RuntimePluginFeatureDependencyReport,
};
pub use runtime_plugin_descriptor::RuntimePluginDescriptor;
pub use runtime_plugin_feature_registration_report::RuntimePluginFeatureRegistrationReport;
pub use runtime_plugin_registration_report::RuntimePluginRegistrationReport;
