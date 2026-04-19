use serde::{Deserialize, Serialize};
use zircon_runtime::ui::event_ui::UiNodePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityDrawerSlotPreference {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    BottomLeft,
    BottomRight,
}

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityWindowDescriptor {
    pub window_id: String,
    pub title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub supports_document_tab: bool,
    pub supports_exclusive_page: bool,
    pub supports_floating_window: bool,
    pub reflection_root: UiNodePath,
}

impl ActivityWindowDescriptor {
    pub fn new(
        window_id: impl Into<String>,
        title: impl Into<String>,
        icon_key: impl Into<String>,
    ) -> Self {
        let window_id = window_id.into();
        Self {
            reflection_root: UiNodePath::new(format!("editor/windows/{window_id}")),
            window_id,
            title: title.into(),
            icon_key: icon_key.into(),
            multi_instance: false,
            supports_document_tab: true,
            supports_exclusive_page: true,
            supports_floating_window: true,
        }
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_supports_document_tab(mut self, supports: bool) -> Self {
        self.supports_document_tab = supports;
        self
    }

    pub fn with_supports_exclusive_page(mut self, supports: bool) -> Self {
        self.supports_exclusive_page = supports;
        self
    }

    pub fn with_supports_floating_window(mut self, supports: bool) -> Self {
        self.supports_floating_window = supports;
        self
    }

    pub fn with_reflection_root(mut self, root: UiNodePath) -> Self {
        self.reflection_root = root;
        self
    }
}
