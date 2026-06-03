use std::path::{Component, Path};

use super::super::{
    for_each_static_plugin_manifest, non_empty_string_array_values, non_empty_string_value,
    plugins_workspace_root,
};
use super::traversal::capability_status_array;

#[test]
fn plugin_tomls_declare_bevy_references_resolve_under_dev_bevy() {
    let plugins_root = plugins_workspace_root();
    let repo_root = plugins_root
        .parent()
        .expect("zircon_plugins workspace should be under the repository root");
    let bevy_root = Path::new("dev").join("bevy");

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(statuses) = capability_status_array(table, relative_path) else {
            return;
        };

        for status in statuses {
            let status = status.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} capability status should be a table")
            });
            let capability =
                non_empty_string_value(status, relative_path, "capability status", "capability");
            let context = format!("capability status `{capability}`");
            let Some(_) = status.get("bevy_references") else {
                continue;
            };

            for reference in
                non_empty_string_array_values(status, relative_path, &context, "bevy_references")
            {
                assert_bevy_reference_path(relative_path, &context, reference, &bevy_root);
                let reference_path = repo_root.join(reference);
                assert!(
                    reference_path.is_file(),
                    "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should resolve to an existing file"
                );
            }
        }
    });
}

fn assert_bevy_reference_path(
    relative_path: &Path,
    context: &str,
    reference: &str,
    bevy_root: &Path,
) {
    let reference_path = Path::new(reference);
    assert!(
        reference_path.is_relative(),
        "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should be repository-relative"
    );
    assert!(
        !reference_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::CurDir | Component::RootDir | Component::Prefix(_)
            )
        }),
        "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should not contain root, current, parent, or drive-prefix path components"
    );
    assert!(
        reference_path.starts_with(bevy_root),
        "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should stay under dev/bevy"
    );
}
