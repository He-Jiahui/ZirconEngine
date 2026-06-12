use std::collections::HashSet;

use crate::plugin::ProjectPluginSelection;

use super::super::ids::RuntimePluginId;

pub(in crate::builtin::runtime_modules) fn linked_plugin_is_available(
    selection: &ProjectPluginSelection,
    runtime_id: RuntimePluginId,
    linked_plugin_ids: &HashSet<String>,
) -> bool {
    linked_plugin_ids.contains(&selection.id) || linked_plugin_ids.contains(runtime_id.key())
}

pub(in crate::builtin::runtime_modules) fn builtin_runtime_domain_is_available(
    id: RuntimePluginId,
) -> bool {
    let _ = id;
    false
}

pub(in crate::builtin::runtime_modules) fn builtin_runtime_domain_message(id: &str) -> String {
    format!("runtime plugin {id} is provided by the built-in runtime domain")
}
