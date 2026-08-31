use std::collections::BTreeMap;

use toml::Value;

use crate::ui::retained_host as host_contract;

mod parse;

use parse::{bool_value, rgba_value, string_array, string_value, table_array};

const TITLE: &str = "title";
const SELECTED_CATEGORY_ID: &str = "selected_category_id";
const SETTINGS_EDITOR_OPEN_KEY: &str = "settings_editor_open_key";
const SETTINGS_EDITOR_OPEN_KIND: &str = "settings_editor_open_kind";
const SETTINGS_PERSISTENCE_HEALTH_GENERATION: &str = "settings_persistence_health_generation";
const SETTINGS_PERSISTENCE_RETRY_SCOPE: &str = "settings_persistence_retry_scope";
const SETTINGS_PERSISTENCE_STATUS_TEXT: &str = "settings_persistence_status_text";
const CATEGORIES: &str = "categories";
const SETTINGS: &str = "settings";
const SETTINGS_VALUES: &str = "settings_values";
const PLUGIN_PAGES: &str = "plugin_pages";
const BUILTIN_DOMAIN: &str = "builtin";

#[derive(Default)]
pub(in crate::ui::retained_host::ui) struct ProjectedSettingsWindowData {
    pub title: String,
    pub selected_category_id: String,
    pub editor_open_key: String,
    pub editor_open_kind: String,
    pub editor_open_row: i32,
    pub persistence_health_generation: u64,
    pub persistence_retry_scope: String,
    pub persistence_status_text: String,
    pub categories: Vec<host_contract::TemplateSettingsCategoryData>,
    pub entries: Vec<host_contract::TemplateSettingEntryData>,
}

#[derive(Clone, Default)]
struct ProjectedResolvedSettingValue {
    text: String,
    source: String,
    color_rgba: [u8; 4],
}

pub(in crate::ui::retained_host::ui) fn projected_settings_window_data(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> ProjectedSettingsWindowData {
    if component_role != "settings-window" {
        return ProjectedSettingsWindowData::default();
    }

    let mut categories = table_array(attributes, CATEGORIES)
        .filter_map(project_category)
        .collect::<Vec<_>>();
    let selected_category_id = attributes
        .get(SELECTED_CATEGORY_ID)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|selected| {
            categories
                .iter()
                .any(|category| category.id.as_str() == *selected)
        })
        .map(str::to_owned)
        .or_else(|| categories.first().map(|category| category.id.to_string()))
        .unwrap_or_default();
    for category in &mut categories {
        category.selected = category.id.as_str() == selected_category_id;
    }

    let selected_category = categories.iter().find(|category| category.selected);
    let setting_values = table_array(attributes, SETTINGS_VALUES)
        .filter_map(project_resolved_value)
        .collect::<BTreeMap<_, _>>();
    let mut entries = table_array(attributes, SETTINGS)
        .filter_map(|values| project_builtin_setting(values, &setting_values))
        .filter(|entry| entry_matches_category(entry, selected_category))
        .collect::<Vec<_>>();
    entries.extend(
        table_array(attributes, PLUGIN_PAGES)
            .filter_map(project_plugin_page)
            .filter(|entry| entry_matches_category(entry, selected_category)),
    );

    let requested_editor_key = attributes
        .get(SETTINGS_EDITOR_OPEN_KEY)
        .and_then(Value::as_str)
        .unwrap_or_default();
    let requested_editor_kind = attributes
        .get(SETTINGS_EDITOR_OPEN_KIND)
        .and_then(Value::as_str)
        .filter(|kind| matches!(*kind, "enum" | "color"))
        .unwrap_or_default();
    let editor_open_row = entries
        .iter()
        .position(|entry| {
            entry.key.as_str() == requested_editor_key
                && entry.schema.as_str() == requested_editor_kind
        })
        .and_then(|row| i32::try_from(row).ok())
        .unwrap_or(-1);
    let (editor_open_key, editor_open_kind) = if editor_open_row >= 0 {
        (
            requested_editor_key.to_owned(),
            requested_editor_kind.to_owned(),
        )
    } else {
        (String::new(), String::new())
    };
    ProjectedSettingsWindowData {
        title: attributes
            .get(TITLE)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        selected_category_id,
        editor_open_key,
        editor_open_kind,
        editor_open_row,
        persistence_health_generation: attributes
            .get(SETTINGS_PERSISTENCE_HEALTH_GENERATION)
            .and_then(Value::as_integer)
            .and_then(|generation| u64::try_from(generation).ok())
            .unwrap_or_default(),
        persistence_retry_scope: attributes
            .get(SETTINGS_PERSISTENCE_RETRY_SCOPE)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        persistence_status_text: attributes
            .get(SETTINGS_PERSISTENCE_STATUS_TEXT)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        categories,
        entries,
    }
}

fn project_category(
    values: &toml::map::Map<String, Value>,
) -> Option<host_contract::TemplateSettingsCategoryData> {
    let domain = string_value(values, "domain");
    let key_path = string_value(values, "key_path");
    let label = string_value(values, "label");
    if domain.is_empty() || key_path.is_empty() || label.is_empty() {
        return None;
    }
    Some(host_contract::TemplateSettingsCategoryData {
        id: category_id(&domain, &key_path).into(),
        domain: domain.into(),
        key_path: key_path.into(),
        label_path: string_value(values, "label_path").into(),
        label: label.into(),
        selected: false,
    })
}

fn project_builtin_setting(
    values: &toml::map::Map<String, Value>,
    resolved_values: &BTreeMap<String, ProjectedResolvedSettingValue>,
) -> Option<host_contract::TemplateSettingEntryData> {
    let key = string_value(values, "key");
    let label = string_value(values, "label");
    if key.is_empty() || label.is_empty() {
        return None;
    }
    let resolved = resolved_values.get(&key).cloned().unwrap_or_default();
    Some(host_contract::TemplateSettingEntryData {
        key: key.into(),
        domain: BUILTIN_DOMAIN.into(),
        label: label.into(),
        description: string_value(values, "description").into(),
        category_key_path: string_value(values, "category_key_path").into(),
        category_label_path: string_value(values, "category_label_path").into(),
        scope: string_value(values, "scope").into(),
        schema: string_value(values, "schema").into(),
        options: string_array(values, "options"),
        value_text: resolved.text.into(),
        color_rgba: resolved.color_rgba,
        value_source: resolved.source.into(),
        requires_restart: bool_value(values, "requires_restart"),
        plugin_page: false,
    })
}

fn project_plugin_page(
    values: &toml::map::Map<String, Value>,
) -> Option<host_contract::TemplateSettingEntryData> {
    let key = string_value(values, "id");
    let label = string_value(values, "label");
    let bundle_id = string_value(values, "localization_bundle_id");
    if key.is_empty() || label.is_empty() || bundle_id.is_empty() {
        return None;
    }
    Some(host_contract::TemplateSettingEntryData {
        key: key.into(),
        domain: format!("plugin:{bundle_id}").into(),
        label: label.into(),
        description: string_value(values, "description").into(),
        category_key_path: string_value(values, "category_key_path").into(),
        category_label_path: string_value(values, "category_label_path").into(),
        scope: String::new().into(),
        schema: "plugin_page".into(),
        options: Vec::new(),
        value_text: String::new().into(),
        color_rgba: [0; 4],
        value_source: String::new().into(),
        requires_restart: false,
        plugin_page: true,
    })
}

fn project_resolved_value(
    values: &toml::map::Map<String, Value>,
) -> Option<(String, ProjectedResolvedSettingValue)> {
    let key = string_value(values, "key");
    if key.is_empty() {
        return None;
    }
    Some((
        key,
        ProjectedResolvedSettingValue {
            text: values
                .get("value_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            source: string_value(values, "value_source"),
            color_rgba: rgba_value(values, "color_channels").unwrap_or_default(),
        },
    ))
}

fn entry_matches_category(
    entry: &host_contract::TemplateSettingEntryData,
    category: Option<&host_contract::TemplateSettingsCategoryData>,
) -> bool {
    let Some(category) = category else {
        return true;
    };
    entry.domain == category.domain && entry.category_key_path == category.key_path
}

fn category_id(domain: &str, key_path: &str) -> String {
    format!("{domain}|{key_path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_settings_components_do_not_project_settings_payloads() {
        let projected = projected_settings_window_data("button", &BTreeMap::new());

        assert!(projected.categories.is_empty());
        assert!(projected.entries.is_empty());
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
    }
}
