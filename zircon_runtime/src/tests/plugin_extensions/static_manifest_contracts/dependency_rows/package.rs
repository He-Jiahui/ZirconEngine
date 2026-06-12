use std::path::Path;

use super::super::non_empty_string_value;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_package_dependency_rows(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, &str),
) {
    let Some(dependencies) = table.get("dependencies") else {
        return;
    };
    let dependencies = dependencies.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} dependencies should be an array")
    });

    for dependency in dependencies {
        let dependency = dependency.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} dependency should be a table")
        });
        let dependency_id =
            non_empty_string_value(dependency, relative_path, "top-level dependency", "id");
        let Some(capability) = dependency.get("capability") else {
            continue;
        };
        let capability = capability.as_str().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} top-level dependency `{dependency_id}` capability should be a string"
            )
        });
        visit(dependency_id, capability);
    }
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_package_dependency_ids(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str),
) {
    let Some(dependencies) = table.get("dependencies") else {
        return;
    };
    let dependencies = dependencies.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} dependencies should be an array")
    });

    for dependency in dependencies {
        let dependency = dependency.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} dependency should be a table")
        });
        let dependency_id =
            non_empty_string_value(dependency, relative_path, "top-level dependency", "id");
        visit(dependency_id);
    }
}
