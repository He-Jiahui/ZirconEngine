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
    /// Identity of the runtime collection actually installed in the renderer.
    pub font_collection_id: Option<u64>,
    pub font_collection_generation: Option<u64>,
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
    pub requested: Option<UiSurfaceTextRequest>,
    pub lines: Vec<UiSurfaceTextLine>,
    pub faces: Vec<UiSurfaceTextFace>,
}
/// Requested command attributes, distinct from the actual selected glyph faces.
#[derive(Clone, Debug, Serialize)]
pub struct UiSurfaceTextRequest {
    pub font_family: Option<String>,
    pub font_weight: u16,
    pub font_style: String,
    pub font_size: f32,
    pub line_height: f32,
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
    #[serde(default)]
    pub glyph_count: usize,
    pub sha256: String,
    pub face_index: u32,
    pub families: Vec<String>,
    pub post_script_name: String,
    pub weight: u16,
    pub style: String,
    /// Actual file source retained by the font database, when available.
    /// Binary-only faces keep this absent; names never imply a file identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_sha256: Option<String>,
}

#[cfg(test)]
#[path = "tests/text_layout.rs"]
mod tests;
