use std::collections::HashMap;

use crate::asset::{AssetId, AssetImportError, AssetUri, ImportedAsset};
use crate::core::resource::{
    ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceRegistryStaging,
};

use super::stage_project_resource;

#[derive(Default)]
struct ResolvedDependencies {
    dependency_ids: Vec<AssetId>,
    diagnostics: Vec<ResourceDiagnostic>,
}

fn resolve_dependencies(
    dependencies: &[AssetUri],
    registry: &ResourceRegistry,
) -> ResolvedDependencies {
    let mut resolved = ResolvedDependencies::default();
    for dependency in dependencies {
        if let Some(record) = registry.get_by_locator(dependency) {
            if !resolved.dependency_ids.contains(&record.id()) {
                resolved.dependency_ids.push(record.id());
            }
        } else {
            resolved.diagnostics.push(ResourceDiagnostic::error(format!(
                "unresolved asset dependency {dependency}"
            )));
        }
    }
    resolved
}

pub(super) fn resolve_imported_dependencies(
    registry: &mut ResourceRegistryStaging,
    imported: &mut [ResourceRecord],
    dependencies_by_id: &HashMap<AssetId, Vec<AssetUri>>,
) -> Result<(), AssetImportError> {
    let resolved_by_id = dependencies_by_id
        .iter()
        .map(|(id, dependencies)| (*id, resolve_dependencies(dependencies, registry)))
        .collect::<HashMap<_, _>>();

    for record in imported.iter_mut() {
        apply_resolved_dependencies(record, &resolved_by_id);
        stage_project_resource(registry, record.clone())?;
    }
    Ok(())
}

fn apply_resolved_dependencies(
    record: &mut ResourceRecord,
    resolved_by_id: &HashMap<AssetId, ResolvedDependencies>,
) {
    let Some(resolved) = resolved_by_id.get(&record.id()) else {
        return;
    };
    record.dependency_ids = resolved.dependency_ids.clone();
    record
        .diagnostics
        .extend(resolved.diagnostics.iter().cloned());
}

pub(super) fn dependencies_for_entry(
    meta: &crate::asset::project::AssetMetaDocument,
    locator: &AssetUri,
) -> Vec<AssetUri> {
    meta.entries
        .iter()
        .find(|entry| &entry.url == locator)
        .map(|entry| entry.dependencies.clone())
        .unwrap_or_else(|| meta.dependencies.clone())
}

pub(super) fn merge_handwritten_dependencies_into_meta(
    meta: &mut crate::asset::project::AssetMetaDocument,
    asset: &ImportedAsset,
) {
    let dependencies =
        crate::asset::registry::dependency_extractors::handwritten_dependencies(asset);
    for dependency in dependencies {
        if !meta.dependencies.contains(&dependency) {
            meta.dependencies.push(dependency.clone());
        }
        if let Some(root) = meta
            .entries
            .iter_mut()
            .find(|entry| entry.url.label().is_none())
        {
            if !root.dependencies.contains(&dependency) {
                root.dependencies.push(dependency);
            }
        }
    }
}
