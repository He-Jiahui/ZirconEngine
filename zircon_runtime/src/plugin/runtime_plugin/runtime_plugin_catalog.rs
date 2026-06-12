use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

mod access;
mod bridge_dependencies;
mod contributions;
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
mod render_contributions;
mod runtime_extensions;
mod runtime_feature_definitions;
mod status;

pub use bridge_dependencies::{RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker};
pub use extension_report::RuntimeExtensionCatalogReport;
pub use feature_report::{RuntimePluginFeatureBlock, RuntimePluginFeatureDependencyReport};

#[derive(Clone, Debug, Default)]
pub struct RuntimePluginCatalog {
    registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    diagnostics: Vec<String>,
}
