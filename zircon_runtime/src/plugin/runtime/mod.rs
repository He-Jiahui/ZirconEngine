#[path = "builtin_catalog.rs"]
mod builtin_catalog;
#[path = "capability_view.rs"]
mod capability_view;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "registration.rs"]
mod feature_registration_report;
#[path = "feature_validation.rs"]
mod feature_validation;
#[path = "module_validation.rs"]
mod module_validation;
#[path = "validation.rs"]
mod package_validation;
#[path = "registration_report.rs"]
mod registration_report;
#[path = "runtime.rs"]
mod runtime_plugin;
#[path = "catalog.rs"]
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
