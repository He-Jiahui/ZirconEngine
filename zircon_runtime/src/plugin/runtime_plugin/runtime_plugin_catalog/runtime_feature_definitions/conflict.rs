use std::collections::HashMap;

use super::super::feature_definitions::FeatureDefinition;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::registration_match::feature_definition_registration_matches;

pub(super) fn append_package_registration_conflict(
    registration: &RuntimePluginFeatureRegistrationReport,
    definitions: &HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    key: &str,
) {
    if let Some(declared) = definitions.get(key) {
        if !feature_definition_registration_matches(&declared.manifest, &registration.manifest) {
            diagnostics.push(format!(
                "optional feature id {} has conflicting package manifest and runtime registration",
                registration.manifest.id
            ));
        }
    }
}
