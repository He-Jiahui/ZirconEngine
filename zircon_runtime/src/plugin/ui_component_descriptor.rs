use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentDescriptor {
    pub component_id: String,
    pub plugin_id: String,
    pub ui_document: String,
}

impl UiComponentDescriptor {
    pub fn new(
        component_id: impl Into<String>,
        plugin_id: impl Into<String>,
        ui_document: impl Into<String>,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            plugin_id: plugin_id.into(),
            ui_document: ui_document.into(),
        }
    }
}
