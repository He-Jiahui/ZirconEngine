use crate::plugin::{
    PluginFeatureBundleManifest, PluginModuleKind, PluginPackageKind, PluginPackageManifest,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::{projection::NativePluginLoadProjection, NativePluginLoadReport};

impl NativePluginLoadReport {
    pub fn runtime_plugin_registration_reports(&self) -> Vec<RuntimePluginRegistrationReport> {
        self.projection().runtime_plugin_registration_reports()
    }

    pub fn runtime_plugin_feature_registration_reports(
        &self,
    ) -> Vec<RuntimePluginFeatureRegistrationReport> {
        self.projection()
            .runtime_plugin_feature_registration_reports()
    }
}

impl NativePluginLoadProjection {
    pub fn runtime_plugin_registration_reports(&self) -> Vec<RuntimePluginRegistrationReport> {
        self.package_manifests()
            .iter()
            .cloned()
            .filter(|manifest| {
                manifest.package_kind != PluginPackageKind::FeatureExtension
                    && has_runtime_module(manifest)
            })
            .map(|manifest| {
                let plugin_id = manifest.id.clone();
                let mut report = RuntimePluginRegistrationReport::from_native_package_manifest(
                    runtime_only_package_manifest(manifest),
                );
                report
                    .diagnostics
                    .extend(self.runtime_diagnostics_for_plugin(&plugin_id));
                report.diagnostics.sort();
                report.diagnostics.dedup();
                report
            })
            .collect()
    }

    pub fn runtime_plugin_feature_registration_reports(
        &self,
    ) -> Vec<RuntimePluginFeatureRegistrationReport> {
        self.package_manifests()
            .iter()
            .flat_map(|manifest| {
                let plugin_id = manifest.id.clone();
                runtime_feature_manifests(&manifest)
                    .into_iter()
                    .filter(has_runtime_feature_module)
                    .map(move |feature| {
                        let provider_package_id = if feature.owner_plugin_id == plugin_id {
                            None
                        } else {
                            Some(plugin_id.clone())
                        };
                        let mut report =
                            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                                feature,
                                provider_package_id,
                            );
                        report
                            .diagnostics
                            .extend(self.runtime_diagnostics_for_plugin(&plugin_id));
                        report.diagnostics.sort();
                        report.diagnostics.dedup();
                        report
                    })
            })
            .collect()
    }
}

fn has_runtime_module(manifest: &PluginPackageManifest) -> bool {
    manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Runtime)
}

fn has_runtime_feature_module(feature: &PluginFeatureBundleManifest) -> bool {
    feature
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Runtime)
}

fn runtime_feature_manifests(manifest: &PluginPackageManifest) -> Vec<PluginFeatureBundleManifest> {
    let mut features = manifest.optional_features.clone();
    features.extend(manifest.feature_extensions.iter().cloned());
    features
}

fn runtime_only_package_manifest(mut manifest: PluginPackageManifest) -> PluginPackageManifest {
    manifest
        .modules
        .retain(|module| module.kind == PluginModuleKind::Runtime);
    manifest
}
