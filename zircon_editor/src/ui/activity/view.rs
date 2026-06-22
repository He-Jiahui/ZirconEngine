use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

use super::slot::ActivityDrawerSlotPreference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityViewDescriptor {
    pub view_id: String,
    pub title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub supports_document_host: bool,
    pub supports_floating_window: bool,
    pub default_drawer: Option<ActivityDrawerSlotPreference>,
    pub reflection_root: UiNodePath,
}

impl ActivityViewDescriptor {
    pub fn new(
        view_id: impl Into<String>,
        title: impl Into<String>,
        icon_key: impl Into<String>,
    ) -> Self {
        let view_id = view_id.into();
        Self {
            reflection_root: UiNodePath::new(format!("editor/views/{view_id}")),
            view_id,
            title: title.into(),
            icon_key: icon_key.into(),
            multi_instance: false,
            supports_document_host: true,
            supports_floating_window: true,
            default_drawer: None,
        }
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_supports_document_host(mut self, supports: bool) -> Self {
        self.supports_document_host = supports;
        self
    }

    pub fn with_supports_floating_window(mut self, supports: bool) -> Self {
        self.supports_floating_window = supports;
        self
    }

    pub fn with_default_drawer(mut self, slot: ActivityDrawerSlotPreference) -> Self {
        self.default_drawer = Some(slot);
        self
    }

    pub fn with_reflection_root(mut self, root: UiNodePath) -> Self {
        self.reflection_root = root;
        self
    }
}
