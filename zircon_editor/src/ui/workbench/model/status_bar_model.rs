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
    pub task_progress: Option<StatusTaskProgressSnapshot>,
}

impl Default for StatusBarModel {
    fn default() -> Self {
        Self {
            primary_text: String::new(),
            secondary_text: None,
            viewport_label: String::new(),
            error_text: "0 Errors".to_string(),
            warning_text: "0 Warnings".to_string(),
            message_text: "0 Messages".to_string(),
            grid_text: "Grid: Off".to_string(),
            snap_text: "T:off R:off S:off".to_string(),
            snap_enabled: false,
            task_progress: None,
        }
    }
}

impl StatusBarModel {
    pub fn from_chrome(chrome: &EditorChromeSnapshot) -> Self {
        let task_progress = chrome.status_task_progress.clone();
        let (grid_text, snap_text) = Self::viewport_chrome_text(&chrome.scene_viewport_settings);
        Self {
            primary_text: non_empty_label(&chrome.status_line, "Ready"),
            secondary_text: chrome
                .inspector
                .as_ref()
                .map(|inspector| format!("Selection {}", inspector.name)),
            viewport_label: format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            error_text: count_text(chrome.status_signals.logs.error, "Error"),
            warning_text: count_text(chrome.status_signals.logs.warning, "Warning"),
            message_text: count_text(chrome.status_signals.active_notifications as u64, "Message"),
            grid_text,
            snap_text,
            // The adjacent button toggles translate snapping, independently of grid visibility.
            snap_enabled: chrome.scene_viewport_settings.translate_snap_enabled,
            task_progress,
        }
    }

    pub(crate) fn viewport_chrome_text(settings: &SceneViewportChromeSettings) -> (String, String) {
        (status_grid_text(settings), status_snap_text(settings))
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
    let translate = if settings.translate_snap_enabled {
        format_step(settings.translate_step)
    } else {
        "off".to_string()
    };
    let rotate = if settings.rotate_snap_enabled {
        format!("{}°", format_number(settings.rotate_step_deg))
    } else {
        "off".to_string()
    };
    let scale = if settings.scale_snap_enabled {
        format_number(settings.scale_step)
    } else {
        "off".to_string()
    };
    format!("T:{translate} R:{rotate} S:{scale}")
}

fn format_step(value: zircon_runtime_interface::math::Real) -> String {
    format!("{} m", format_number(value))
}

fn format_number(value: zircon_runtime_interface::math::Real) -> String {
    let mut text = format!("{value:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn count_text(count: u64, singular: &str) -> String {
    format!("{count} {singular}{}", if count == 1 { "" } else { "s" })
}

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
#[path = "tests/status_bar_viewport_chrome_text.rs"]
mod status_bar_viewport_chrome_text_tests;
