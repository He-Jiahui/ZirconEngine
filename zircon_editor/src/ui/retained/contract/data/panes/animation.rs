use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct AnimationEditorPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub mode: SharedString,
    pub asset_path: SharedString,
    pub status: SharedString,
    pub selection: SharedString,
    pub current_frame: i32,
    pub timeline_start_frame: i32,
    pub timeline_end_frame: i32,
    pub playback_label: SharedString,
    pub track_items: ModelRc<SharedString>,
    pub parameter_items: ModelRc<SharedString>,
    pub node_items: ModelRc<SharedString>,
    pub state_items: ModelRc<SharedString>,
    pub transition_items: ModelRc<SharedString>,
}
