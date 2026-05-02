use serde::{Deserialize, Serialize};

use crate::ui::component::UiComponentCategory;

use super::UiDefaultNodeTemplate;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPaletteMetadata {
    pub display_name: String,
    pub category: UiComponentCategory,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub sort_key: String,
    #[serde(default)]
    pub default_node: UiDefaultNodeTemplate,
}

impl UiPaletteMetadata {
    pub fn new(
        display_name: impl Into<String>,
        category: UiComponentCategory,
        sort_key: impl Into<String>,
        default_node: UiDefaultNodeTemplate,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            category,
            icon: None,
            sort_key: sort_key.into(),
            default_node,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}
