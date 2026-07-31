use std::collections::HashMap;
use std::sync::{atomic::AtomicU64, Arc, Mutex};

use crate::core::CoreError;

use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

mod access;
mod bridge_dependencies;
mod bridge_lifecycle;
mod bridge_lifecycle_state;
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
mod package_feature_definitions;
mod project;
mod project_extension_report;
mod project_manifest;
mod registration;
#[cfg(feature = "graphics")]
mod render_contributions;
mod runtime_extensions;
mod runtime_feature_definitions;
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
pub use derived_projection::RuntimePluginCatalogProjectionMetrics;
pub use extension_report::RuntimeExtensionCatalogReport;
pub use feature_report::{RuntimePluginFeatureBlock, RuntimePluginFeatureDependencyReport};
pub use project::RuntimePluginCatalogProjectPlanMetrics;
pub use registration::{
    RuntimePluginCatalogUpdate, RuntimePluginCatalogUpdateMetrics,
    RuntimePluginCatalogUpdateOutcome,
};

#[derive(Debug, Default)]
pub struct RuntimePluginCatalog {
    registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    projection: Arc<derived_projection::RuntimePluginCatalogProjection>,
    catalog_generation: u64,
    projection_builds: u64,
    // Serializes cache misses so one catalog generation builds each frozen plan once.
    project_plans: Mutex<HashMap<u8, Arc<project::CompiledProjectPluginPlan>>>,
    project_plan_builds: AtomicU64,
    module_order_error: Option<Arc<CoreError>>,
    diagnostics: Vec<String>,
}

impl Clone for RuntimePluginCatalog {
    fn clone(&self) -> Self {
        Self {
            registrations: self.registrations.clone(),
            feature_registrations: self.feature_registrations.clone(),
            projection: Arc::clone(&self.projection),
            catalog_generation: self.catalog_generation,
            projection_builds: self.projection_builds,
            project_plans: Mutex::new(HashMap::new()),
            project_plan_builds: AtomicU64::new(0),
            module_order_error: self.module_order_error.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}
