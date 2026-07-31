use std::collections::HashMap;

use crate::core::framework::project::ProjectPluginManifest;

use super::RuntimePluginFeatureRegistrationReport;

pub(super) type ProjectFeatureProviderLookup<'a> = HashMap<&'a str, &'a str>;

pub(super) fn project_feature_provider_lookup(
    manifest: &ProjectPluginManifest,
) -> ProjectFeatureProviderLookup<'_> {
    let mut providers = HashMap::new();
    for selection in &manifest.selections {
        for feature in &selection.features {
            providers
                .entry(feature.id.as_str())
                .or_insert_with(|| feature.provider_package_id_or_owner(selection.id.as_str()));
        }
    }
    providers
}

pub(super) fn feature_registration_matches_project_selection(
    registration: &RuntimePluginFeatureRegistrationReport,
    selected_providers: &ProjectFeatureProviderLookup<'_>,
    feature_id: &str,
) -> bool {
    selected_providers
        .get(feature_id)
        .is_some_and(|provider| registration.provider_package_id_or_owner() == *provider)
}
