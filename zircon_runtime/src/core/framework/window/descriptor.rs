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

    pub fn without_primary_window(mut self) -> Self {
        self.primary_window = None;
        self.visible = false;
        self.focused = false;
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

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let physical_size = self.resolution.physical_size();
        let logical_size = self.resolution.logical_size();
        let constraints = self.resize_constraints.validated();
        vec![
            format!(
                "window.primary_window={}",
                self.primary_window
                    .map(|handle| handle.raw().to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("window.title={}", self.title),
            format!("window.present_mode={:?}", self.present_mode),
            format!("window.mode={:?}", self.mode),
            format!("window.position={:?}", self.position),
            format!(
                "window.physical_size={}x{}",
                physical_size.x, physical_size.y
            ),
            format!(
                "window.logical_size={}x{}",
                format_window_axis(logical_size[0]),
                format_window_axis(logical_size[1])
            ),
            format!("window.scale_factor={}", self.resolution.scale_factor()),
            format!(
                "window.scale_factor_override={}",
                self.resolution
                    .scale_factor_override()
                    .map(|scale_factor| scale_factor.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "window.resize_constraints={}x{}..{}x{}",
                format_window_axis(constraints.min_width),
                format_window_axis(constraints.min_height),
                format_window_axis(constraints.max_width),
                format_window_axis(constraints.max_height)
            ),
            format!("window.resizable={}", self.resizable),
            format!("window.decorated={}", self.decorated),
            format!("window.visible={}", self.visible),
            format!("window.focused={}", self.focused),
        ]
    }

    pub fn format_diagnostics(&self) -> String {
        self.diagnostic_lines().join("\n")
    }
}

fn format_window_axis(axis: f32) -> String {
    if axis.fract() == 0.0 {
        format!("{axis:.0}")
    } else {
        axis.to_string()
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
