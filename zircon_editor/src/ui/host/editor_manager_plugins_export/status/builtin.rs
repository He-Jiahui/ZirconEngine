use std::collections::{HashMap, HashSet};

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ProjectPluginFeatureSelection,
    core::framework::project::ProjectPluginSelection, plugin::PluginFeatureBundleManifest,
    plugin::PluginFeatureDependency, plugin::PluginModuleKind, plugin::PluginPackageManifest,
    plugin::RuntimePluginFeatureBlock,
};

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::{module_crate, runtime_capabilities_for_package};
use super::super::reports::{
    EditorPluginFeatureDependencyStatus, EditorPluginFeatureStatus, EditorPluginStatus,
    EditorPluginStatusReport,
};

impl EditorManager {
    pub fn plugin_status_report(&self, manifest: &ProjectManifest) -> EditorPluginStatusReport {
        let runtime_catalog = self.runtime_plugin_catalog();
        let editor_catalog = self.editor_plugin_catalog();
        let packages = editor_catalog.package_manifests();
        let mut diagnostics = Vec::new();
        diagnostics.extend(runtime_catalog.diagnostics().iter().cloned());
        diagnostics.extend(editor_catalog.diagnostics().iter().cloned());
        let status_target = RuntimeTargetMode::EditorHost;
        let completed_plugins =
            runtime_catalog.complete_project_manifest(&manifest.plugins, status_target);
        let feature_report =
            runtime_catalog.feature_dependency_report(&manifest.plugins, status_target);
        diagnostics.extend(feature_report.diagnostics.iter().cloned());
        let available_feature_ids = feature_report
            .available_features
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let enabled_plugins = completed_plugins
            .enabled_for_target(status_target)
            .map(|selection| selection.id.clone())
            .collect::<HashSet<_>>();
        let available_capabilities = available_capabilities_for_feature_status(
            &packages,
            &enabled_plugins,
            &available_feature_ids,
            status_target,
        );
        let blocked_feature_diagnostics =
            blocked_feature_diagnostic_map(&feature_report.blocked_features);

        let mut plugins = packages
            .iter()
            .map(|package| {
                let project_selection = manifest
                    .plugins
                    .selections
                    .iter()
                    .find(|selection| selection.id == package.id);
                let completed_project_selection = completed_plugins
                    .selections
                    .iter()
                    .find(|selection| selection.id == package.id);
                let catalog_selection = runtime_catalog.project_selection_for_package(&package.id);
                let runtime_crate = project_selection
                    .and_then(|selection| selection.runtime_crate.clone())
                    .or_else(|| {
                        catalog_selection
                            .as_ref()
                            .and_then(|selection| selection.runtime_crate.clone())
                    })
                    .or_else(|| module_crate(package, PluginModuleKind::Runtime));
                let editor_crate = project_selection
                    .and_then(|selection| selection.editor_crate.clone())
                    .or_else(|| module_crate(package, PluginModuleKind::Editor));
                let runtime_capabilities = runtime_capabilities_for_package(package);
                let editor_capabilities = editor_catalog
                    .capabilities_for_package(&package.id)
                    .to_vec();
                let mut plugin_diagnostics = Vec::new();
                if package
                    .modules
                    .iter()
                    .any(|module| module.kind == PluginModuleKind::Runtime)
                    && runtime_crate.is_none()
                {
                    plugin_diagnostics.push("runtime crate is not declared".to_string());
                }
                if editor_crate.is_none() && !editor_capabilities.is_empty() {
                    plugin_diagnostics.push(
                        "editor capabilities are declared without an editor crate".to_string(),
                    );
                }
                EditorPluginStatus {
                    plugin_id: package.id.clone(),
                    display_name: package.display_name.clone(),
                    package_source: "builtin".to_string(),
                    load_state: "catalog".to_string(),
                    enabled: project_selection
                        .map(|selection| selection.enabled)
                        .or_else(|| completed_project_selection.map(|selection| selection.enabled))
                        .unwrap_or(false),
                    required: project_selection
                        .map(|selection| selection.required)
                        .or_else(|| {
                            catalog_selection
                                .as_ref()
                                .map(|selection| selection.required)
                        })
                        .unwrap_or(false),
                    target_modes: project_selection
                        .map(|selection| selection.target_modes.clone())
                        .filter(|modes| !modes.is_empty())
                        .or_else(|| {
                            catalog_selection
                                .as_ref()
                                .map(|selection| selection.target_modes.clone())
                                .filter(|modes| !modes.is_empty())
                        })
                        .unwrap_or_else(|| target_modes_for_package(package)),
                    packaging: project_selection
                        .map(|selection| selection.packaging)
                        .or_else(|| {
                            catalog_selection
                                .as_ref()
                                .map(|selection| selection.packaging)
                        })
                        .unwrap_or_else(|| default_packaging_for_builtin_package(package)),
                    runtime_crate,
                    editor_crate,
                    runtime_capabilities,
                    editor_capabilities,
                    optional_features: optional_feature_statuses(
                        package,
                        completed_project_selection,
                        &enabled_plugins,
                        &available_capabilities,
                        &available_feature_ids,
                        &blocked_feature_diagnostics,
                        status_target,
                    ),
                    diagnostics: plugin_diagnostics,
                }
            })
            .collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));

        EditorPluginStatusReport {
            plugins,
            diagnostics,
        }
    }
}

fn target_modes_for_package(package: &PluginPackageManifest) -> Vec<RuntimeTargetMode> {
    let mut modes = Vec::new();
    for mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

fn default_packaging_for_builtin_package(
    package: &PluginPackageManifest,
) -> ExportPackagingStrategy {
    package
        .default_packaging
        .first()
        .copied()
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
}

pub(super) fn optional_feature_statuses(
    package: &PluginPackageManifest,
    owner_selection: Option<&ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
    available_feature_ids: &HashSet<String>,
    blocked_feature_diagnostics: &HashMap<String, Vec<String>>,
    target: RuntimeTargetMode,
) -> Vec<EditorPluginFeatureStatus> {
    package
        .optional_features
        .iter()
        .map(|feature| {
            let catalog_selection = feature_selection_from_feature_manifest(feature);
            let selection = owner_selection
                .and_then(|owner| {
                    owner
                        .features
                        .iter()
                        .find(|selection| selection.id == feature.id)
                })
                .unwrap_or(&catalog_selection);
            let dependencies = feature
                .dependencies
                .iter()
                .map(|dependency| {
                    feature_dependency_status(dependency, enabled_plugins, available_capabilities)
                })
                .collect::<Vec<_>>();
            let available = feature_is_available_for_status(
                feature,
                selection,
                &dependencies,
                available_feature_ids,
                target,
            );
            let mut diagnostics =
                feature_status_diagnostics(feature, selection, &dependencies, target);
            if selection.enabled {
                diagnostics.extend(
                    blocked_feature_diagnostics
                        .get(&feature.id)
                        .into_iter()
                        .flat_map(|messages| messages.iter().cloned()),
                );
            }
            diagnostics.sort();
            diagnostics.dedup();

            EditorPluginFeatureStatus {
                id: feature.id.clone(),
                display_name: feature.display_name.clone(),
                owner_plugin_id: feature.owner_plugin_id.clone(),
                enabled: selection.enabled,
                required: selection.required,
                available,
                target_modes: selection_target_modes(selection, feature),
                packaging: selection.packaging,
                runtime_crate: selection
                    .runtime_crate
                    .clone()
                    .or_else(|| feature_module_crate(feature, PluginModuleKind::Runtime)),
                editor_crate: selection
                    .editor_crate
                    .clone()
                    .or_else(|| feature_module_crate(feature, PluginModuleKind::Editor)),
                provided_capabilities: feature_capabilities_for_target(feature, target),
                dependencies,
                diagnostics,
            }
        })
        .collect()
}

fn feature_dependency_status(
    dependency: &PluginFeatureDependency,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
) -> EditorPluginFeatureDependencyStatus {
    EditorPluginFeatureDependencyStatus {
        plugin_id: dependency.plugin_id.clone(),
        capability: dependency.capability.clone(),
        primary: dependency.primary,
        plugin_enabled: enabled_plugins.contains(&dependency.plugin_id),
        capability_available: available_capabilities.contains(&dependency.capability),
    }
}

fn feature_is_available_for_status(
    feature: &PluginFeatureBundleManifest,
    selection: &ProjectPluginFeatureSelection,
    dependencies: &[EditorPluginFeatureDependencyStatus],
    available_feature_ids: &HashSet<String>,
    target: RuntimeTargetMode,
) -> bool {
    available_feature_ids.contains(&feature.id)
        || (owner_dependency_is_valid(feature)
            && feature_manifest_supports_target(feature, target)
            && selection.supports_target(target)
            && dependencies
                .iter()
                .all(|dependency| dependency.plugin_enabled && dependency.capability_available))
}

fn feature_status_diagnostics(
    feature: &PluginFeatureBundleManifest,
    selection: &ProjectPluginFeatureSelection,
    dependencies: &[EditorPluginFeatureDependencyStatus],
    target: RuntimeTargetMode,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !owner_dependency_is_valid(feature) {
        diagnostics.push(
            "owner dependency is missing, not marked primary, or not the only primary dependency"
                .to_string(),
        );
    }
    if !feature_manifest_supports_target(feature, target) || !selection.supports_target(target) {
        diagnostics.push("target mode is not supported".to_string());
    }
    let mut missing_plugins = Vec::new();
    let mut missing_capabilities = Vec::new();
    for dependency in dependencies {
        if !dependency.plugin_enabled {
            push_unique(&mut missing_plugins, dependency.plugin_id.clone());
        }
        if !dependency.capability_available {
            push_unique(&mut missing_capabilities, dependency.capability.clone());
        }
    }
    if !missing_plugins.is_empty() {
        diagnostics.push(format!("missing plugins: {}", missing_plugins.join(", ")));
    }
    if !missing_capabilities.is_empty() {
        diagnostics.push(format!(
            "missing capabilities: {}",
            missing_capabilities.join(", ")
        ));
    }
    diagnostics
}

pub(super) fn available_capabilities_for_feature_status(
    packages: &[PluginPackageManifest],
    enabled_plugins: &HashSet<String>,
    available_feature_ids: &HashSet<String>,
    target: RuntimeTargetMode,
) -> HashSet<String> {
    let mut capabilities = HashSet::new();
    for package in packages {
        if enabled_plugins.contains(&package.id) {
            for module in &package.modules {
                if module_supports_target(module, target) {
                    capabilities.extend(module.capabilities.iter().cloned());
                }
            }
        }
        for feature in &package.optional_features {
            if available_feature_ids.contains(&feature.id) {
                capabilities.extend(feature_capabilities_for_target(feature, target));
            }
        }
    }
    capabilities
}

pub(super) fn blocked_feature_diagnostic_map(
    blocked_features: &[RuntimePluginFeatureBlock],
) -> HashMap<String, Vec<String>> {
    let mut diagnostics = HashMap::<String, Vec<String>>::new();
    for blocked in blocked_features {
        diagnostics
            .entry(blocked.feature_id.clone())
            .or_default()
            .push(blocked.to_diagnostic());
    }
    diagnostics
}

fn feature_selection_from_feature_manifest(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection = ProjectPluginFeatureSelection::new(feature.id.clone())
        .enabled(feature.enabled_by_default)
        .with_packaging(default_packaging_for_feature(feature))
        .with_target_modes(feature_target_modes(feature));
    if let Some(crate_name) = feature_module_crate(feature, PluginModuleKind::Runtime) {
        selection = selection.with_runtime_crate(crate_name);
    }
    if let Some(crate_name) = feature_module_crate(feature, PluginModuleKind::Editor) {
        selection = selection.with_editor_crate(crate_name);
    }
    selection
}

fn default_packaging_for_feature(feature: &PluginFeatureBundleManifest) -> ExportPackagingStrategy {
    if feature
        .default_packaging
        .contains(&ExportPackagingStrategy::LibraryEmbed)
    {
        ExportPackagingStrategy::LibraryEmbed
    } else {
        feature
            .default_packaging
            .first()
            .copied()
            .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
    }
}

fn selection_target_modes(
    selection: &ProjectPluginFeatureSelection,
    feature: &PluginFeatureBundleManifest,
) -> Vec<RuntimeTargetMode> {
    if selection.target_modes.is_empty() {
        feature_target_modes(feature)
    } else {
        selection.target_modes.clone()
    }
}

fn feature_target_modes(feature: &PluginFeatureBundleManifest) -> Vec<RuntimeTargetMode> {
    let mut modes = Vec::new();
    for mode in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

fn feature_module_crate(
    feature: &PluginFeatureBundleManifest,
    kind: PluginModuleKind,
) -> Option<String> {
    feature
        .modules
        .iter()
        .find(|module| module.kind == kind)
        .map(|module| module.crate_name.clone())
}

fn feature_manifest_supports_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> bool {
    let mut runtime_modules = feature
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime);
    let Some(first_runtime_module) = runtime_modules.next() else {
        return true;
    };
    module_supports_target(first_runtime_module, target)
        || runtime_modules.any(|module| module_supports_target(module, target))
}

fn feature_capabilities_for_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> Vec<String> {
    let mut capabilities = feature.capabilities.clone();
    for module in &feature.modules {
        if module_supports_target(module, target) {
            for capability in &module.capabilities {
                push_unique(&mut capabilities, capability.clone());
            }
        }
    }
    capabilities
}

fn module_supports_target(
    module: &zircon_runtime::plugin::PluginModuleManifest,
    target: RuntimeTargetMode,
) -> bool {
    module.target_modes.is_empty() || module.target_modes.contains(&target)
}

fn owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
    let mut primary_dependencies = feature
        .dependencies
        .iter()
        .filter(|dependency| dependency.primary);
    let Some(owner_dependency) = primary_dependencies.next() else {
        return false;
    };
    owner_dependency.plugin_id == feature.owner_plugin_id && primary_dependencies.next().is_none()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::InitLevel;
    use zircon_runtime::{
        plugin::PluginFeatureBundleManifest, plugin::PluginFeatureDependency,
        plugin::PluginModuleKind, plugin::PluginModuleManifest,
    };

    use super::{
        feature_is_available_for_status, feature_manifest_supports_target,
        feature_status_diagnostics, owner_dependency_is_valid, EditorPluginFeatureDependencyStatus,
        ProjectPluginFeatureSelection,
    };

    fn benchmark_runtime_module(index: usize) -> PluginModuleManifest {
        PluginModuleManifest {
            name: format!("benchmark.runtime.{index}"),
            description: String::new(),
            kind: PluginModuleKind::Runtime,
            crate_name: format!("benchmark_runtime_{index}"),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: vec![RuntimeTargetMode::ServerRuntime],
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
            event_consumers: Vec::new(),
        }
    }

    fn legacy_feature_manifest_supports_target(
        feature: &PluginFeatureBundleManifest,
        target: RuntimeTargetMode,
    ) -> bool {
        let runtime_modules = feature
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
            .collect::<Vec<_>>();
        if runtime_modules.is_empty() {
            return true;
        }
        runtime_modules
            .iter()
            .any(|module| module.target_modes.is_empty() || module.target_modes.contains(&target))
    }

    fn legacy_owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
        let primary_dependencies = feature
            .dependencies
            .iter()
            .filter(|dependency| dependency.primary)
            .collect::<Vec<_>>();
        primary_dependencies.len() == 1
            && primary_dependencies[0].plugin_id == feature.owner_plugin_id
    }

    #[test]
    fn feature_status_rejects_secondary_primary_dependency() {
        let feature =
            PluginFeatureBundleManifest::new("sound.invalid_extra_primary", "Invalid", "sound")
                .with_dependency(PluginFeatureDependency::primary(
                    "sound",
                    "runtime.plugin.sound",
                ))
                .with_dependency(PluginFeatureDependency::primary(
                    "animation",
                    "runtime.feature.animation.timeline_event_track",
                ));
        let selection = ProjectPluginFeatureSelection::new("sound.invalid_extra_primary");
        let dependencies = vec![
            EditorPluginFeatureDependencyStatus {
                plugin_id: "sound".to_string(),
                capability: "runtime.plugin.sound".to_string(),
                primary: true,
                plugin_enabled: true,
                capability_available: true,
            },
            EditorPluginFeatureDependencyStatus {
                plugin_id: "animation".to_string(),
                capability: "runtime.feature.animation.timeline_event_track".to_string(),
                primary: true,
                plugin_enabled: true,
                capability_available: true,
            },
        ];
        let available_feature_ids = std::collections::HashSet::new();

        assert!(!feature_is_available_for_status(
            &feature,
            &selection,
            &dependencies,
            &available_feature_ids,
            RuntimeTargetMode::EditorHost,
        ));
        assert!(feature_status_diagnostics(
            &feature,
            &selection,
            &dependencies,
            RuntimeTargetMode::EditorHost,
        )
        .iter()
        .any(|diagnostic| diagnostic.contains("not the only primary dependency")));
    }

    #[test]
    fn optimization_batch_en_feature_checks_stream_without_temporary_vectors() {
        let source = include_str!("builtin.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("builtin plugin status implementation");
        let target_check = production
            .split("fn feature_manifest_supports_target(")
            .nth(1)
            .expect("target support check")
            .split("fn feature_capabilities_for_target(")
            .next()
            .expect("target support function body");
        let owner_check = production
            .split("fn owner_dependency_is_valid(")
            .nth(1)
            .expect("owner dependency check");

        assert!(!target_check.contains("collect::<Vec<_>>()"));
        assert!(!owner_check.contains("collect::<Vec<_>>()"));

        let mut feature =
            PluginFeatureBundleManifest::new("benchmark.feature", "Benchmark Feature", "benchmark")
                .with_dependency(PluginFeatureDependency::primary(
                    "benchmark",
                    "runtime.plugin.benchmark",
                ));
        feature.modules.push(benchmark_runtime_module(0));
        assert_eq!(
            feature_manifest_supports_target(&feature, RuntimeTargetMode::ClientRuntime),
            legacy_feature_manifest_supports_target(&feature, RuntimeTargetMode::ClientRuntime)
        );
        assert_eq!(
            owner_dependency_is_valid(&feature),
            legacy_owner_dependency_is_valid(&feature)
        );
    }

    #[test]
    #[ignore = "release-only streaming feature checks benchmark"]
    fn optimization_batch_en_streaming_feature_checks_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const CHECKS_PER_SAMPLE: usize = 8_192;
        const MODULE_COUNT: usize = 128;

        fn measure_legacy(feature: &PluginFeatureBundleManifest) -> u128 {
            let started = Instant::now();
            let mut matched = 0_usize;
            for _ in 0..CHECKS_PER_SAMPLE {
                matched += usize::from(black_box(
                    legacy_feature_manifest_supports_target(
                        black_box(feature),
                        RuntimeTargetMode::ClientRuntime,
                    ) && legacy_owner_dependency_is_valid(black_box(feature)),
                ));
            }
            black_box(matched);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(feature: &PluginFeatureBundleManifest) -> u128 {
            let started = Instant::now();
            let mut matched = 0_usize;
            for _ in 0..CHECKS_PER_SAMPLE {
                matched += usize::from(black_box(
                    feature_manifest_supports_target(
                        black_box(feature),
                        RuntimeTargetMode::ClientRuntime,
                    ) && owner_dependency_is_valid(black_box(feature)),
                ));
            }
            black_box(matched);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let mut feature =
            PluginFeatureBundleManifest::new("benchmark.feature", "Benchmark Feature", "benchmark")
                .with_dependency(PluginFeatureDependency::primary(
                    "benchmark",
                    "runtime.plugin.benchmark",
                ));
        feature.modules = (0..MODULE_COUNT)
            .map(benchmark_runtime_module)
            .collect::<Vec<_>>();
        feature
            .modules
            .last_mut()
            .expect("benchmark runtime module")
            .target_modes = vec![RuntimeTargetMode::ClientRuntime];

        assert!(feature_manifest_supports_target(
            &feature,
            RuntimeTargetMode::ClientRuntime
        ));
        assert!(owner_dependency_is_valid(&feature));
        for _ in 0..4 {
            black_box(measure_legacy(&feature));
            black_box(measure_optimized(&feature));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&feature));
                optimized_samples.push(measure_optimized(&feature));
            } else {
                optimized_samples.push(measure_optimized(&feature));
                legacy_samples.push(measure_legacy(&feature));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR376_STREAMING_FEATURE_CHECKS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             checks_per_sample={CHECKS_PER_SAMPLE} module_count={MODULE_COUNT} \
             pair_order=alternating_legacy_even legacy_vector_allocations_per_check=2 \
             optimized_vector_allocations_per_check=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
            "streaming feature checks must reduce P95 by at least 20%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
