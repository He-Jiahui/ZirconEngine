use crate::plugin::ProjectPluginSelection;

use super::super::RuntimePluginRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn project_selection_for_package(
    registrations: &[RuntimePluginRegistrationReport],
    package_id: &str,
) -> Option<ProjectPluginSelection> {
    registrations
        .iter()
        .find(|registration| registration.package_manifest.id == package_id)
        .map(|registration| registration.project_selection.clone())
}
