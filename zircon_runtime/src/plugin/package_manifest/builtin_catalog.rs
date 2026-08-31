use crate::plugin::RuntimePluginDescriptor;

use super::PluginPackageManifest;

fn exact_builtin_editor_crate_name(package_id: &str) -> String {
    let capacity = "zircon_plugin_".len() + package_id.len() + "_editor".len();
    let mut crate_name = String::with_capacity(capacity);
    crate_name.push_str("zircon_plugin_");
    crate_name.push_str(package_id);
    crate_name.push_str("_editor");
    crate_name
}

impl PluginPackageManifest {
    pub fn builtin_catalog() -> Vec<Self> {
        RuntimePluginDescriptor::builtin_catalog()
            .into_iter()
            .map(|descriptor| {
                descriptor
                    .package_manifest()
                    .with_editor_crate(exact_builtin_editor_crate_name(descriptor.package_id()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::exact_builtin_editor_crate_name;

    #[test]
    fn exact_builtin_editor_crate_names_preserve_package_identity() {
        assert_eq!(
            exact_builtin_editor_crate_name("net"),
            "zircon_plugin_net_editor"
        );
        assert_eq!(
            exact_builtin_editor_crate_name("rendering_deferred"),
            "zircon_plugin_rendering_deferred_editor"
        );
    }
}
