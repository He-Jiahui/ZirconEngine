use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

use super::rows::NetFeatureRow;

pub(super) fn net_feature(row: &NetFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = format!("net.{}", row.id_suffix);
    let mut manifest =
        PluginFeatureBundleManifest::new(feature_id.clone(), row.display_name, "net")
            .with_dependency(PluginFeatureDependency::primary(
                "net",
                "runtime.plugin.net",
            ))
            .with_capability(row.capability)
            .with_runtime_module(
                PluginModuleManifest::runtime(format!("{feature_id}.runtime"), row.runtime_crate)
                    .with_target_modes(row.target_modes.iter().copied())
                    .with_capabilities([row.capability.to_string()]),
            );
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}
