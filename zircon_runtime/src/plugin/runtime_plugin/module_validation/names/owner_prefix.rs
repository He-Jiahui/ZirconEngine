use crate::plugin::PluginModuleManifest;

fn has_module_owner_prefix(module_name: &str, owner_id: &str) -> bool {
    module_name
        .strip_prefix(owner_id)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

pub(super) fn validate_runtime_plugin_module_name_owner_prefix(
    manifest_label: &str,
    owner_label: &str,
    owner_id: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    if !has_module_owner_prefix(&module.name, owner_id) {
        diagnostics.push(format!(
            "{manifest_label} module name `{}` must be prefixed by {owner_label} `{owner_id}`",
            module.name
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::has_module_owner_prefix;

    #[test]
    fn borrowed_owner_prefix_preserves_boundary_semantics() {
        assert!(has_module_owner_prefix("weather.runtime", "weather"));
        assert!(!has_module_owner_prefix("weather2.runtime", "weather"));
        assert!(!has_module_owner_prefix("weather", "weather"));
        assert!(has_module_owner_prefix(".runtime", ""));
    }
}
