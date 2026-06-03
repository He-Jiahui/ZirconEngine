use crate::plugin::{ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection};

use super::RuntimePluginFeatureRegistrationReport;

pub(super) fn feature_registration_matches_project_selection(
    registration: &RuntimePluginFeatureRegistrationReport,
    manifest: &ProjectPluginManifest,
    feature_id: &str,
) -> bool {
    let Some((owner_selection, feature_selection)) = feature_selection(manifest, feature_id) else {
        return false;
    };
    registration.provider_package_id_or_owner()
        == feature_selection.provider_package_id_or_owner(&owner_selection.id)
}

fn feature_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    feature_id: &str,
) -> Option<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    manifest.selections.iter().find_map(|selection| {
        selection
            .features
            .iter()
            .find(|feature| feature.id == feature_id)
            .map(|feature| (selection, feature))
    })
}
