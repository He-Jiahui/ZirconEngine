//! Admission checks that must finish before an editor-plugin catalog is published.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;

use super::catalog::EditorPluginCatalog;

/// A structural error that prevents a catalog generation from becoming visible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorPluginCatalogAdmissionError {
    DuplicatePackage { package_id: String },
    DependencyCycle { package_ids: Vec<String> },
}

impl fmt::Display for EditorPluginCatalogAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePackage { package_id } => {
                write!(
                    formatter,
                    "editor plugin catalog contains duplicate package `{package_id}`"
                )
            }
            Self::DependencyCycle { package_ids } => write!(
                formatter,
                "editor plugin catalog contains dependency cycle {}",
                package_ids.join(" -> ")
            ),
        }
    }
}

impl std::error::Error for EditorPluginCatalogAdmissionError {}

/// Rejects a catalog whose declared package dependencies form a cycle.
///
/// Native ABI and engine-version checks stay with the runtime native loader. This editor-side
/// admission boundary only checks the in-memory package graph before a generation is published.
pub(super) fn validate_catalog_admission(
    catalog: &EditorPluginCatalog,
) -> Result<(), EditorPluginCatalogAdmissionError> {
    if let Some(package_id) = catalog.admission_duplicate_package_ids().iter().next() {
        return Err(EditorPluginCatalogAdmissionError::DuplicatePackage {
            package_id: package_id.clone(),
        });
    }
    let mut dependencies_by_package = BTreeMap::<String, BTreeSet<String>>::new();
    for package in catalog.package_manifests() {
        let dependencies = package
            .dependencies
            .into_iter()
            .map(|dependency| dependency.id)
            .collect();
        if dependencies_by_package
            .insert(package.id.clone(), dependencies)
            .is_some()
        {
            return Err(EditorPluginCatalogAdmissionError::DuplicatePackage {
                package_id: package.id,
            });
        }
    }

    if let Some(package_ids) = find_dependency_cycle(&dependencies_by_package) {
        return Err(EditorPluginCatalogAdmissionError::DependencyCycle { package_ids });
    }
    Ok(())
}

fn find_dependency_cycle(
    dependencies_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    let mut completed = HashSet::with_capacity(dependencies_by_package.len());
    let mut visiting = HashSet::with_capacity(dependencies_by_package.len());
    let mut path = Vec::<&str>::new();
    for package_id in dependencies_by_package.keys() {
        if let Some(cycle) = visit_dependency(
            package_id.as_str(),
            dependencies_by_package,
            &mut completed,
            &mut visiting,
            &mut path,
        ) {
            return Some(cycle);
        }
    }
    None
}

fn visit_dependency<'a>(
    package_id: &'a str,
    dependencies_by_package: &'a BTreeMap<String, BTreeSet<String>>,
    completed: &mut HashSet<&'a str>,
    visiting: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    if completed.contains(package_id) {
        return None;
    }
    if !visiting.insert(package_id) {
        let cycle_start = path
            .iter()
            .position(|candidate| *candidate == package_id)
            .expect("a visiting package is always on the dependency path");
        let mut cycle = path[cycle_start..]
            .iter()
            .map(|package_id| (*package_id).to_string())
            .collect::<Vec<_>>();
        cycle.push(package_id.to_string());
        return Some(cycle);
    }

    path.push(package_id);
    if let Some(dependencies) = dependencies_by_package.get(package_id) {
        for dependency_id in dependencies {
            if dependencies_by_package.contains_key(dependency_id) {
                if let Some(cycle) = visit_dependency(
                    dependency_id.as_str(),
                    dependencies_by_package,
                    completed,
                    visiting,
                    path,
                ) {
                    return Some(cycle);
                }
            }
        }
    }
    path.pop();
    visiting.remove(package_id);
    completed.insert(package_id);
    None
}

#[cfg(test)]
#[path = "tests/admission.rs"]
mod tests;
