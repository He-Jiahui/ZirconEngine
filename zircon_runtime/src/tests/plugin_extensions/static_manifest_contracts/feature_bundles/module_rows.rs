use std::path::Path;

use super::helpers::for_each_feature_bundle;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn for_each_module_row(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    visit_module_rows(table.get("modules"), relative_path, "package", visit);

    visit_feature_bundle_module_rows(
        table,
        relative_path,
        "optional_features",
        "optional feature",
        visit,
    );
    visit_feature_bundle_module_rows(
        table,
        relative_path,
        "feature_extensions",
        "feature extension",
        visit,
    );
}

fn visit_feature_bundle_module_rows(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
    row_label: &str,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    for_each_feature_bundle(
        table,
        relative_path,
        field_name,
        row_label,
        &mut |feature_table, feature_context| {
            visit_module_rows(
                feature_table.get("modules"),
                relative_path,
                feature_context,
                visit,
            );
        },
    );
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_module_rows(
    modules: Option<&toml::Value>,
    relative_path: &Path,
    module_context: &str,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(modules) = modules else {
        return;
    };
    let modules = modules.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} {module_context} modules should be an array")
    });

    for module in modules {
        let module = module.as_table().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {module_context} module row should be a table"
            )
        });
        visit(module, module_context);
    }
}
