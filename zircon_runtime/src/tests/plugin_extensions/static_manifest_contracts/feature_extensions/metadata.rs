use std::collections::BTreeMap;

use super::{
    assert_known_default_packaging_strategies, assert_lowercase_dot_namespace,
    assert_package_token, assert_trimmed, assert_unique_provider_row,
    assert_unique_string_array_entries, bool_value, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value, visit_feature_extension_rows,
};

#[test]
fn plugin_tomls_declare_feature_extension_metadata() {
    let mut provider_rows = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let provider_package_id = non_empty_string_value(table, relative_path, "top-level", "id");

        visit_feature_extension_rows(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert_lowercase_dot_namespace(relative_path, feature_context, "id", feature_id);

            let owner_plugin_id =
                non_empty_string_value(feature, relative_path, feature_context, "owner_plugin_id");
            assert_package_token(
                relative_path,
                feature_context,
                "owner_plugin_id",
                owner_plugin_id,
            );
            let owner_prefix = format!("{owner_plugin_id}.");
            assert!(
                feature_id.starts_with(&owner_prefix),
                "plugin manifest {relative_path:?} {feature_context} id `{feature_id}` should stay under owner namespace `{owner_prefix}`"
            );

            let display_name =
                non_empty_string_value(feature, relative_path, feature_context, "display_name");
            assert_trimmed(relative_path, feature_context, "display_name", display_name);

            for capability in non_empty_string_array_values(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            ) {
                assert_lowercase_dot_namespace(
                    relative_path,
                    feature_context,
                    "capability",
                    capability,
                );
            }
            assert_unique_string_array_entries(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            );
            assert_unique_string_array_entries(
                feature,
                relative_path,
                feature_context,
                "default_packaging",
            );
            assert_known_default_packaging_strategies(feature, relative_path, feature_context);
            bool_value(
                feature,
                relative_path,
                feature_context,
                "enabled_by_default",
            );

            assert_unique_provider_row(
                relative_path,
                &mut provider_rows,
                feature_id,
                provider_package_id,
                feature_context,
            );
        });
    });
}
