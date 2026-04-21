use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::core::editor_event::EditorAnimationEvent;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub fn apply_animation_event(&self, event: &EditorAnimationEvent) -> Result<bool, EditorError> {
        match event {
            EditorAnimationEvent::AddKey { track_path, frame } => {
                let instance_id = self.active_animation_sequence_instance()?;
                let changed = self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .add_key(track_path, *frame)
                        .map_err(EditorError::UiAsset)
                })?;
                Ok(changed)
            }
            EditorAnimationEvent::RemoveKey { track_path, frame } => {
                let instance_id = self.active_animation_sequence_instance()?;
                let changed = self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .remove_key(track_path, *frame)
                        .map_err(EditorError::UiAsset)
                })?;
                Ok(changed)
            }
            EditorAnimationEvent::CreateTrack { track_path } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .create_track(track_path)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::RemoveTrack { track_path } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .remove_track(track_path)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::RebindTrack {
                from_track_path,
                to_track_path,
            } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .rebind_track(from_track_path, to_track_path)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::ScrubTimeline { frame } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session.scrub_timeline(*frame).map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SetTimelineRange {
                start_frame,
                end_frame,
            } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .set_timeline_range(*start_frame, *end_frame)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SelectTimelineSpan {
                track_path,
                start_frame,
                end_frame,
            } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .select_timeline_span(track_path, *start_frame, *end_frame)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SetPlayback {
                playing,
                looping,
                speed,
            } => {
                let instance_id = self.active_animation_sequence_instance()?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .set_playback(*playing, *looping, *speed)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::AddGraphNode {
                graph_path,
                node_id,
                node_kind,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .add_graph_node(node_id, node_kind)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::RemoveGraphNode {
                graph_path,
                node_id,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .remove_graph_node(node_id)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::ConnectGraphNodes {
                graph_path,
                from_node_id,
                to_node_id,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .connect_graph_nodes(from_node_id, to_node_id)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::DisconnectGraphNodes {
                graph_path,
                from_node_id,
                to_node_id,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .disconnect_graph_nodes(from_node_id, to_node_id)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SetGraphParameter {
                graph_path,
                parameter_name,
                value_literal,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .set_graph_parameter(parameter_name, value_literal)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::CreateState {
                state_machine_path,
                state_name,
                graph_path,
            } => {
                let instance_id =
                    self.resolve_animation_graph_instance(Some(state_machine_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .create_state(state_name, graph_path)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::RemoveState {
                state_machine_path,
                state_name,
            } => {
                let instance_id =
                    self.resolve_animation_graph_instance(Some(state_machine_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .remove_state(state_name)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SetEntryState {
                state_machine_path,
                state_name,
            } => {
                let instance_id =
                    self.resolve_animation_graph_instance(Some(state_machine_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .set_entry_state(state_name)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::CreateTransition {
                state_machine_path,
                from_state,
                to_state,
                duration_frames,
            } => {
                let instance_id =
                    self.resolve_animation_graph_instance(Some(state_machine_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .create_transition(from_state, to_state, *duration_frames)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::RemoveTransition {
                state_machine_path,
                from_state,
                to_state,
            } => {
                let instance_id =
                    self.resolve_animation_graph_instance(Some(state_machine_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .remove_transition(from_state, to_state)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SetTransitionCondition {
                state_machine_path,
                from_state,
                to_state,
                parameter_name,
                operator,
                value_literal,
            } => {
                let instance_id =
                    self.resolve_animation_graph_instance(Some(state_machine_path))?;
                self.with_animation_session_mut(&instance_id, |session| {
                    session
                        .set_transition_condition(
                            from_state,
                            to_state,
                            parameter_name,
                            operator,
                            value_literal,
                        )
                        .map_err(EditorError::UiAsset)
                })
            }
        }
    }

    fn with_animation_session_mut<F>(
        &self,
        instance_id: &ViewInstanceId,
        mutator: F,
    ) -> Result<bool, EditorError>
    where
        F: FnOnce(
            &mut crate::ui::animation_editor::AnimationEditorSession,
        ) -> Result<bool, EditorError>,
    {
        self.ensure_animation_editor_session(instance_id)?;
        let (changed, title, dirty, payload) = {
            let mut sessions = self.animation_editor_sessions.lock().unwrap();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing animation editor session {}",
                    instance_id.0
                ))
            })?;
            let changed = mutator(&mut entry.session)?;
            (
                changed,
                entry.session.display_name(),
                entry.session.is_dirty(),
                serde_json::json!({ "path": entry.session.asset_path() }),
            )
        };
        self.update_view_instance_metadata(instance_id, Some(title), Some(dirty), Some(payload))?;
        Ok(changed)
    }

    fn active_animation_sequence_instance(&self) -> Result<ViewInstanceId, EditorError> {
        let session = self.session.lock().unwrap();
        let instance_id = session.active_center_tab.clone().ok_or_else(|| {
            EditorError::UiAsset("no active animation sequence editor".to_string())
        })?;
        let descriptor_id = session
            .open_view_instances
            .get(&instance_id)
            .map(|instance| instance.descriptor_id.0.as_str())
            .ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing active animation sequence view {}",
                    instance_id.0
                ))
            })?;
        if descriptor_id != "editor.animation_sequence" {
            return Err(EditorError::UiAsset(
                "active center tab is not an animation sequence editor".to_string(),
            ));
        }
        Ok(instance_id)
    }

    fn resolve_animation_graph_instance(
        &self,
        asset_path: Option<&str>,
    ) -> Result<ViewInstanceId, EditorError> {
        if let Some(asset_path) = asset_path {
            if let Some(instance_id) =
                self.find_animation_editor_instance("editor.animation_graph", asset_path)
            {
                return Ok(instance_id);
            }
        }
        let session = self.session.lock().unwrap();
        let instance_id = session
            .active_center_tab
            .clone()
            .ok_or_else(|| EditorError::UiAsset("no active animation graph editor".to_string()))?;
        let descriptor_id = session
            .open_view_instances
            .get(&instance_id)
            .map(|instance| instance.descriptor_id.0.as_str())
            .ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing active animation graph view {}",
                    instance_id.0
                ))
            })?;
        if descriptor_id != "editor.animation_graph" {
            return Err(EditorError::UiAsset(
                "active center tab is not an animation graph editor".to_string(),
            ));
        }
        Ok(instance_id)
    }

    fn find_animation_editor_instance(
        &self,
        descriptor_id: &str,
        asset_path: &str,
    ) -> Option<ViewInstanceId> {
        self.session
            .lock()
            .unwrap()
            .open_view_instances
            .values()
            .find(|instance| {
                instance.descriptor_id.0 == descriptor_id
                    && instance.serializable_payload["path"].as_str() == Some(asset_path)
            })
            .map(|instance| instance.instance_id.clone())
    }
}
