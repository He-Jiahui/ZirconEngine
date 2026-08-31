use std::sync::{Arc, Mutex};

use crate::core::CoreError;

use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

mod access;
mod bridge_dependencies;
mod bridge_lifecycle;
mod bridge_lifecycle_state;
mod candidate;
mod contributions;
mod derived_projection;
mod descriptor_contributions;
mod diagnostics;
mod extension_merge;
mod extension_report;
mod feature_blocking;
mod feature_capabilities;
mod feature_completion;
mod feature_definition_collection;
mod feature_definitions;
mod feature_dependencies;
mod feature_registration_match;
mod feature_report;
mod feature_resolution;
mod feature_selection;
mod feature_status;
mod feature_status_record;
mod feature_support;
mod features;
mod generation;
mod package_feature_definitions;
mod project;
mod project_extension_report;
mod project_manifest;
mod publication;
mod registration;
#[cfg(feature = "graphics")]
mod render_contributions;
mod runtime_extensions;
mod runtime_feature_definitions;
mod runtime_module_target;
mod snapshot;
mod status;

pub use bridge_dependencies::{RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker};
pub use bridge_lifecycle::{
    RuntimePluginBridgeLifecycleBlock, RuntimePluginBridgeLifecycleError,
    RuntimePluginBridgeLifecycleReport,
};
pub use bridge_lifecycle_state::{
    RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleOutcome,
    RuntimePluginBridgeLifecycleState,
};
pub use candidate::{
    RuntimePluginCatalogCandidate, RuntimePluginCatalogPreparationError,
    RuntimePluginCatalogPreparedGeneration,
};
pub use derived_projection::RuntimePluginCatalogProjectionMetrics;
pub use extension_report::RuntimeExtensionCatalogReport;
pub use feature_report::{RuntimePluginFeatureBlock, RuntimePluginFeatureDependencyReport};
pub use generation::PluginCatalogGeneration;
pub use project::{
    CompiledProjectPluginPlan, RuntimePluginCatalogProjectPlanCacheMetrics,
    RuntimePluginCatalogProjectPlanMetrics, RuntimePluginModuleProposal,
};
pub use publication::{RuntimePluginCatalogAuthority, RuntimePluginCatalogPublicationError};
pub use registration::{
    RuntimePluginCatalogUpdate, RuntimePluginCatalogUpdateMetrics,
    RuntimePluginCatalogUpdateOutcome,
};
pub use snapshot::RuntimePluginCatalogSnapshot;

#[derive(Debug, Default)]
pub struct RuntimePluginCatalog {
    registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    projection: Arc<derived_projection::RuntimePluginCatalogProjection>,
    catalog_generation: PluginCatalogGeneration,
    projection_builds: u64,
    project_plans: Mutex<project::ProjectPlanCache>,
    module_order_error: Option<Arc<CoreError>>,
    diagnostics: Vec<String>,
}
