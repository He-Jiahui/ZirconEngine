use std::path::Path;

use super::super::non_empty_string_value;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_feature_dependency_rows(
    feature: &toml::Table,
    relative_path: &Path,
    feature_context: &str,
    visit: &mut impl FnMut(&str, &str),
) {
    let Some(dependencies) = feature.get("dependencies") else {
        return;
    };
    let dependencies = dependencies.as_array().unwrap_or_else(|| {
        panic!(
            "plugin manifest {relative_path:?} {feature_context} dependencies should be an array"
        )
    });

    for dependency in dependencies {
        let dependency = dependency.as_table().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {feature_context} dependency should be a table"
            )
        });
        let dependency_plugin =
            non_empty_string_value(dependency, relative_path, feature_context, "plugin_id");
        let capability =
            non_empty_string_value(dependency, relative_path, feature_context, "capability");
        visit(dependency_plugin, capability);
    }
}
