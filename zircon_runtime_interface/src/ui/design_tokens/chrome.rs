use serde::{Deserialize, Serialize};

/// Logical Workbench chrome dimensions. Physical scaling occurs only at render assembly.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorChromeTokens {
    pub top_bar_height: f32,
    pub host_bar_height: f32,
    pub status_bar_height: f32,
    pub panel_header_height: f32,
    pub document_header_height: f32,
    pub viewport_toolbar_height: f32,
    pub activity_rail_width: f32,
    pub separator_thickness: f32,
    pub splitter_hit_size: f32,
}

impl Default for EditorChromeTokens {
    fn default() -> Self {
        Self::workbench_dense()
    }
}

impl EditorChromeTokens {
    pub fn workbench_dense() -> Self {
        Self {
            top_bar_height: 25.0,
            host_bar_height: 32.0,
            status_bar_height: 24.0,
            panel_header_height: 25.0,
            document_header_height: 31.0,
            viewport_toolbar_height: 28.0,
            activity_rail_width: 34.0,
            separator_thickness: 1.0,
            splitter_hit_size: 8.0,
        }
    }

    pub(super) fn cascade_entries(&self) -> [(&'static str, f32); 9] {
        [
            ("editor.chrome.top_bar.height", self.top_bar_height),
            ("editor.chrome.host_bar.height", self.host_bar_height),
            ("editor.chrome.status_bar.height", self.status_bar_height),
            (
                "editor.chrome.panel_header.height",
                self.panel_header_height,
            ),
            (
                "editor.chrome.document_header.height",
                self.document_header_height,
            ),
            (
                "editor.chrome.viewport_toolbar.height",
                self.viewport_toolbar_height,
            ),
            (
                "editor.chrome.activity_rail.width",
                self.activity_rail_width,
            ),
            (
                "editor.chrome.separator.thickness",
                self.separator_thickness,
            ),
            ("editor.chrome.splitter.hit_size", self.splitter_hit_size),
        ]
    }
}
