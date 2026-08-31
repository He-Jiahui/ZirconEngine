mod builtin_catalog;
mod capability_view;
mod descriptor;
mod feature_registration_report;
mod feature_validation;
mod module_validation;
mod package_validation;
mod registration_report;
mod runtime_plugin;
mod runtime_plugin_catalog;

pub use capability_view::CapabilityView;
pub use descriptor::{RuntimePluginDescriptor, RuntimePluginDescriptorBuilder};
pub use feature_registration_report::RuntimePluginFeatureRegistrationReport;
pub use registration_report::RuntimePluginRegistrationReport;
pub use runtime_plugin::RuntimePlugin;
pub use runtime_plugin::RuntimePluginFeature;
pub use runtime_plugin_catalog::{
    CompiledProjectPluginPlan, PluginCatalogGeneration, RuntimeExtensionCatalogReport,
    RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker,
    RuntimePluginBridgeLifecycleBlock, RuntimePluginBridgeLifecycleError,
    RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleOutcome,
    RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleState, RuntimePluginCatalog,
    RuntimePluginCatalogAuthority, RuntimePluginCatalogCandidate,
    RuntimePluginCatalogPreparationError, RuntimePluginCatalogPreparedGeneration,
    RuntimePluginCatalogProjectPlanCacheMetrics, RuntimePluginCatalogProjectPlanMetrics,
    RuntimePluginCatalogProjectionMetrics, RuntimePluginCatalogPublicationError,
    RuntimePluginCatalogSnapshot, RuntimePluginCatalogUpdate, RuntimePluginCatalogUpdateMetrics,
    RuntimePluginCatalogUpdateOutcome, RuntimePluginFeatureBlock,
    RuntimePluginFeatureDependencyReport, RuntimePluginModuleProposal,
};
