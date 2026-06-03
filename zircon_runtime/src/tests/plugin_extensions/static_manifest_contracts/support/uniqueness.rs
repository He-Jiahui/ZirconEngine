use std::collections::BTreeMap;
use std::path::Path;

use super::non_empty_string_array_values;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_unique_static_identity(
    identities: &mut BTreeMap<String, String>,
    identity: &str,
    context: String,
) {
    if let Some(previous_context) = identities.insert(identity.to_string(), context.clone()) {
        panic!(
            "static plugin identity `{identity}` should be globally unique; first declared by {previous_context}, repeated by {context}"
        );
    }
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_unique_dependency_row(
    dependency_rows: &mut BTreeMap<String, String>,
    plugin_id: &str,
    capability: &str,
    context: String,
) {
    let dependency_key = format!("{plugin_id}:{capability}");
    if let Some(previous_context) = dependency_rows.insert(dependency_key.clone(), context.clone())
    {
        panic!(
            "dependency row `{dependency_key}` should be unique; first declared by {previous_context}, repeated by {context}"
        );
    }
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_unique_string_array_entries(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    let mut entries = BTreeMap::new();
    for (index, entry) in non_empty_string_array_values(table, relative_path, context, field_name)
        .into_iter()
        .enumerate()
    {
        if let Some(previous_index) = entries.insert(entry.to_string(), index) {
            panic!(
                "plugin manifest {relative_path:?} {context} `{field_name}` entry `{entry}` should be unique; first declared at index {previous_index}, repeated at index {index}"
            );
        }
    }
}
