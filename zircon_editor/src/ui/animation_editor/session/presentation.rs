use super::super::AnimationEditorPanePresentation;
use super::graph::graph_node_label;
use super::parameters::parameter_value_label;
use super::state_machine::transition_label;
use super::support::fallback_title;
use super::{AnimationEditorDocument, AnimationEditorSession};

impl AnimationEditorSession {
    pub fn display_name(&self) -> String {
        match &self.document {
            AnimationEditorDocument::Sequence(document) => document
                .asset
                .name
                .clone()
                .unwrap_or_else(|| fallback_title(&self.asset_path)),
            AnimationEditorDocument::Graph(asset) => asset
                .name
                .clone()
                .unwrap_or_else(|| fallback_title(&self.asset_path)),
            AnimationEditorDocument::StateMachine(asset) => asset
                .name
                .clone()
                .unwrap_or_else(|| fallback_title(&self.asset_path)),
        }
    }

    pub fn pane_presentation(&self) -> AnimationEditorPanePresentation {
        match &self.document {
            AnimationEditorDocument::Sequence(document) => AnimationEditorPanePresentation {
                mode: "sequence".to_string(),
                asset_path: self.asset_path.clone(),
                status: format!(
                    "{} tracks • frame {}",
                    document.asset.track_paths().len(),
                    document.current_frame
                ),
                selection_summary: document
                    .selected_span
                    .as_ref()
                    .map(|(track_path, start_frame, end_frame)| {
                        format!("{track_path} [{start_frame}..{end_frame}]")
                    })
                    .unwrap_or_default(),
                current_frame: document.current_frame,
                timeline_start_frame: document.timeline_start_frame,
                timeline_end_frame: document.timeline_end_frame,
                playback_label: format!(
                    "{} • loop={} • speed={:.2}",
                    if document.playing {
                        "Playing"
                    } else {
                        "Paused"
                    },
                    document.looping,
                    document.speed
                ),
                track_items: document
                    .asset
                    .track_paths()
                    .into_iter()
                    .map(|track_path| track_path.to_string())
                    .collect(),
                parameter_items: Vec::new(),
                node_items: Vec::new(),
                state_items: Vec::new(),
                transition_items: Vec::new(),
            },
            AnimationEditorDocument::Graph(asset) => AnimationEditorPanePresentation {
                mode: "graph".to_string(),
                asset_path: self.asset_path.clone(),
                status: format!(
                    "{} parameters • {} nodes",
                    asset.parameters.len(),
                    asset.nodes.len()
                ),
                selection_summary: String::new(),
                current_frame: 0,
                timeline_start_frame: 0,
                timeline_end_frame: 0,
                playback_label: "Graph Authoring".to_string(),
                track_items: Vec::new(),
                parameter_items: asset
                    .parameters
                    .iter()
                    .map(|parameter| {
                        format!(
                            "{} = {}",
                            parameter.name,
                            parameter_value_label(&parameter.default_value)
                        )
                    })
                    .collect(),
                node_items: asset.nodes.iter().map(graph_node_label).collect(),
                state_items: Vec::new(),
                transition_items: Vec::new(),
            },
            AnimationEditorDocument::StateMachine(asset) => AnimationEditorPanePresentation {
                mode: "state_machine".to_string(),
                asset_path: self.asset_path.clone(),
                status: format!(
                    "entry {} • {} states • {} transitions",
                    asset.entry_state,
                    asset.states.len(),
                    asset.transitions.len()
                ),
                selection_summary: asset.entry_state.clone(),
                current_frame: 0,
                timeline_start_frame: 0,
                timeline_end_frame: 0,
                playback_label: "State Machine Authoring".to_string(),
                track_items: Vec::new(),
                parameter_items: Vec::new(),
                node_items: Vec::new(),
                state_items: asset
                    .states
                    .iter()
                    .map(|state| state.name.clone())
                    .collect(),
                transition_items: asset.transitions.iter().map(transition_label).collect(),
            },
        }
    }
}
