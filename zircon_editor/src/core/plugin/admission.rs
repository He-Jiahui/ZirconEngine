//! Admission checks that must finish before an editor-plugin catalog is published.

use std::collections::{BTreeMap, BTreeSet};
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
    let mut completed = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    let mut path = Vec::new();
    for package_id in dependencies_by_package.keys() {
        if let Some(cycle) = visit_dependency(
            package_id,
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

fn visit_dependency(
    package_id: &str,
    dependencies_by_package: &BTreeMap<String, BTreeSet<String>>,
    completed: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if completed.contains(package_id) {
        return None;
    }
    if !visiting.insert(package_id.to_string()) {
        let cycle_start = path
            .iter()
            .position(|candidate| candidate == package_id)
            .expect("a visiting package is always on the dependency path");
        let mut cycle = path[cycle_start..].to_vec();
        cycle.push(package_id.to_string());
        return Some(cycle);
    }

    path.push(package_id.to_string());
    if let Some(dependencies) = dependencies_by_package.get(package_id) {
        for dependency_id in dependencies {
            if dependencies_by_package.contains_key(dependency_id) {
                if let Some(cycle) = visit_dependency(
                    dependency_id,
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
    completed.insert(package_id.to_string());
    None
}

#[cfg(test)]
mod tests {
    use zircon_runtime::plugin::{PluginDependencyManifest, PluginPackageManifest};

    use crate::core::plugin::{EditorPluginCatalog, EditorPluginDescriptor};

    use super::{validate_catalog_admission, EditorPluginCatalogAdmissionError};

    #[test]
    fn rejects_a_cycle_between_declared_catalog_packages() {
        let catalog = catalog_with_dependencies(&[
            ("plugin.alpha", "plugin.beta"),
            ("plugin.beta", "plugin.alpha"),
        ]);

        assert_eq!(
            validate_catalog_admission(&catalog),
            Err(EditorPluginCatalogAdmissionError::DependencyCycle {
                package_ids: vec![
                    "plugin.alpha".to_string(),
                    "plugin.beta".to_string(),
                    "plugin.alpha".to_string(),
                ],
            })
        );
    }

    #[test]
    fn ignores_dependencies_outside_the_published_catalog() {
        let catalog = catalog_with_dependencies(&[("plugin.alpha", "plugin.external")]);

        assert_eq!(validate_catalog_admission(&catalog), Ok(()));
    }

    #[test]
    fn rejects_duplicate_runtime_manifest_input_for_one_editor_package() {
        let catalog = EditorPluginCatalog::from_descriptors(
            [EditorPluginDescriptor::new(
                "plugin.alpha",
                "Alpha",
                "alpha",
            )],
            [
                PluginPackageManifest::new("plugin.alpha", "Alpha"),
                PluginPackageManifest::new("plugin.alpha", "Conflicting Alpha"),
            ],
        );

        assert_eq!(
            validate_catalog_admission(&catalog),
            Err(EditorPluginCatalogAdmissionError::DuplicatePackage {
                package_id: "plugin.alpha".to_string(),
            })
        );
    }

    #[test]
    fn ignores_duplicate_runtime_only_manifest_input() {
        let catalog = EditorPluginCatalog::from_descriptors(
            [EditorPluginDescriptor::new(
                "plugin.alpha",
                "Alpha",
                "alpha",
            )],
            [
                PluginPackageManifest::new("plugin.alpha", "Alpha"),
                PluginPackageManifest::new("runtime.only", "Runtime Only"),
                PluginPackageManifest::new("runtime.only", "Conflicting Runtime Only"),
            ],
        );

        assert_eq!(validate_catalog_admission(&catalog), Ok(()));
    }

    fn catalog_with_dependencies(dependencies: &[(&str, &str)]) -> EditorPluginCatalog {
        let descriptors = dependencies
            .iter()
            .map(|(package_id, _)| {
                EditorPluginDescriptor::new(*package_id, *package_id, *package_id)
            })
            .collect::<Vec<_>>();
        let manifests = dependencies
            .iter()
            .map(|(package_id, dependency_id)| {
                let mut manifest = PluginPackageManifest::new(*package_id, *package_id);
                manifest
                    .dependencies
                    .push(PluginDependencyManifest::new(*dependency_id, true));
                manifest
            })
            .collect::<Vec<_>>();
        EditorPluginCatalog::from_descriptors(descriptors, manifests)
    }
}
