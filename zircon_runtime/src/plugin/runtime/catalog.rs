use std::sync::{Arc, Mutex};

use crate::core::CoreError;

use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

#[path = "catalog/access.rs"]
mod access;
#[path = "catalog/bridge_dependencies.rs"]
mod bridge_dependencies;
#[path = "catalog/bridge_lifecycle.rs"]
mod bridge_lifecycle;
#[path = "catalog/bridge_lifecycle_state.rs"]
mod bridge_lifecycle_state;
#[path = "catalog/candidate.rs"]
mod candidate;
#[path = "catalog/contributions.rs"]
mod contributions;
#[path = "catalog/derived_projection.rs"]
mod derived_projection;
#[path = "catalog/descriptor_contributions.rs"]
mod descriptor_contributions;
#[path = "catalog/diagnostics.rs"]
mod diagnostics;
#[path = "catalog/extension_merge.rs"]
mod extension_merge;
#[path = "catalog/extension_report.rs"]
mod extension_report;
#[path = "catalog/feature_blocking.rs"]
mod feature_blocking;
#[path = "catalog/feature_capabilities.rs"]
mod feature_capabilities;
#[path = "catalog/feature_completion.rs"]
mod feature_completion;
#[path = "catalog/feature_definition_collect.rs"]
mod feature_definition_collection;
#[path = "catalog/feature_definitions.rs"]
mod feature_definitions;
#[path = "catalog/feature_dependencies.rs"]
mod feature_dependencies;
#[path = "catalog/feature_register_match.rs"]
mod feature_registration_match;
#[path = "catalog/feature_report.rs"]
mod feature_report;
#[path = "catalog/feature_resolution.rs"]
mod feature_resolution;
#[path = "catalog/feature_selection.rs"]
mod feature_selection;
#[path = "catalog/feature_status.rs"]
mod feature_status;
#[path = "catalog/feature_status_record.rs"]
mod feature_status_record;
#[path = "catalog/feature_support.rs"]
mod feature_support;
#[path = "catalog/features.rs"]
mod features;
#[path = "catalog/generation.rs"]
mod generation;
#[path = "catalog/package_feature_definitions.rs"]
mod package_feature_definitions;
#[path = "catalog/project.rs"]
mod project;
#[path = "catalog/project_extension_report.rs"]
mod project_extension_report;
#[path = "catalog/project_manifest.rs"]
mod project_manifest;
#[path = "catalog/publication.rs"]
mod publication;
#[path = "catalog/registration.rs"]
mod registration;
#[cfg(feature = "graphics")]
#[path = "catalog/render_contributions.rs"]
mod render_contributions;
#[path = "catalog/runtime_extensions.rs"]
mod runtime_extensions;
#[path = "catalog/definitions.rs"]
mod runtime_feature_definitions;
#[path = "catalog/runtime_module_target.rs"]
mod runtime_module_target;
#[path = "catalog/snapshot.rs"]
mod snapshot;
#[path = "catalog/status.rs"]
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

/// 保存同一代的注册行和派生投影；项目计划缓存只为该代及目标提供复用。
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
