use super::super::{
    for_each_feature_extension, for_each_optional_feature, for_each_static_plugin_manifest,
    visit_feature_dependency_rows, visit_package_dependency_ids,
};
use super::provider_tokens::assert_dependency_provider_id_token;

#[test]
fn plugin_tomls_declare_dependency_provider_ids_are_tokens() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_package_dependency_ids(table, relative_path, &mut |dependency_id| {
            assert_dependency_provider_id_token(
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "id",
                dependency_id,
            );
        });

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, _| {
                    assert_dependency_provider_id_token(
                        relative_path,
                        &format!("{feature_context} dependency `{dependency_plugin}`"),
                        "plugin_id",
                        dependency_plugin,
                    );
                },
            );
        });
        for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, _| {
                    assert_dependency_provider_id_token(
                        relative_path,
                        &format!("{feature_context} dependency `{dependency_plugin}`"),
                        "plugin_id",
                        dependency_plugin,
                    );
                },
            );
        });
    });
}
