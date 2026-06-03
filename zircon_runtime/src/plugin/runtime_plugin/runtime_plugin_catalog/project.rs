use crate::plugin::{ProjectPluginManifest, ProjectPluginSelection};

use super::project_manifest::{
    catalog_project_manifest, complete_project_manifest as complete_catalog_project_manifest,
    project_selection_for_package as lookup_project_selection_for_package,
};
use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn project_manifest(&self) -> ProjectPluginManifest {
        catalog_project_manifest(&self.registrations, &self.feature_registrations)
    }

    pub fn complete_project_manifest(
        &self,
        manifest: &ProjectPluginManifest,
    ) -> ProjectPluginManifest {
        complete_catalog_project_manifest(
            &self.registrations,
            &self.feature_registrations,
            manifest,
        )
    }

    pub fn project_selection_for_package(
        &self,
        package_id: &str,
    ) -> Option<ProjectPluginSelection> {
        lookup_project_selection_for_package(&self.registrations, package_id)
    }
}
