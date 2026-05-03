use crate::plugin::RuntimePluginDescriptor;

use super::PluginPackageManifest;

impl PluginPackageManifest {
    pub fn builtin_catalog() -> Vec<Self> {
        RuntimePluginDescriptor::builtin_catalog()
            .into_iter()
            .map(|descriptor| {
                descriptor
                    .package_manifest()
                    .with_editor_crate(format!("zircon_plugin_{}_editor", descriptor.package_id))
            })
            .collect()
    }
}
