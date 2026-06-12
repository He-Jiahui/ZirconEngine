mod builtin_catalog;
mod descriptor;
mod feature_registration_report;
mod feature_validation;
mod lifecycle_context;
mod module_validation;
mod package_validation;
mod registration_report;
mod runtime_plugin;
mod runtime_plugin_catalog;

pub use descriptor::RuntimePluginDescriptor;
pub use feature_registration_report::RuntimePluginFeatureRegistrationReport;
pub use lifecycle_context::{CapabilityView, PluginFinishContext, PluginRuntimeContext};
pub use registration_report::RuntimePluginRegistrationReport;
pub use runtime_plugin::RuntimePlugin;
pub use runtime_plugin::RuntimePluginFeature;
pub use runtime_plugin_catalog::{
    RuntimeExtensionCatalogReport, RuntimePluginCatalog, RuntimePluginFeatureBlock,
    RuntimePluginFeatureDependencyReport,
};
