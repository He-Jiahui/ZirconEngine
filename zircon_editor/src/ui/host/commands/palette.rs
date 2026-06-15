use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::UiValue;

use super::{EditorCommandContext, EditorCommandDescriptor};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCommandPaletteEntry {
    pub id: String,
    pub label: String,
    pub source: String,
    pub shortcut: String,
    pub category: String,
    pub keywords: Vec<String>,
    pub disabled: bool,
}

impl EditorCommandPaletteEntry {
    pub fn from_descriptor(
        descriptor: &EditorCommandDescriptor,
        context: EditorCommandContext,
    ) -> Self {
        Self {
            id: descriptor.id().to_string(),
            label: descriptor.label().to_string(),
            source: descriptor.category().source_tag().to_string(),
            shortcut: descriptor
                .default_chord()
                .map(ToString::to_string)
                .unwrap_or_default(),
            category: descriptor.category().as_str().to_string(),
            keywords: descriptor.keywords().to_vec(),
            disabled: !context.is_enabled(descriptor),
        }
    }

    pub fn to_ui_value(&self) -> UiValue {
        let mut values = BTreeMap::new();
        values.insert("id".to_string(), UiValue::String(self.id.clone()));
        values.insert("label".to_string(), UiValue::String(self.label.clone()));
        values.insert("source".to_string(), UiValue::String(self.source.clone()));
        values.insert(
            "shortcut".to_string(),
            UiValue::String(self.shortcut.clone()),
        );
        values.insert(
            "category".to_string(),
            UiValue::String(self.category.clone()),
        );
        values.insert(
            "keywords".to_string(),
            UiValue::Array(self.keywords.iter().cloned().map(UiValue::String).collect()),
        );
        values.insert("disabled".to_string(), UiValue::Bool(self.disabled));
        UiValue::Map(values)
    }
}
