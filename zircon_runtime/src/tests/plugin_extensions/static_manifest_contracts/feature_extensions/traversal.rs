use std::path::Path;

use super::optional_table_array;

pub(super) fn visit_feature_extension_rows(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(features) =
        optional_table_array(table, relative_path, "top-level", "feature_extensions")
    else {
        return;
    };
    assert!(
        !features.is_empty(),
        "plugin manifest {relative_path:?} feature_extensions should not be empty when declared"
    );

    for feature in features {
        let feature_id = feature
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let feature_context = format!("feature extension `{feature_id}`");
        visit(feature, &feature_context);
    }
}
