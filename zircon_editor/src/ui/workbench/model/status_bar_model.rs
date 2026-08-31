use crate::scene::viewport::{GridMode, SceneViewportChromeSettings};
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, StatusTaskProgressSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBarModel {
    pub primary_text: String,
    pub secondary_text: Option<String>,
    pub viewport_label: String,
    pub error_text: String,
    pub warning_text: String,
    pub message_text: String,
    pub grid_text: String,
    pub snap_text: String,
    pub snap_enabled: bool,
    pub zoom_text: String,
    pub task_progress: Option<StatusTaskProgressSnapshot>,
}

impl Default for StatusBarModel {
    fn default() -> Self {
        Self {
            primary_text: String::new(),
            secondary_text: None,
            viewport_label: String::new(),
            error_text: "No Errors".to_string(),
            warning_text: "0 Warnings".to_string(),
            message_text: "0 Messages".to_string(),
            grid_text: "Grid: Off".to_string(),
            snap_text: "Snap: Off".to_string(),
            snap_enabled: false,
            zoom_text: "100%".to_string(),
            task_progress: None,
        }
    }
}

impl StatusBarModel {
    pub fn from_chrome(chrome: &EditorChromeSnapshot) -> Self {
        let task_progress = chrome.status_task_progress.clone();
        Self {
            primary_text: non_empty_label(&chrome.status_line, "Ready"),
            secondary_text: chrome
                .inspector
                .as_ref()
                .map(|inspector| format!("Selection {}", inspector.name)),
            viewport_label: format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            error_text: "No Errors".to_string(),
            warning_text: "0 Warnings".to_string(),
            message_text: if task_progress.is_some() {
                "1 Message".to_string()
            } else {
                "0 Messages".to_string()
            },
            grid_text: status_grid_text(&chrome.scene_viewport_settings),
            snap_text: status_snap_text(&chrome.scene_viewport_settings),
            snap_enabled: matches!(
                chrome.scene_viewport_settings.grid_mode,
                GridMode::VisibleAndSnap
            ),
            zoom_text: "100%".to_string(),
            task_progress,
        }
    }
}

fn status_grid_text(settings: &SceneViewportChromeSettings) -> String {
    match settings.grid_mode {
        GridMode::Hidden => "Grid: Off".to_string(),
        GridMode::VisibleNoSnap | GridMode::VisibleAndSnap => {
            format!("Grid: {}", format_step(settings.translate_step))
        }
    }
}

fn status_snap_text(settings: &SceneViewportChromeSettings) -> String {
    match settings.grid_mode {
        GridMode::VisibleAndSnap => "Snap: On".to_string(),
        GridMode::Hidden | GridMode::VisibleNoSnap => "Snap: Off".to_string(),
    }
}

fn format_step(value: zircon_runtime_interface::math::Real) -> String {
    let mut text = format!("{value:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    format!("{text} m")
}

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
