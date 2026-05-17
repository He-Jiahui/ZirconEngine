use serde::{Deserialize, Serialize};

use super::{
    PrimaryWindowHandle, WindowMode, WindowPosition, WindowPresentMode, WindowResizeConstraints,
    WindowResolution, DEFAULT_WINDOW_TITLE,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowDescriptor {
    pub primary_window: Option<PrimaryWindowHandle>,
    pub title: String,
    pub present_mode: WindowPresentMode,
    pub mode: WindowMode,
    pub position: WindowPosition,
    pub resolution: WindowResolution,
    pub resize_constraints: WindowResizeConstraints,
    pub resizable: bool,
    pub decorated: bool,
    pub visible: bool,
    pub focused: bool,
}

impl WindowDescriptor {
    pub fn with_primary_window(mut self, primary_window: PrimaryWindowHandle) -> Self {
        self.primary_window = Some(primary_window);
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_present_mode(mut self, present_mode: WindowPresentMode) -> Self {
        self.present_mode = present_mode;
        self
    }

    pub fn with_mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_position(mut self, position: WindowPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_resolution(mut self, resolution: WindowResolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn with_resize_constraints(mut self, resize_constraints: WindowResizeConstraints) -> Self {
        self.resize_constraints = resize_constraints.validated();
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn with_decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            primary_window: Some(PrimaryWindowHandle::default()),
            title: DEFAULT_WINDOW_TITLE.to_string(),
            present_mode: WindowPresentMode::default(),
            mode: WindowMode::default(),
            position: WindowPosition::default(),
            resolution: WindowResolution::default(),
            resize_constraints: WindowResizeConstraints::default(),
            resizable: true,
            decorated: true,
            visible: true,
            focused: true,
        }
    }
}
