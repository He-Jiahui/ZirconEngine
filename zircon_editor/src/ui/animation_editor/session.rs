use std::path::Path;

use zircon_runtime::asset::assets::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationConditionOperatorAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationGraphParameterAsset, AnimationInterpolationAsset, AnimationSequenceAsset,
    AnimationSequenceBindingAsset, AnimationSequenceTrackAsset, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{AnimationParameterValue, AnimationTrackPath};

use super::AnimationEditorPanePresentation;

const DEFAULT_SEQUENCE_FRAMES_PER_SECOND: f32 = 30.0;
const DEFAULT_STATE_MACHINE_TRANSITION_FPS: f32 = 30.0;

#[derive(Clone, Debug)]
pub struct AnimationEditorSessionError(pub String);

impl std::fmt::Display for AnimationEditorSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AnimationEditorSessionError {}

#[derive(Clone, Debug)]
struct AnimationSequenceDocument {
    asset: AnimationSequenceAsset,
    current_frame: u32,
    timeline_start_frame: u32,
    timeline_end_frame: u32,
    selected_span: Option<(AnimationTrackPath, u32, u32)>,
    playing: bool,
    looping: bool,
    speed: f32,
}

#[derive(Clone, Debug)]
enum AnimationEditorDocument {
    Sequence(AnimationSequenceDocument),
    Graph(AnimationGraphAsset),
    StateMachine(AnimationStateMachineAsset),
}

#[derive(Clone, Debug)]
pub struct AnimationEditorSession {
    asset_path: String,
    document: AnimationEditorDocument,
    dirty: bool,
}

impl AnimationEditorSession {
    pub fn from_path(path: &Path) -> Result<Self, AnimationEditorSessionError> {
        let bytes =
            std::fs::read(path).map_err(|error| AnimationEditorSessionError(error.to_string()))?;
        let asset_path = path.to_string_lossy().into_owned();
        let lowered = asset_path.to_ascii_lowercase();
        if lowered.ends_with(".sequence.zranim") {
            let asset =
                AnimationSequenceAsset::from_bytes(&bytes).map_err(AnimationEditorSessionError)?;
            let timeline_end_frame =
                duration_frames(asset.duration_seconds, asset.frames_per_second);
            return Ok(Self {
                asset_path,
                document: AnimationEditorDocument::Sequence(AnimationSequenceDocument {
                    asset,
                    current_frame: 0,
                    timeline_start_frame: 0,
                    timeline_end_frame,
                    selected_span: None,
                    playing: false,
                    looping: false,
                    speed: 1.0,
                }),
                dirty: false,
            });
        }
        if lowered.ends_with(".graph.zranim") {
            let asset =
                AnimationGraphAsset::from_bytes(&bytes).map_err(AnimationEditorSessionError)?;
            return Ok(Self {
                asset_path,
                document: AnimationEditorDocument::Graph(asset),
                dirty: false,
            });
        }
        if lowered.ends_with(".state_machine.zranim") {
            let asset = AnimationStateMachineAsset::from_bytes(&bytes)
                .map_err(AnimationEditorSessionError)?;
            return Ok(Self {
                asset_path,
                document: AnimationEditorDocument::StateMachine(asset),
                dirty: false,
            });
        }
        Err(AnimationEditorSessionError(format!(
            "unsupported animation editor asset {}",
            path.display()
        )))
    }

    pub fn asset_path(&self) -> &str {
        &self.asset_path
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn save(&mut self) -> Result<(), AnimationEditorSessionError> {
        let bytes = self.document_bytes()?;
        std::fs::write(&self.asset_path, bytes)
            .map_err(|error| AnimationEditorSessionError(error.to_string()))?;
        self.dirty = false;
        Ok(())
    }

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

    pub fn add_key(&mut self, track_path: &AnimationTrackPath, frame: u32) -> Result<bool, String> {
        let time_seconds = frame_to_seconds(frame, self.sequence_frames_per_second());
        let track = self
            .sequence_track_mut(track_path)?
            .ok_or_else(|| format!("missing animation track {track_path}"))?;
        if track
            .channel
            .keys
            .iter()
            .any(|key| (key.time_seconds - time_seconds).abs() <= f32::EPSILON)
        {
            return Ok(false);
        }
        let value = track
            .channel
            .keys
            .last()
            .map(|key| key.value.clone())
            .unwrap_or(AnimationChannelValueAsset::Scalar(0.0));
        track.channel.keys.push(AnimationChannelKeyAsset {
            time_seconds,
            value,
            in_tangent: None,
            out_tangent: None,
        });
        track
            .channel
            .keys
            .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_key(
        &mut self,
        track_path: &AnimationTrackPath,
        frame: u32,
    ) -> Result<bool, String> {
        let time_seconds = frame_to_seconds(frame, self.sequence_frames_per_second());
        let track = self
            .sequence_track_mut(track_path)?
            .ok_or_else(|| format!("missing animation track {track_path}"))?;
        let before = track.channel.keys.len();
        track
            .channel
            .keys
            .retain(|key| (key.time_seconds - time_seconds).abs() > f32::EPSILON);
        let changed = before != track.channel.keys.len();
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn create_track(&mut self, track_path: &AnimationTrackPath) -> Result<bool, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        let binding_index = document
            .asset
            .bindings
            .iter()
            .position(|binding| binding.entity_path == entity_path);
        let binding = if let Some(binding_index) = binding_index {
            &mut document.asset.bindings[binding_index]
        } else {
            document.asset.bindings.push(AnimationSequenceBindingAsset {
                entity_path,
                tracks: Vec::new(),
            });
            document
                .asset
                .bindings
                .last_mut()
                .expect("binding just pushed")
        };
        if binding
            .tracks
            .iter()
            .any(|track| track.property_path == property_path)
        {
            return Ok(false);
        }
        binding.tracks.push(AnimationSequenceTrackAsset {
            property_path,
            channel: default_channel(),
        });
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_track(&mut self, track_path: &AnimationTrackPath) -> Result<bool, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        let mut changed = false;
        document.asset.bindings.retain_mut(|binding| {
            if binding.entity_path != entity_path {
                return true;
            }
            let before = binding.tracks.len();
            binding
                .tracks
                .retain(|track| track.property_path != property_path);
            changed |= before != binding.tracks.len();
            !binding.tracks.is_empty()
        });
        if changed
            && matches!(
                document.selected_span.as_ref(),
                Some((selected_track_path, _, _)) if selected_track_path == track_path
            )
        {
            document.selected_span = None;
        }
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn rebind_track(
        &mut self,
        from_track_path: &AnimationTrackPath,
        to_track_path: &AnimationTrackPath,
    ) -> Result<bool, String> {
        let (from_entity, from_property) =
            from_track_path.split().map_err(|error| error.to_string())?;
        let (to_entity, to_property) = to_track_path.split().map_err(|error| error.to_string())?;
        if from_entity == to_entity && from_property == to_property {
            return Ok(false);
        }
        let document = self.sequence_document_mut()?;
        if document.asset.bindings.iter().any(|binding| {
            binding.entity_path == to_entity
                && binding
                    .tracks
                    .iter()
                    .any(|track| track.property_path == to_property)
        }) {
            return Ok(false);
        }
        let mut moved_track = None;
        document.asset.bindings.retain_mut(|binding| {
            if binding.entity_path != from_entity {
                return true;
            }
            if let Some(track_index) = binding
                .tracks
                .iter()
                .position(|track| track.property_path == from_property)
            {
                moved_track = Some(binding.tracks.remove(track_index));
            }
            !binding.tracks.is_empty()
        });
        let Some(mut moved_track) = moved_track else {
            return Ok(false);
        };
        moved_track.property_path = to_property;
        let binding_index = document
            .asset
            .bindings
            .iter()
            .position(|binding| binding.entity_path == to_entity);
        let binding = if let Some(binding_index) = binding_index {
            &mut document.asset.bindings[binding_index]
        } else {
            document.asset.bindings.push(AnimationSequenceBindingAsset {
                entity_path: to_entity,
                tracks: Vec::new(),
            });
            document
                .asset
                .bindings
                .last_mut()
                .expect("binding just pushed")
        };
        binding.tracks.push(moved_track);
        if let Some((selected_track_path, start_frame, end_frame)) = document.selected_span.clone()
        {
            if selected_track_path == *from_track_path {
                document.selected_span = Some((to_track_path.clone(), start_frame, end_frame));
            }
        }
        self.dirty = true;
        Ok(true)
    }

    pub fn scrub_timeline(&mut self, frame: u32) -> Result<bool, String> {
        let document = self.sequence_document_mut()?;
        let next = frame.clamp(document.timeline_start_frame, document.timeline_end_frame);
        let changed = document.current_frame != next;
        document.current_frame = next;
        Ok(changed)
    }

    pub fn set_timeline_range(&mut self, start_frame: u32, end_frame: u32) -> Result<bool, String> {
        let document = self.sequence_document_mut()?;
        let (next_start, next_end) = if start_frame <= end_frame {
            (start_frame, end_frame)
        } else {
            (end_frame, start_frame)
        };
        let changed =
            document.timeline_start_frame != next_start || document.timeline_end_frame != next_end;
        document.timeline_start_frame = next_start;
        document.timeline_end_frame = next_end;
        document.current_frame = document.current_frame.clamp(next_start, next_end);
        if let Some((track_path, selected_start, selected_end)) = document.selected_span.clone() {
            let (selected_start, selected_end) =
                clamp_timeline_span(selected_start, selected_end, next_start, next_end);
            document.selected_span = Some((track_path, selected_start, selected_end));
        }
        Ok(changed)
    }

    pub fn select_timeline_span(
        &mut self,
        track_path: &AnimationTrackPath,
        start_frame: u32,
        end_frame: u32,
    ) -> Result<bool, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        let has_track = document.asset.bindings.iter().any(|binding| {
            binding.entity_path == entity_path
                && binding
                    .tracks
                    .iter()
                    .any(|track| track.property_path == property_path)
        });
        if !has_track {
            return Ok(false);
        }
        let (start_frame, end_frame) = if start_frame <= end_frame {
            (start_frame, end_frame)
        } else {
            (end_frame, start_frame)
        };
        let (start_frame, end_frame) = clamp_timeline_span(
            start_frame,
            end_frame,
            document.timeline_start_frame,
            document.timeline_end_frame,
        );
        let next = Some((track_path.clone(), start_frame, end_frame));
        let changed = document.selected_span != next;
        document.selected_span = next;
        Ok(changed)
    }

    pub fn set_playback(
        &mut self,
        playing: bool,
        looping: bool,
        speed: f32,
    ) -> Result<bool, String> {
        if !speed.is_finite() {
            return Ok(false);
        }
        let document = self.sequence_document_mut()?;
        let changed = document.playing != playing
            || document.looping != looping
            || (document.speed - speed).abs() > f32::EPSILON;
        document.playing = playing;
        document.looping = looping;
        document.speed = speed;
        Ok(changed)
    }

    pub fn add_graph_node(&mut self, node_id: &str, node_kind: &str) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        let node = match node_kind.to_ascii_lowercase().as_str() {
            "output" => {
                if graph_has_output_node(asset) {
                    return Ok(false);
                }
                AnimationGraphNodeAsset::Output {
                    source: String::new(),
                }
            }
            "blend" => {
                if asset
                    .nodes
                    .iter()
                    .any(|node| graph_node_id(node) == Some(node_id))
                {
                    return Ok(false);
                }
                AnimationGraphNodeAsset::Blend {
                    id: node_id.to_string(),
                    inputs: Vec::new(),
                    weight_parameter: None,
                }
            }
            _ => return Ok(false),
        };
        asset.nodes.push(node);
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_graph_node(&mut self, node_id: &str) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        let before = asset.nodes.len();
        asset.nodes.retain(|node| {
            graph_node_id(node) != Some(node_id)
                && !(node_id == "output" && matches!(node, AnimationGraphNodeAsset::Output { .. }))
        });
        for node in &mut asset.nodes {
            match node {
                AnimationGraphNodeAsset::Blend { inputs, .. } => {
                    inputs.retain(|input| input != node_id);
                }
                AnimationGraphNodeAsset::Output { source } if source == node_id => {
                    source.clear();
                }
                _ => {}
            }
        }
        let changed = before != asset.nodes.len();
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn connect_graph_nodes(
        &mut self,
        from_node_id: &str,
        to_node_id: &str,
    ) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        if from_node_id == to_node_id {
            return Ok(false);
        }
        if !graph_has_named_node(asset, from_node_id) {
            return Ok(false);
        }
        let mut changed = false;
        for node in &mut asset.nodes {
            match node {
                AnimationGraphNodeAsset::Blend { id, inputs, .. } if id == to_node_id => {
                    if !inputs.iter().any(|input| input == from_node_id) {
                        inputs.push(from_node_id.to_string());
                        changed = true;
                    }
                }
                AnimationGraphNodeAsset::Output { source } if to_node_id == "output" => {
                    if source != from_node_id {
                        *source = from_node_id.to_string();
                        changed = true;
                    }
                }
                _ => {}
            }
        }
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn disconnect_graph_nodes(
        &mut self,
        from_node_id: &str,
        to_node_id: &str,
    ) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        let mut changed = false;
        for node in &mut asset.nodes {
            match node {
                AnimationGraphNodeAsset::Blend { id, inputs, .. } if id == to_node_id => {
                    let before = inputs.len();
                    inputs.retain(|input| input != from_node_id);
                    changed |= before != inputs.len();
                }
                AnimationGraphNodeAsset::Output { source } if to_node_id == "output" => {
                    if source == from_node_id {
                        source.clear();
                        changed = true;
                    }
                }
                _ => {}
            }
        }
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn set_graph_parameter(
        &mut self,
        parameter_name: &str,
        value_literal: &str,
    ) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        if let Some(parameter) = asset
            .parameters
            .iter_mut()
            .find(|parameter| parameter.name == parameter_name)
        {
            let Some(next) = parse_parameter_value(Some(&parameter.default_value), value_literal)
            else {
                return Ok(false);
            };
            let changed = parameter.default_value != next;
            parameter.default_value = next;
            self.dirty |= changed;
            return Ok(changed);
        }
        let Some(default_value) = parse_parameter_value(None, value_literal) else {
            return Ok(false);
        };
        asset.parameters.push(AnimationGraphParameterAsset {
            name: parameter_name.to_string(),
            default_value,
        });
        self.dirty = true;
        Ok(true)
    }

    pub fn create_state(&mut self, state_name: &str, graph_path: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if asset.states.iter().any(|state| state.name == state_name) {
            return Ok(false);
        }
        let graph = AssetReference::from_locator(
            AssetUri::parse(graph_path).map_err(|error| error.to_string())?,
        );
        asset.states.push(AnimationStateAsset {
            name: state_name.to_string(),
            graph,
        });
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_state(&mut self, state_name: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        let before = asset.states.len();
        asset.states.retain(|state| state.name != state_name);
        if before == asset.states.len() {
            return Ok(false);
        }
        asset.transitions.retain(|transition| {
            transition.from_state != state_name && transition.to_state != state_name
        });
        if asset.entry_state == state_name {
            asset.entry_state = asset
                .states
                .first()
                .map(|state| state.name.clone())
                .unwrap_or_default();
        }
        self.dirty = true;
        Ok(true)
    }

    pub fn set_entry_state(&mut self, state_name: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if asset.entry_state == state_name {
            return Ok(false);
        }
        if !state_machine_has_state(asset, state_name) {
            return Ok(false);
        }
        asset.entry_state = state_name.to_string();
        self.dirty = true;
        Ok(true)
    }

    pub fn create_transition(
        &mut self,
        from_state: &str,
        to_state: &str,
        duration_frames: u32,
    ) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if !state_machine_has_state(asset, from_state) || !state_machine_has_state(asset, to_state)
        {
            return Ok(false);
        }
        if let Some(transition) = asset.transitions.iter_mut().find(|transition| {
            transition.from_state == from_state && transition.to_state == to_state
        }) {
            let duration_seconds =
                frame_to_seconds(duration_frames, DEFAULT_STATE_MACHINE_TRANSITION_FPS);
            let changed = (transition.duration_seconds - duration_seconds).abs() > f32::EPSILON;
            transition.duration_seconds = duration_seconds;
            self.dirty |= changed;
            return Ok(changed);
        }
        asset.transitions.push(AnimationStateTransitionAsset {
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            duration_seconds: frame_to_seconds(
                duration_frames,
                DEFAULT_STATE_MACHINE_TRANSITION_FPS,
            ),
            conditions: Vec::new(),
        });
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_transition(&mut self, from_state: &str, to_state: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        let before = asset.transitions.len();
        asset.transitions.retain(|transition| {
            !(transition.from_state == from_state && transition.to_state == to_state)
        });
        let changed = before != asset.transitions.len();
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn set_transition_condition(
        &mut self,
        from_state: &str,
        to_state: &str,
        parameter_name: &str,
        operator: &str,
        value_literal: &str,
    ) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if !state_machine_has_state(asset, from_state) || !state_machine_has_state(asset, to_state)
        {
            return Ok(false);
        }
        let Some(transition_index) = asset.transitions.iter().position(|transition| {
            transition.from_state == from_state && transition.to_state == to_state
        }) else {
            return Ok(false);
        };
        let Some(operator) = parse_condition_operator(operator) else {
            return Ok(false);
        };
        let transition = &mut asset.transitions[transition_index];
        let existing_value = transition
            .conditions
            .iter()
            .find(|condition| condition.parameter == parameter_name)
            .and_then(|condition| condition.value.clone());
        let Some(value) = parse_parameter_value(existing_value.as_ref(), value_literal) else {
            return Ok(false);
        };
        let next_condition = AnimationTransitionConditionAsset {
            parameter: parameter_name.to_string(),
            operator,
            value: Some(value),
        };
        if let Some(condition) = transition
            .conditions
            .iter_mut()
            .find(|condition| condition.parameter == parameter_name)
        {
            let changed = *condition != next_condition;
            *condition = next_condition;
            self.dirty |= changed;
            return Ok(changed);
        }
        transition.conditions.push(next_condition);
        self.dirty = true;
        Ok(true)
    }

    fn sequence_document_mut(&mut self) -> Result<&mut AnimationSequenceDocument, String> {
        match &mut self.document {
            AnimationEditorDocument::Sequence(document) => Ok(document),
            _ => Err("active animation editor is not a sequence document".to_string()),
        }
    }

    fn graph_asset_mut(&mut self) -> Result<&mut AnimationGraphAsset, String> {
        match &mut self.document {
            AnimationEditorDocument::Graph(asset) => Ok(asset),
            _ => Err("active animation editor is not a graph document".to_string()),
        }
    }

    fn state_machine_asset_mut(&mut self) -> Result<&mut AnimationStateMachineAsset, String> {
        match &mut self.document {
            AnimationEditorDocument::StateMachine(asset) => Ok(asset),
            _ => Err("active animation editor is not a state-machine document".to_string()),
        }
    }

    fn sequence_frames_per_second(&self) -> f32 {
        match &self.document {
            AnimationEditorDocument::Sequence(document) => {
                sanitize_frames_per_second(document.asset.frames_per_second)
            }
            _ => DEFAULT_SEQUENCE_FRAMES_PER_SECOND,
        }
    }

    fn sequence_track_mut(
        &mut self,
        track_path: &AnimationTrackPath,
    ) -> Result<Option<&mut AnimationSequenceTrackAsset>, String> {
        let (entity_path, property_path) = track_path.split().map_err(|error| error.to_string())?;
        let document = self.sequence_document_mut()?;
        for binding in &mut document.asset.bindings {
            if binding.entity_path != entity_path {
                continue;
            }
            if let Some(track) = binding
                .tracks
                .iter_mut()
                .find(|track| track.property_path == property_path)
            {
                return Ok(Some(track));
            }
        }
        Ok(None)
    }

    fn document_bytes(&self) -> Result<Vec<u8>, AnimationEditorSessionError> {
        match &self.document {
            AnimationEditorDocument::Sequence(document) => document
                .asset
                .to_bytes()
                .map_err(AnimationEditorSessionError),
            AnimationEditorDocument::Graph(asset) => {
                asset.to_bytes().map_err(AnimationEditorSessionError)
            }
            AnimationEditorDocument::StateMachine(asset) => {
                asset.to_bytes().map_err(AnimationEditorSessionError)
            }
        }
    }
}

fn default_channel() -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(0.0),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn fallback_title(asset_path: &str) -> String {
    Path::new(asset_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| asset_path.to_string())
}

fn duration_frames(duration_seconds: f32, frames_per_second: f32) -> u32 {
    (sanitize_duration_seconds(duration_seconds) * sanitize_frames_per_second(frames_per_second))
        .round() as u32
}

fn clamp_timeline_span(
    start_frame: u32,
    end_frame: u32,
    range_start: u32,
    range_end: u32,
) -> (u32, u32) {
    (
        start_frame.clamp(range_start, range_end),
        end_frame.clamp(range_start, range_end),
    )
}

fn frame_to_seconds(frame: u32, frames_per_second: f32) -> f32 {
    frame as f32 / sanitize_frames_per_second(frames_per_second).max(1.0)
}

fn sanitize_duration_seconds(duration_seconds: f32) -> f32 {
    if duration_seconds.is_finite() && duration_seconds >= 0.0 {
        duration_seconds
    } else {
        0.0
    }
}

fn sanitize_frames_per_second(frames_per_second: f32) -> f32 {
    if frames_per_second.is_finite() && frames_per_second > 0.0 {
        frames_per_second
    } else {
        DEFAULT_SEQUENCE_FRAMES_PER_SECOND
    }
}

fn graph_node_id(node: &AnimationGraphNodeAsset) -> Option<&str> {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. } => Some(id),
        AnimationGraphNodeAsset::Blend { id, .. } => Some(id),
        AnimationGraphNodeAsset::Output { .. } => None,
    }
}

fn graph_has_output_node(asset: &AnimationGraphAsset) -> bool {
    asset
        .nodes
        .iter()
        .any(|node| matches!(node, AnimationGraphNodeAsset::Output { .. }))
}

fn graph_has_named_node(asset: &AnimationGraphAsset, node_id: &str) -> bool {
    asset
        .nodes
        .iter()
        .any(|node| graph_node_id(node) == Some(node_id))
}

fn graph_node_label(node: &AnimationGraphNodeAsset) -> String {
    match node {
        AnimationGraphNodeAsset::Clip { id, clip, .. } => {
            format!("Clip {id} • {}", clip.locator)
        }
        AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
            if inputs.is_empty() {
                format!("Blend {id}")
            } else {
                format!("Blend {id} • {}", inputs.join(", "))
            }
        }
        AnimationGraphNodeAsset::Output { source } => format!("Output <- {source}"),
    }
}

fn transition_label(transition: &AnimationStateTransitionAsset) -> String {
    format!("{} -> {}", transition.from_state, transition.to_state)
}

fn state_machine_has_state(asset: &AnimationStateMachineAsset, state_name: &str) -> bool {
    asset.states.iter().any(|state| state.name == state_name)
}

fn parameter_value_label(value: &AnimationParameterValue) -> String {
    match value {
        AnimationParameterValue::Bool(value) => value.to_string(),
        AnimationParameterValue::Integer(value) => value.to_string(),
        AnimationParameterValue::Scalar(value) => format!("{value:.2}"),
        AnimationParameterValue::Vec2(value) => format!("{}, {}", value[0], value[1]),
        AnimationParameterValue::Vec3(value) => {
            format!("{}, {}, {}", value[0], value[1], value[2])
        }
        AnimationParameterValue::Vec4(value) => {
            format!("{}, {}, {}, {}", value[0], value[1], value[2], value[3])
        }
        AnimationParameterValue::Trigger => "trigger".to_string(),
    }
}

fn parse_parameter_value(
    existing: Option<&AnimationParameterValue>,
    value_literal: &str,
) -> Option<AnimationParameterValue> {
    match existing {
        Some(AnimationParameterValue::Trigger) => parse_trigger_literal(value_literal),
        Some(AnimationParameterValue::Bool(_)) => {
            parse_bool_literal(value_literal).map(AnimationParameterValue::Bool)
        }
        Some(AnimationParameterValue::Integer(_)) => value_literal
            .parse::<i32>()
            .ok()
            .map(AnimationParameterValue::Integer),
        Some(AnimationParameterValue::Scalar(_)) => {
            parse_finite_scalar_literal(value_literal).map(AnimationParameterValue::Scalar)
        }
        Some(AnimationParameterValue::Vec2(_)) => {
            parse_vector_literal::<2>(value_literal).map(AnimationParameterValue::Vec2)
        }
        Some(AnimationParameterValue::Vec3(_)) => {
            parse_vector_literal::<3>(value_literal).map(AnimationParameterValue::Vec3)
        }
        Some(AnimationParameterValue::Vec4(_)) => {
            parse_vector_literal::<4>(value_literal).map(AnimationParameterValue::Vec4)
        }
        None => parse_trigger_literal(value_literal)
            .or_else(|| parse_bool_literal(value_literal).map(AnimationParameterValue::Bool))
            .or_else(|| {
                value_literal
                    .parse::<i32>()
                    .ok()
                    .map(AnimationParameterValue::Integer)
            })
            .or_else(|| {
                parse_finite_scalar_literal(value_literal).map(AnimationParameterValue::Scalar)
            })
            .or_else(|| parse_vector_literal::<2>(value_literal).map(AnimationParameterValue::Vec2))
            .or_else(|| parse_vector_literal::<3>(value_literal).map(AnimationParameterValue::Vec3))
            .or_else(|| {
                parse_vector_literal::<4>(value_literal).map(AnimationParameterValue::Vec4)
            }),
    }
}

fn parse_finite_scalar_literal(value_literal: &str) -> Option<f32> {
    let value = value_literal.parse::<f32>().ok()?;
    value.is_finite().then_some(value)
}

fn parse_trigger_literal(value_literal: &str) -> Option<AnimationParameterValue> {
    value_literal
        .eq_ignore_ascii_case("trigger")
        .then_some(AnimationParameterValue::Trigger)
}

fn parse_bool_literal(value_literal: &str) -> Option<bool> {
    if value_literal.eq_ignore_ascii_case("true") {
        Some(true)
    } else if value_literal.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

fn parse_vector_literal<const N: usize>(value_literal: &str) -> Option<[f32; N]> {
    let parts: Vec<_> = value_literal.split(',').map(str::trim).collect();
    if parts.len() != N {
        return None;
    }
    let mut values = [0.0; N];
    for (index, part) in parts.into_iter().enumerate() {
        values[index] = parse_finite_scalar_literal(part)?;
    }
    Some(values)
}

fn parse_condition_operator(operator: &str) -> Option<AnimationConditionOperatorAsset> {
    match operator {
        "equal" => Some(AnimationConditionOperatorAsset::Equal),
        "not_equal" => Some(AnimationConditionOperatorAsset::NotEqual),
        "greater" => Some(AnimationConditionOperatorAsset::Greater),
        "greater_equal" => Some(AnimationConditionOperatorAsset::GreaterEqual),
        "less" => Some(AnimationConditionOperatorAsset::Less),
        "less_equal" => Some(AnimationConditionOperatorAsset::LessEqual),
        "triggered" => Some(AnimationConditionOperatorAsset::Triggered),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
