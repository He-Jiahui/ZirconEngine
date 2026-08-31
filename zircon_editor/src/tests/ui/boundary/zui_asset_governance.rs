use std::collections::BTreeMap;
use std::path::PathBuf;

mod asset_identity;
mod asset_imports;
mod asset_placement;
mod support;

mod class;
mod component;
mod component_root;
mod control;
mod event;
mod graph;
mod interaction_policy;
mod layout;
mod layout_axis;
mod metadata;
mod node_component;
mod node_metadata;
mod props;
mod slot;
mod slot_schema;
mod style;
mod style_selector_policy;
mod visual_policy;
mod workbench_atomic_density;
mod workbench_composites;
mod workbench_modules;
mod workbench_overlay_density;
mod workbench_primitives;
mod workbench_shell;
mod workbench_status_semantics;

use self::metadata::string_metadata_offender;

fn duplicate_entries<'a>(values: impl IntoIterator<Item = &'a String>) -> Vec<String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() {
            *counts.entry(value).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(value, count)| (count > 1).then(|| value.to_string()))
        .collect()
}

fn import_entry_metadata_offenders(
    path: &PathBuf,
    import_section: &str,
    imports: &[String],
) -> (usize, Vec<String>) {
    let mut offenders = Vec::new();

    for (import_index, import) in imports.iter().enumerate() {
        if let Some(invalid_import) = string_metadata_offender(import, "import entry") {
            offenders.push(format!(
                "{} {import_section} #{} declares {invalid_import}",
                path.display(),
                import_index + 1
            ));
        }
    }

    (imports.len(), offenders)
}

fn push_asset_header_metadata_offenders(
    path: &PathBuf,
    asset_id: &str,
    display_name: &str,
    offenders: &mut Vec<String>,
) {
    if let Some(invalid_asset_id) = string_metadata_offender(asset_id, "asset id") {
        offenders.push(format!("{} declares {invalid_asset_id}", path.display()));
    }
    if let Some(invalid_display_name) = string_metadata_offender(display_name, "asset display_name")
    {
        offenders.push(format!(
            "{} declares {invalid_display_name}",
            path.display()
        ));
    }
}
