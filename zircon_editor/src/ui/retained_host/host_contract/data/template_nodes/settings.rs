use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct TemplateSettingsCategoryData {
    pub id: SharedString,
    pub domain: SharedString,
    pub key_path: SharedString,
    pub label_path: SharedString,
    pub label: SharedString,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TemplateSettingEntryData {
    pub key: SharedString,
    pub domain: SharedString,
    pub label: SharedString,
    pub description: SharedString,
    pub category_key_path: SharedString,
    pub category_label_path: SharedString,
    pub scope: SharedString,
    pub schema: SharedString,
    pub options: Vec<SharedString>,
    pub value_text: SharedString,
    pub color_rgba: [u8; 4],
    pub value_source: SharedString,
    pub requires_restart: bool,
    pub plugin_page: bool,
}
