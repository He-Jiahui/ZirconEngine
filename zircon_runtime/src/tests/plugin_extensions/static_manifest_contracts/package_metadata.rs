use super::{for_each_optional_feature, for_each_static_plugin_manifest, non_empty_string_value};

#[test]
fn plugin_tomls_declare_public_display_strings_are_trimmed() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["display_name", "description"] {
            let value = non_empty_string_value(table, relative_path, "top-level", field_name);
            assert_trimmed(relative_path, "top-level", field_name, value);
        }

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let display_name =
                non_empty_string_value(feature, relative_path, feature_context, "display_name");
            assert_trimmed(relative_path, feature_context, "display_name", display_name);
        });

        let Some(statuses) = table
            .get("capability_statuses")
            .and_then(toml::Value::as_array)
        else {
            return;
        };

        for status in statuses {
            let status = status.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} capability status should be a table")
            });
            let capability =
                non_empty_string_value(status, relative_path, "capability status", "capability");
            let context = format!("capability status `{capability}`");
            if status.get("note").is_some() {
                let note = non_empty_string_value(status, relative_path, &context, "note");
                assert_trimmed(relative_path, &context, "note", note);
            }
        }
    });
}

fn assert_trimmed(relative_path: &std::path::Path, context: &str, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {context} `{field_name}` should not have leading or trailing whitespace"
    );
}
