//! Immutable, presentation-safe projection of one editor-plugin catalog generation.

use serde::Serialize;
use zircon_runtime::plugin::PluginModuleKind;

use super::registration::EditorPluginRegistrationReport;

/// Stable, read-only projection of an editor-plugin catalog generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EditorPluginCatalogProjection {
    entries: Vec<EditorPluginCatalogEntry>,
}

impl EditorPluginCatalogProjection {
    pub(super) fn from_registrations(registrations: &[EditorPluginRegistrationReport]) -> Self {
        let mut entries = registrations
            .iter()
            .map(EditorPluginCatalogEntry::from_registration)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.package_id.cmp(&right.package_id));
        Self { entries }
    }

    pub fn entries(&self) -> &[EditorPluginCatalogEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EditorPluginCatalogEntry {
    pub package_id: String,
    pub display_name: String,
    pub crate_name: String,
    pub category: String,
    pub capabilities: Vec<String>,
}

impl EditorPluginCatalogEntry {
    fn from_registration(registration: &EditorPluginRegistrationReport) -> Self {
        let package = &registration.package_manifest;
        let mut capabilities = registration.capabilities.clone();
        capabilities.sort();
        capabilities.dedup();
        let crate_name = package
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .map(|module| module.crate_name.clone())
            .unwrap_or_default();
        Self {
            package_id: package.id.clone(),
            display_name: package.display_name.clone(),
            crate_name,
            category: package.category.clone(),
            capabilities,
        }
    }
}
