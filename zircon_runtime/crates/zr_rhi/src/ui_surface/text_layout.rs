//! Observations of the exact buffers admitted by the GPU text renderer; physical pixels.
use super::UiSurfaceRect;
use serde::Serialize;
#[derive(Clone, Debug, Default, Serialize)]
pub struct UiSurfaceTextLayoutSnapshot {
    pub presented_frame_count: u64,
    pub projection_size: (u32, u32),
    pub damage: Option<UiSurfaceRect>,
    pub prepared_this_present: bool,
    pub retained_cache_copy_bytes: u64,
    pub draw_list_generation: Option<u64>,
    pub runs: Vec<UiSurfaceTextLayoutRun>,
}
impl UiSurfaceTextLayoutSnapshot {
    /// A prepared snapshot has zero here until an actual successful surface submission stamps it.
    pub fn is_from_present(&self, presented_frame_count: u64) -> bool {
        presented_frame_count != 0 && self.presented_frame_count == presented_frame_count
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct UiSurfaceTextLayoutRun {
    pub command_index: usize,
    pub text: String,
    pub clip: UiSurfaceRect,
    pub lines: Vec<UiSurfaceTextLine>,
    pub faces: Vec<UiSurfaceTextFace>,
}
#[derive(Clone, Debug, Serialize)]
pub struct UiSurfaceTextLine {
    pub original_line_text: String,
    pub line_index: usize,
    pub byte_range: Option<(usize, usize)>,
    pub frame: UiSurfaceRect,
    pub baseline_y: f32,
    /// Actual fallback faces used by this shaped line, joined to run.faces by font_id.
    pub font_ids: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct UiSurfaceTextFace {
    pub font_id: String,
    pub sha256: String,
    pub face_index: u32,
    pub families: Vec<String>,
    pub post_script_name: String,
    pub weight: u16,
    pub style: String,
}

#[cfg(test)]
#[path = "tests/text_layout.rs"]
mod tests;
