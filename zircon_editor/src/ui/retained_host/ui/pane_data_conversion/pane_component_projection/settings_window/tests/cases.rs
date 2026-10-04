use super::*;

#[test]
fn non_settings_components_do_not_project_settings_payloads() {
    let projected = projected_settings_window_data("button", &BTreeMap::new());

    assert!(projected.categories.is_empty());
    assert!(projected.entries.is_empty());
}

#[test]
fn parent_builtin_category_shows_descendant_rows_with_resolved_values() {
    let attributes = toml::from_str::<BTreeMap<String, Value>>(
            r#"
selected_category_id = "builtin|settings.category.editor"
categories = [
    { domain = "builtin", key_path = "settings.category.editor", label = "Editor" },
]
settings = [
    { key = "editor.language.locale", label = "Language", category_key_path = " settings.category.editor/settings.category.language " },
    { key = "editor.autosave.interval_secs", label = "Autosave", category_key_path = "settings.category.editor/settings.category.autosave" },
    { key = "editor.unrelated", label = "Unrelated", category_key_path = "settings.category.editorial/settings.category.other" },
]
settings_values = [
    { key = "editor.language.locale", value_text = "English", value_source = "default" },
    { key = "editor.autosave.interval_secs", value_text = "30 s", value_source = "user" },
]
"#,
        )
        .unwrap();

    let projected = projected_settings_window_data("settings-window", &attributes);
    assert_eq!(projected.entries.len(), 2);
    assert_eq!(projected.entries[0].key.as_str(), "editor.language.locale");
    assert_eq!(projected.entries[0].value_text.as_str(), "English");
    assert_eq!(
        projected.entries[1].key.as_str(),
        "editor.autosave.interval_secs"
    );
    assert_eq!(projected.entries[1].value_text.as_str(), "30 s");
}

#[test]
fn plugin_category_keeps_exact_domain_and_path_matching() {
    let attributes = toml::from_str::<BTreeMap<String, Value>>(
            r#"
selected_category_id = "plugin:vendor.alpha|settings.category.plugin"
categories = [
    { domain = "plugin:vendor.alpha", key_path = "settings.category.plugin", label = "Vendor" },
]
settings = [
    { key = "builtin.plugin_path", label = "Built in", category_key_path = "settings.category.plugin" },
]
plugin_pages = [
    { id = "vendor.exact", localization_bundle_id = " vendor.alpha ", label = "Exact", category_key_path = " settings.category.plugin " },
    { id = "vendor.descendant", localization_bundle_id = "vendor.alpha", label = "Descendant", category_key_path = "settings.category.plugin/settings.category.child" },
    { id = "vendor.near_prefix", localization_bundle_id = "vendor.alpha", label = "Near prefix", category_key_path = "settings.category.pluginish" },
    { id = "other.exact", localization_bundle_id = "vendor.beta", label = "Other domain", category_key_path = "settings.category.plugin" },
]
"#,
        )
        .expect("plugin settings projection fixture should parse");

    let projected = projected_settings_window_data("settings-window", &attributes);

    assert_eq!(projected.entries.len(), 1);
    assert_eq!(projected.entries[0].key.as_str(), "vendor.exact");
    assert_eq!(projected.entries[0].domain.as_str(), "plugin:vendor.alpha");
    assert!(projected.entries[0].plugin_page);
}

#[test]
fn editor_kind_key_and_open_row_share_the_selected_category_projection() {
    let attributes = toml::from_str::<BTreeMap<String, Value>>(
            r#"
selected_category_id = "builtin|settings.category.editor"
settings_editor_open_key = "editor.language.locale"
settings_editor_open_kind = "enum"
settings_persistence_health_generation = 9
settings_persistence_retry_scope = "project"
settings_persistence_status_text = "Project settings: Save failed"
categories = [
    { domain = "builtin", key_path = "settings.category.editor", label = "Editor" },
]
settings = [
    { key = "editor.language.locale", label = "Language", category_key_path = "settings.category.editor", schema = "enum", options = ["en", "zh-CN"] },
]
settings_values = [
    { key = "editor.language.locale", value_text = "zh-CN", value_source = "user" },
]
"#,
        )
        .expect("settings projection fixture should parse");

    let projected = projected_settings_window_data("settings-window", &attributes);

    assert_eq!(projected.editor_open_key, "editor.language.locale");
    assert_eq!(projected.editor_open_kind, "enum");
    assert_eq!(projected.editor_open_row, 0);
    assert_eq!(projected.persistence_health_generation, 9);
    assert_eq!(projected.persistence_retry_scope, "project");
    assert_eq!(
        projected.persistence_status_text,
        "Project settings: Save failed"
    );
    assert_eq!(projected.entries.len(), 1);
    assert_eq!(projected.entries[0].value_text.as_str(), "zh-CN");
    assert_eq!(
        projected.entries[0]
            .options
            .iter()
            .map(|option| option.as_str())
            .collect::<Vec<_>>(),
        ["en", "zh-CN"]
    );
}

#[test]
fn color_channels_remain_structured_through_projection() {
    let attributes = toml::from_str::<BTreeMap<String, Value>>(
            r##"
selected_category_id = "builtin|settings.category.editor"
settings_editor_open_key = "editor.appearance.tint"
settings_editor_open_kind = "color"
categories = [
    { domain = "builtin", key_path = "settings.category.editor", label = "Editor" },
]
settings = [
    { key = "editor.appearance.tint", label = "Tint", category_key_path = "settings.category.editor", schema = "color" },
]
settings_values = [
    { key = "editor.appearance.tint", value_text = "#0C22384E", color_channels = [12, 34, 56, 78], value_source = "user" },
]
"##,
        )
        .expect("color settings projection fixture should parse");

    let projected = projected_settings_window_data("settings-window", &attributes);

    assert_eq!(projected.editor_open_kind, "color");
    assert_eq!(projected.editor_open_row, 0);
    assert_eq!(projected.entries[0].value_text.as_str(), "#0C22384E");
    assert_eq!(projected.entries[0].color_rgba, [12, 34, 56, 78]);
}

#[test]
fn editor_state_closes_when_kind_does_not_match_the_projected_schema() {
    let attributes = toml::from_str::<BTreeMap<String, Value>>(
        r#"
settings_editor_open_key = "editor.language.locale"
settings_editor_open_kind = "color"
settings = [
    { key = "editor.language.locale", label = "Language", schema = "enum", options = ["en"] },
]
"#,
    )
    .expect("mismatched settings editor fixture should parse");

    let projected = projected_settings_window_data("settings-window", &attributes);

    assert!(projected.editor_open_key.is_empty());
    assert!(projected.editor_open_kind.is_empty());
    assert_eq!(projected.editor_open_row, -1);
    assert_eq!(projected.entries.len(), 1);
}

#[path = "selected_category_refresh_profile.rs"]
mod selected_category_refresh_profile;
