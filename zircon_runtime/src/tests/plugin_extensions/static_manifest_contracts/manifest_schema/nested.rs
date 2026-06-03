use super::super::{for_each_static_plugin_manifest, optional_table_array};
use super::assertions::{
    assert_known_component_fields, assert_known_feature_bundle_fields, assert_known_row_fields,
    assert_known_table_fields,
};
use super::field_sets::{
    KNOWN_ASSET_IMPORTER_FIELDS, KNOWN_CAPABILITY_STATUS_FIELDS, KNOWN_DEPENDENCY_FIELDS,
    KNOWN_EVENT_CATALOG_FIELDS, KNOWN_EVENT_FIELDS, KNOWN_MODULE_FIELDS, KNOWN_OPTION_FIELDS,
    KNOWN_UI_COMPONENT_FIELDS,
};

#[test]
fn plugin_tomls_declare_known_nested_fields() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_row_fields(
            table,
            relative_path,
            "asset_importers",
            "asset importer",
            &KNOWN_ASSET_IMPORTER_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "capability_statuses",
            "capability status",
            &KNOWN_CAPABILITY_STATUS_FIELDS,
        );
        assert_known_component_fields(table, relative_path);
        assert_known_row_fields(
            table,
            relative_path,
            "dependencies",
            "top-level dependency",
            &KNOWN_DEPENDENCY_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "modules",
            "package module",
            &KNOWN_MODULE_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "options",
            "option",
            &KNOWN_OPTION_FIELDS,
        );

        assert_known_feature_bundle_fields(
            table,
            relative_path,
            "optional_features",
            "optional feature",
        );
        assert_known_feature_bundle_fields(
            table,
            relative_path,
            "feature_extensions",
            "feature extension",
        );
        assert_known_row_fields(
            table,
            relative_path,
            "ui_components",
            "ui component",
            &KNOWN_UI_COMPONENT_FIELDS,
        );

        for catalog in optional_table_array(table, relative_path, "top-level", "event_catalogs")
            .unwrap_or_default()
        {
            let namespace = catalog
                .get("namespace")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let catalog_context = format!("event catalog `{namespace}`");
            assert_known_table_fields(
                relative_path,
                &catalog_context,
                catalog,
                &KNOWN_EVENT_CATALOG_FIELDS,
            );
            assert_known_row_fields(
                catalog,
                relative_path,
                "events",
                &format!("{catalog_context} event"),
                &KNOWN_EVENT_FIELDS,
            );
        }
    });
}
