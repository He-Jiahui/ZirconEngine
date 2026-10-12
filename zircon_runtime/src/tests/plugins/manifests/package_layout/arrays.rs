use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn string_array_values<'a>(
    value: &'a toml::Value,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Vec<&'a str> {
    value
        .as_array()
        .unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {context} `{field_name}` should be an array")
        })
        .iter()
        .map(|entry| {
            entry.as_str().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {context} `{field_name}` entries should be strings"
                )
            })
        })
        .inspect(|entry| {
            assert!(
                !entry.is_empty(),
                "plugin manifest {relative_path:?} {context} `{field_name}` entries should not be empty"
            );
        })
        .collect()
}

pub(super) fn assert_unique_entries(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    entries: &[&str],
) {
    let mut seen = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(previous_index) = seen.insert((*entry).to_string(), index) {
            panic!(
                "plugin manifest {relative_path:?} {context} `{field_name}` entry `{entry}` should be unique; first declared at index {previous_index}, repeated at index {index}"
            );
        }
    }
}
