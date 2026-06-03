use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency};

use super::rows::ParticlesFeatureRow;

pub(super) fn particles_feature(row: &ParticlesFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = format!("particles.{}", row.id_suffix);
    let mut manifest = PluginFeatureBundleManifest::new(feature_id, row.display_name, "particles")
        .with_dependency(PluginFeatureDependency::primary(
            "particles",
            "runtime.plugin.particles",
        ))
        .with_capability(row.capability);
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}
