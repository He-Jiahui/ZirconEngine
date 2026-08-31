use std::collections::HashSet;

use super::super::super::package_validation::RuntimePluginPackageValidationProjection;
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_system_anchors(
    _package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    let plugin_system_rows = extensions.plugin_systems();
    let runtime_system_rows = extensions.plugin_runtime_systems();
    let registered_system_capacity = plugin_system_rows
        .size_hint()
        .1
        .unwrap_or(0)
        .saturating_add(runtime_system_rows.size_hint().1.unwrap_or(0));
    let mut registered_systems = HashSet::with_capacity(registered_system_capacity);
    for (owner, system) in plugin_system_rows {
        if let Some(module_name) = extensions.plugin_module_name(owner) {
            registered_systems.insert((module_name, system.id.as_str()));
        }
    }
    for (owner, system) in runtime_system_rows {
        if let Some(module_name) = extensions.plugin_module_name(owner) {
            registered_systems.insert((module_name, system.id.as_str()));
        }
    }

    for (module_name, anchor) in projection.runtime_system_anchors() {
        if !registered_systems.contains(&(module_name, anchor)) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` declares system anchor `{anchor}` but did not register a matching runtime system"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn preallocated_system_anchor_index_preserves_borrowed_registration_contract() {
        let source = include_str!("system_anchors.rs");
        let capacity_constructor = ["HashSet::with_", "capacity"].concat();
        let capacity_hint = [".size_", "hint()"].concat();
        let unbounded_collect = ["collect::<HashSet", "<_>>()"].concat();

        assert_eq!(source.matches(&capacity_constructor).count(), 1);
        assert_eq!(source.matches(&capacity_hint).count(), 2);
        assert!(!source.contains(&unbounded_collect));
    }
}
