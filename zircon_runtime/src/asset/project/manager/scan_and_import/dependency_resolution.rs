use std::collections::{HashMap, HashSet};

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
    let mut resolved = ResolvedDependencies {
        dependency_ids: Vec::with_capacity(dependencies.len()),
        diagnostics: Vec::new(),
    };
    let mut seen_dependency_ids = HashSet::with_capacity(dependencies.len());
    for dependency in dependencies {
        if let Some(record) = registry.get_by_locator(dependency) {
            admit_resolved_dependency_id(
                &mut resolved.dependency_ids,
                &mut seen_dependency_ids,
                record.id(),
            );
        } else {
            if resolved.diagnostics.is_empty() {
                resolved.diagnostics.reserve(dependencies.len());
            }
            resolved.diagnostics.push(ResourceDiagnostic::error(format!(
                "unresolved asset dependency {dependency}"
            )));
        }
    }
    resolved
}

fn admit_resolved_dependency_id(
    dependency_ids: &mut Vec<AssetId>,
    seen_dependency_ids: &mut HashSet<AssetId>,
    dependency_id: AssetId,
) {
    if seen_dependency_ids.insert(dependency_id) {
        dependency_ids.push(dependency_id);
    }
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
    if dependencies.is_empty() {
        return;
    }

    let meta_dependencies = &mut meta.dependencies;
    let meta_dependency_capacity = meta_dependencies.len().saturating_add(dependencies.len());
    let mut meta_dependency_index: HashSet<&AssetUri> =
        HashSet::with_capacity(meta_dependency_capacity);
    meta_dependency_index.extend(meta_dependencies.iter());

    let mut root = meta
        .entries
        .iter_mut()
        .find(|entry| entry.url.label().is_none());
    let mut root_dependency_index: Option<HashSet<&AssetUri>> = root.as_deref().map(|root| {
        let root_dependency_capacity = root.dependencies.len().saturating_add(dependencies.len());
        let mut index = HashSet::with_capacity(root_dependency_capacity);
        index.extend(root.dependencies.iter());
        index
    });

    const META_ADMISSION: u8 = 1;
    const ROOT_ADMISSION: u8 = 1 << 1;
    let mut admission_flags = Vec::with_capacity(dependencies.len());
    let mut meta_addition_count = 0;
    let mut root_addition_count = 0;
    for dependency in &dependencies {
        let meta_is_new = meta_dependency_index.insert(dependency);
        let root_is_new = root_dependency_index
            .as_mut()
            .map(|index| index.insert(dependency))
            .unwrap_or(false);
        admission_flags
            .push(u8::from(meta_is_new) * META_ADMISSION + u8::from(root_is_new) * ROOT_ADMISSION);
        meta_addition_count += usize::from(meta_is_new);
        root_addition_count += usize::from(root_is_new);
    }
    drop(meta_dependency_index);
    drop(root_dependency_index);

    meta_dependencies.reserve(meta_addition_count);
    let mut root_dependencies = root.map(|entry| &mut entry.dependencies);
    if let Some(root_dependencies) = root_dependencies.as_mut() {
        root_dependencies.reserve(root_addition_count);
    }

    for (dependency, flags) in dependencies.into_iter().zip(admission_flags) {
        let meta_is_new = flags & META_ADMISSION != 0;
        let root_is_new = flags & ROOT_ADMISSION != 0;
        match (meta_is_new, root_is_new) {
            (true, true) => {
                meta_dependencies.push(dependency.clone());
                root_dependencies
                    .as_mut()
                    .expect("root admission requires a root entry")
                    .push(dependency);
            }
            (true, false) => meta_dependencies.push(dependency),
            (false, true) => root_dependencies
                .as_mut()
                .expect("root admission requires a root entry")
                .push(dependency),
            (false, false) => {}
        }
    }
}

#[cfg(test)]
#[path = "tests/dependency_resolution_optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "dependency_resolution/tests/optimization_batch_jd_runtime643_tests.rs"]
mod optimization_batch_jd_runtime643_tests;

#[cfg(test)]
#[path = "dependency_resolution/tests/optimization_batch_runtime866_resolved_dependency_capacity_tests.rs"]
mod optimization_batch_runtime866_resolved_dependency_capacity_tests;
