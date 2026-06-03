use std::path::Path;

use super::super::{non_empty_string_array_values, non_empty_string_value};

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_option_required_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, &str),
) {
    let Some(options) = table.get("options") else {
        return;
    };
    let options = options
        .as_array()
        .unwrap_or_else(|| panic!("plugin manifest {relative_path:?} options should be an array"));

    for option in options {
        let option = option.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} option should be a table")
        });
        let key = non_empty_string_value(option, relative_path, "plugin option", "key");
        if option.get("required_capability").is_some() {
            let capability = non_empty_string_value(
                option,
                relative_path,
                &format!("plugin option `{key}`"),
                "required_capability",
            );
            visit(key, capability);
        }
    }
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_asset_importer_required_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, &str),
) {
    let Some(importers) = table.get("asset_importers") else {
        return;
    };
    let importers = importers.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} asset_importers should be an array")
    });

    for importer in importers {
        let importer = importer.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} asset importer should be a table")
        });
        let importer_id = non_empty_string_value(importer, relative_path, "asset importer", "id");
        let importer_context = format!("asset importer `{importer_id}`");
        for capability in non_empty_string_array_values(
            importer,
            relative_path,
            &importer_context,
            "required_capabilities",
        ) {
            visit(importer_id, capability);
        }
    }
}
