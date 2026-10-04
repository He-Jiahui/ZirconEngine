use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

const ACTIVITY_WINDOW_ROOT_PREFIX: &str = "editor/windows/";

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
            reflection_root: activity_window_reflection_root(&window_id),
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

fn activity_window_reflection_root(window_id: &str) -> UiNodePath {
    let mut path = String::with_capacity(ACTIVITY_WINDOW_ROOT_PREFIX.len() + window_id.len());
    path.push_str(ACTIVITY_WINDOW_ROOT_PREFIX);
    path.push_str(window_id);
    UiNodePath::new(path)
}

#[cfg(test)]
#[path = "tests/window_optimization_batch_fn_tests.rs"]
mod optimization_batch_fn_tests;
