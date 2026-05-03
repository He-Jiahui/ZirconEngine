use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{
    plugin::NativePluginLoader, plugin::PluginFeatureBundleManifest,
    plugin::PluginFeatureDependency, plugin::PluginPackageManifest, plugin::ProjectPluginManifest,
    plugin::ProjectPluginSelection, plugin::RuntimePluginCatalog, RuntimeTargetMode,
};

use super::super::super::editor_manager::EditorManager;
use super::super::reports::EditorPluginFeatureSelectionUpdateReport;

impl EditorManager {
    pub fn set_project_plugin_feature_enabled(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        feature_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginFeatureSelectionUpdateReport, String> {
        let catalog = self.runtime_plugin_catalog();
        set_project_plugin_feature_enabled_with_catalog(
            &catalog,
            manifest,
            plugin_id,
            feature_id,
            enabled,
            "builtin plugin catalogs",
        )
    }

    pub fn set_native_aware_project_plugin_feature_enabled(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        feature_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginFeatureSelectionUpdateReport, String> {
        let catalog = self.native_aware_runtime_plugin_catalog(project_root);
        set_project_plugin_feature_enabled_with_catalog(
            &catalog,
            manifest,
            plugin_id,
            feature_id,
            enabled,
            "builtin or native plugin catalogs",
        )
    }

    pub fn enable_project_plugin_feature_dependencies(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        feature_id: &str,
    ) -> Result<EditorPluginFeatureSelectionUpdateReport, String> {
        let catalog = self.runtime_plugin_catalog();
        enable_project_plugin_feature_dependencies_with_catalog(
            &catalog,
            manifest,
            plugin_id,
            feature_id,
            "builtin plugin catalogs",
        )
    }

    pub fn enable_native_aware_project_plugin_feature_dependencies(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        feature_id: &str,
    ) -> Result<EditorPluginFeatureSelectionUpdateReport, String> {
        let catalog = self.native_aware_runtime_plugin_catalog(project_root);
        enable_project_plugin_feature_dependencies_with_catalog(
            &catalog,
            manifest,
            plugin_id,
            feature_id,
            "builtin or native plugin catalogs",
        )
    }

    fn native_aware_runtime_plugin_catalog(
        &self,
        project_root: impl AsRef<Path>,
    ) -> RuntimePluginCatalog {
        let builtin = self.runtime_plugin_catalog();
        let native_report =
            NativePluginLoader.discover(self.plugin_directory(project_root.as_ref()));
        RuntimePluginCatalog::from_registration_reports(
            builtin
                .registrations()
                .iter()
                .cloned()
                .chain(native_report.runtime_plugin_registration_reports()),
            builtin.feature_registrations().iter().cloned(),
        )
    }
}

fn set_project_plugin_feature_enabled_with_catalog(
    catalog: &RuntimePluginCatalog,
    manifest: &mut ProjectManifest,
    plugin_id: &str,
    feature_id: &str,
    enabled: bool,
    catalog_label: &str,
) -> Result<EditorPluginFeatureSelectionUpdateReport, String> {
    let mut candidate = catalog.complete_project_manifest(&manifest.plugins);
    let _feature = feature_manifest(catalog, plugin_id, feature_id)?;
    let owner_selection = project_selection_mut(&mut candidate, plugin_id, catalog_label)?;
    let feature_selection = owner_selection
        .features
        .iter_mut()
        .find(|selection| selection.id == feature_id)
        .ok_or_else(|| {
            format!("feature {feature_id} is not registered under plugin {plugin_id}")
        })?;
    if !enabled && feature_selection.required {
        return Err(format!("required feature {feature_id} cannot be disabled"));
    }
    feature_selection.enabled = enabled;

    if enabled {
        ensure_feature_can_enable(catalog, &candidate, feature_id)?;
    }

    let project_selection = project_selection(&candidate, plugin_id, catalog_label)?.clone();
    manifest.plugins.set_enabled(project_selection.clone());
    Ok(EditorPluginFeatureSelectionUpdateReport {
        plugin_id: plugin_id.to_string(),
        feature_id: feature_id.to_string(),
        enabled,
        project_selection,
        enabled_dependency_plugins: Vec::new(),
        enabled_dependency_features: Vec::new(),
        diagnostics: vec![format!(
            "feature {feature_id} on plugin {plugin_id} {}",
            if enabled { "enabled" } else { "disabled" }
        )],
    })
}

fn enable_project_plugin_feature_dependencies_with_catalog(
    catalog: &RuntimePluginCatalog,
    manifest: &mut ProjectManifest,
    plugin_id: &str,
    feature_id: &str,
    catalog_label: &str,
) -> Result<EditorPluginFeatureSelectionUpdateReport, String> {
    let packages = catalog.package_manifests();
    let feature = feature_manifest(catalog, plugin_id, feature_id)?;
    let mut candidate = catalog.complete_project_manifest(&manifest.plugins);
    let mut enabled_dependency_plugins = Vec::new();
    let mut enabled_dependency_features = Vec::new();
    let mut diagnostics = Vec::new();

    let mut dependency_stack = vec![feature.id.clone()];
    enable_feature_dependency_tree(
        &mut candidate,
        &packages,
        &feature,
        &mut enabled_dependency_plugins,
        &mut enabled_dependency_features,
        &mut diagnostics,
        &mut dependency_stack,
        catalog_label,
    )?;

    for selection in candidate.selections.iter().filter(|selection| {
        enabled_dependency_plugins.contains(&selection.id)
            || enabled_dependency_features.iter().any(|feature_id| {
                selection
                    .features
                    .iter()
                    .any(|feature| feature.id.as_str() == feature_id.as_str())
            })
    }) {
        manifest.plugins.set_enabled(selection.clone());
    }

    let project_selection = project_selection(&candidate, plugin_id, catalog_label)?.clone();
    if enabled_dependency_plugins.is_empty() && enabled_dependency_features.is_empty() {
        diagnostics.push(format!(
            "dependencies for feature {feature_id} on plugin {plugin_id} were already enabled"
        ));
    } else {
        diagnostics.push(format!(
            "enabled dependencies for feature {feature_id} on plugin {plugin_id}"
        ));
    }

    Ok(EditorPluginFeatureSelectionUpdateReport {
        plugin_id: plugin_id.to_string(),
        feature_id: feature_id.to_string(),
        enabled: project_selection
            .features
            .iter()
            .find(|selection| selection.id == feature_id)
            .map(|selection| selection.enabled)
            .unwrap_or(false),
        project_selection,
        enabled_dependency_plugins,
        enabled_dependency_features,
        diagnostics,
    })
}

fn enable_feature_dependency_tree(
    manifest: &mut ProjectPluginManifest,
    packages: &[PluginPackageManifest],
    feature: &PluginFeatureBundleManifest,
    enabled_dependency_plugins: &mut Vec<String>,
    enabled_dependency_features: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
    dependency_stack: &mut Vec<String>,
    catalog_label: &str,
) -> Result<(), String> {
    // Provider features can themselves provide stable capabilities for a larger bundle,
    // so dependency enablement walks the feature tree without enabling the requested feature.
    for dependency in &feature.dependencies {
        enable_dependency_plugin(
            manifest,
            dependency,
            enabled_dependency_plugins,
            catalog_label,
        )?;
        enable_dependency_feature_provider(
            manifest,
            packages,
            dependency,
            enabled_dependency_plugins,
            enabled_dependency_features,
            diagnostics,
            dependency_stack,
            catalog_label,
        )?;
    }
    Ok(())
}

fn ensure_feature_can_enable(
    catalog: &RuntimePluginCatalog,
    candidate: &ProjectPluginManifest,
    feature_id: &str,
) -> Result<(), String> {
    let report = catalog.feature_dependency_report(candidate, RuntimeTargetMode::EditorHost);
    if report
        .available_features
        .iter()
        .any(|available| available == feature_id)
    {
        return Ok(());
    }
    if let Some(blocked) = report
        .blocked_features
        .iter()
        .find(|blocked| blocked.feature_id == feature_id)
    {
        return Err(format!(
            "{}; enable the listed dependencies first",
            blocked.to_diagnostic()
        ));
    }
    Err(format!(
        "feature {feature_id} cannot be enabled because dependency status is unresolved"
    ))
}

fn enable_dependency_plugin(
    manifest: &mut ProjectPluginManifest,
    dependency: &PluginFeatureDependency,
    enabled_dependency_plugins: &mut Vec<String>,
    catalog_label: &str,
) -> Result<(), String> {
    let selection = project_selection_mut(manifest, &dependency.plugin_id, catalog_label)?;
    if !selection.enabled {
        selection.enabled = true;
        push_unique(enabled_dependency_plugins, dependency.plugin_id.clone());
    }
    Ok(())
}

fn enable_dependency_feature_provider(
    manifest: &mut ProjectPluginManifest,
    packages: &[PluginPackageManifest],
    dependency: &PluginFeatureDependency,
    enabled_dependency_plugins: &mut Vec<String>,
    enabled_dependency_features: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
    dependency_stack: &mut Vec<String>,
    catalog_label: &str,
) -> Result<(), String> {
    if package_provides_capability(packages, dependency, RuntimeTargetMode::EditorHost) {
        return Ok(());
    }

    let providers =
        feature_providers_for_dependency(packages, dependency, RuntimeTargetMode::EditorHost);
    match providers.as_slice() {
        [] => {
            diagnostics.push(format!(
                "dependency capability {} is not provided by a known feature under plugin {}",
                dependency.capability, dependency.plugin_id
            ));
        }
        [provider] => {
            if dependency_stack.contains(&provider.feature_id) {
                diagnostics.push(format!(
                    "dependency capability {} resolves to feature dependency cycle {} -> {}",
                    dependency.capability,
                    dependency_stack.join(" -> "),
                    provider.feature_id
                ));
                return Ok(());
            }
            let selection =
                project_selection_mut(manifest, &provider.owner_plugin_id, catalog_label)?;
            let feature_selection = selection
                .features
                .iter_mut()
                .find(|feature| feature.id == provider.feature_id)
                .ok_or_else(|| {
                    format!(
                        "feature {} is not completed under plugin {}",
                        provider.feature_id, provider.owner_plugin_id
                    )
                })?;
            if !feature_selection.enabled {
                feature_selection.enabled = true;
                push_unique(enabled_dependency_features, provider.feature_id.clone());
            }
            dependency_stack.push(provider.feature_id.clone());
            enable_feature_dependency_tree(
                manifest,
                packages,
                &provider.feature,
                enabled_dependency_plugins,
                enabled_dependency_features,
                diagnostics,
                dependency_stack,
                catalog_label,
            )?;
            dependency_stack.pop();
        }
        _ => diagnostics.push(format!(
            "dependency capability {} has multiple feature providers under plugin {}; enable the intended provider manually",
            dependency.capability, dependency.plugin_id
        )),
    }
    Ok(())
}

fn feature_manifest(
    catalog: &RuntimePluginCatalog,
    plugin_id: &str,
    feature_id: &str,
) -> Result<PluginFeatureBundleManifest, String> {
    catalog
        .package_manifests()
        .into_iter()
        .flat_map(|package| package.optional_features)
        .find(|feature| feature.owner_plugin_id == plugin_id && feature.id == feature_id)
        .ok_or_else(|| format!("feature {feature_id} is not registered under plugin {plugin_id}"))
}

fn project_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    plugin_id: &str,
    catalog_label: &str,
) -> Result<&'a ProjectPluginSelection, String> {
    manifest
        .selections
        .iter()
        .find(|selection| selection.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} is not registered in {catalog_label}"))
}

fn project_selection_mut<'a>(
    manifest: &'a mut ProjectPluginManifest,
    plugin_id: &str,
    catalog_label: &str,
) -> Result<&'a mut ProjectPluginSelection, String> {
    manifest
        .selections
        .iter_mut()
        .find(|selection| selection.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} is not registered in {catalog_label}"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FeatureProvider {
    owner_plugin_id: String,
    feature_id: String,
    feature: PluginFeatureBundleManifest,
}

fn feature_providers_for_dependency(
    packages: &[PluginPackageManifest],
    dependency: &PluginFeatureDependency,
    target: RuntimeTargetMode,
) -> Vec<FeatureProvider> {
    packages
        .iter()
        .filter(|package| package.id == dependency.plugin_id)
        .flat_map(|package| package.optional_features.iter())
        .filter(|feature| feature_provides_capability(feature, &dependency.capability, target))
        .map(|feature| FeatureProvider {
            owner_plugin_id: feature.owner_plugin_id.clone(),
            feature_id: feature.id.clone(),
            feature: feature.clone(),
        })
        .collect()
}

fn package_provides_capability(
    packages: &[PluginPackageManifest],
    dependency: &PluginFeatureDependency,
    target: RuntimeTargetMode,
) -> bool {
    packages
        .iter()
        .filter(|package| package.id == dependency.plugin_id)
        .flat_map(|package| package.modules.iter())
        .filter(|module| module_supports_target(module, target))
        .any(|module| module.capabilities.contains(&dependency.capability))
}

fn feature_provides_capability(
    feature: &PluginFeatureBundleManifest,
    capability: &str,
    target: RuntimeTargetMode,
) -> bool {
    feature
        .capabilities
        .iter()
        .any(|provided| provided == capability)
        || feature
            .modules
            .iter()
            .filter(|module| module_supports_target(module, target))
            .any(|module| {
                module
                    .capabilities
                    .iter()
                    .any(|provided| provided == capability)
            })
}

fn module_supports_target(
    module: &zircon_runtime::plugin::PluginModuleManifest,
    target: RuntimeTargetMode,
) -> bool {
    module.target_modes.is_empty() || module.target_modes.contains(&target)
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}
