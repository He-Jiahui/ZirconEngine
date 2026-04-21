#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationEditorPanePresentation {
    pub mode: String,
    pub asset_path: String,
    pub status: String,
    pub selection_summary: String,
    pub current_frame: u32,
    pub timeline_start_frame: u32,
    pub timeline_end_frame: u32,
    pub playback_label: String,
    pub track_items: Vec<String>,
    pub parameter_items: Vec<String>,
    pub node_items: Vec<String>,
    pub state_items: Vec<String>,
    pub transition_items: Vec<String>,
}
