use serde::{Deserialize, Serialize};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor as RuntimeUiComponentDescriptor,
    UiDefaultNodeTemplate, UiHostCapability, UiPaletteMetadata, UiPropSchema, UiSlotSchema,
    UiValue, UiValueKind,
};

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

    pub fn to_runtime_component_descriptor(&self) -> RuntimeUiComponentDescriptor {
        RuntimeUiComponentDescriptor::new(
            self.component_id.clone(),
            self.display_name(),
            UiComponentCategory::Container,
            "plugin-ui-component",
        )
        .default_prop("plugin_id", UiValue::String(self.plugin_id.clone()))
        .default_prop("ui_document", UiValue::String(self.ui_document.clone()))
        .with_prop(UiPropSchema::new("plugin_id", UiValueKind::String).required(true))
        .with_prop(UiPropSchema::new("ui_document", UiValueKind::String).required(true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .requires_host_capability(UiHostCapability::Editor)
        .requires_host_capability(UiHostCapability::Runtime)
        .default_node_template(UiDefaultNodeTemplate::native(self.component_id.as_str()))
        .palette(UiPaletteMetadata::new(
            self.display_name(),
            UiComponentCategory::Container,
            self.component_id.clone(),
            UiDefaultNodeTemplate::native(self.component_id.as_str()),
        ))
    }

    fn display_name(&self) -> String {
        self.component_id
            .rsplit('.')
            .next()
            .unwrap_or(&self.component_id)
            .to_string()
    }
}
