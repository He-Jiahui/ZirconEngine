use super::super::editor_error::{
    AnimationEditorTargetDiagnostic, AnimationEditorTargetKind,
    AnimationEditorTargetUnavailableReason, EditorError,
};
use super::super::editor_ui_host::EditorUiHost;
use crate::core::editing::animation_document::{AnimationDocumentMutation, AnimationEditCommand};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditCommandError, HistoryContextId};
use crate::core::editor_event::EditorAnimationEvent;
use crate::ui::animation_editor::{resolve_animation_graph_node_kind, AnimationEditorDocumentKind};
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub fn apply_animation_event(&self, event: &EditorAnimationEvent) -> Result<bool, EditorError> {
        match event {
            EditorAnimationEvent::AddKey { track_path, frame } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::AddKey {
                        track_path: track_path.clone(),
                        frame: *frame,
                    },
                )
            }
            EditorAnimationEvent::RemoveKey { track_path, frame } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::RemoveKey {
                        track_path: track_path.clone(),
                        frame: *frame,
                    },
                )
            }
            EditorAnimationEvent::CreateTrack { track_path } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::CreateTrack {
                        track_path: track_path.clone(),
                    },
                )
            }
            EditorAnimationEvent::RemoveTrack { track_path } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::RemoveTrack {
                        track_path: track_path.clone(),
                    },
                )
            }
            EditorAnimationEvent::RebindTrack {
                from_track_path,
                to_track_path,
            } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::RebindTrack {
                        from_track_path: from_track_path.clone(),
                        to_track_path: to_track_path.clone(),
                    },
                )
            }
            EditorAnimationEvent::ScrubTimeline { frame } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.with_animation_transient_session_mut(&instance_id, |session| {
                    session.scrub_timeline(*frame).map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::SetTimelineRange {
                start_frame,
                end_frame,
            } => {
                let instance_id = self.focused_animation_sequence_instance()?;
                self.with_animation_transient_session_mut(&instance_id, |session| {
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
                let instance_id = self.focused_animation_sequence_instance()?;
                self.with_animation_transient_session_mut(&instance_id, |session| {
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
                let instance_id = self.focused_animation_sequence_instance()?;
                self.with_animation_transient_session_mut(&instance_id, |session| {
                    session
                        .set_playback(*playing, *looping, *speed)
                        .map_err(EditorError::UiAsset)
                })
            }
            EditorAnimationEvent::AddGraphNode {
                graph_locator,
                node_id,
                node_kind,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_locator))?;
                let node_kind =
                    resolve_animation_graph_node_kind(node_kind).map_err(|diagnostic| {
                        EditorError::AnimationCommandUnavailable { diagnostic }
                    })?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::AddGraphNode {
                        node_id: node_id.clone(),
                        node_kind,
                    },
                )
            }
            EditorAnimationEvent::RemoveGraphNode {
                graph_locator,
                node_id,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::RemoveGraphNode {
                        node_id: node_id.clone(),
                    },
                )
            }
            EditorAnimationEvent::ConnectGraphNodes {
                graph_locator,
                from_node_id,
                to_node_id,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::ConnectGraphNodes {
                        from_node_id: from_node_id.clone(),
                        to_node_id: to_node_id.clone(),
                    },
                )
            }
            EditorAnimationEvent::DisconnectGraphNodes {
                graph_locator,
                from_node_id,
                to_node_id,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::DisconnectGraphNodes {
                        from_node_id: from_node_id.clone(),
                        to_node_id: to_node_id.clone(),
                    },
                )
            }
            EditorAnimationEvent::SetGraphParameter {
                graph_locator,
                parameter_name,
                value_literal,
            } => {
                let instance_id = self.resolve_animation_graph_instance(Some(graph_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::SetGraphParameter {
                        parameter_name: parameter_name.clone(),
                        value_literal: value_literal.clone(),
                    },
                )
            }
            EditorAnimationEvent::CreateState {
                state_machine_locator,
                state_name,
                graph_locator,
            } => {
                let instance_id =
                    self.resolve_animation_state_machine_instance(Some(state_machine_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::CreateState {
                        state_name: state_name.clone(),
                        graph_locator: graph_locator.clone(),
                    },
                )
            }
            EditorAnimationEvent::RemoveState {
                state_machine_locator,
                state_name,
            } => {
                let instance_id =
                    self.resolve_animation_state_machine_instance(Some(state_machine_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::RemoveState {
                        state_name: state_name.clone(),
                    },
                )
            }
            EditorAnimationEvent::SetEntryState {
                state_machine_locator,
                state_name,
            } => {
                let instance_id =
                    self.resolve_animation_state_machine_instance(Some(state_machine_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::SetEntryState {
                        state_name: state_name.clone(),
                    },
                )
            }
            EditorAnimationEvent::CreateTransition {
                state_machine_locator,
                from_state,
                to_state,
                duration_frames,
            } => {
                let instance_id =
                    self.resolve_animation_state_machine_instance(Some(state_machine_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::CreateTransition {
                        from_state: from_state.clone(),
                        to_state: to_state.clone(),
                        duration_frames: *duration_frames,
                    },
                )
            }
            EditorAnimationEvent::RemoveTransition {
                state_machine_locator,
                from_state,
                to_state,
            } => {
                let instance_id =
                    self.resolve_animation_state_machine_instance(Some(state_machine_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::RemoveTransition {
                        from_state: from_state.clone(),
                        to_state: to_state.clone(),
                    },
                )
            }
            EditorAnimationEvent::SetTransitionCondition {
                state_machine_locator,
                from_state,
                to_state,
                parameter_name,
                operator,
                value_literal,
            } => {
                let instance_id =
                    self.resolve_animation_state_machine_instance(Some(state_machine_locator))?;
                self.apply_animation_document_mutation(
                    &instance_id,
                    AnimationDocumentMutation::SetTransitionCondition {
                        from_state: from_state.clone(),
                        to_state: to_state.clone(),
                        parameter_name: parameter_name.clone(),
                        operator: operator.clone(),
                        value_literal: value_literal.clone(),
                    },
                )
            }
        }
    }

    fn apply_animation_document_mutation(
        &self,
        instance_id: &ViewInstanceId,
        mutation: AnimationDocumentMutation,
    ) -> Result<bool, EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        let document = self.animation_document_for_instance(instance_id)?;
        let prepared = self
            .transactions
            .with_context::<CoreEditContext, _>(|context| {
                context
                    .animation_documents()
                    .prepare_mutation(document, &mutation)
            })
            .map_err(animation_transaction_error)?
            .ok_or_else(|| {
                EditorError::UiAsset("animation transaction context type mismatch".to_string())
            })?
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let Some((expected_revision, replacement)) = prepared else {
            return Ok(false);
        };
        let mut transaction = self
            .transactions
            .begin(mutation.label(), HistoryContextId::Document(document))
            .map_err(animation_transaction_error)?;
        transaction
            .push(AnimationEditCommand::new(
                mutation.label(),
                document,
                expected_revision,
                replacement,
            ))
            .map_err(animation_transaction_error)?;
        transaction
            .commit_after_apply(|_| self.project_animation_document_commit(instance_id))
            .map_err(animation_transaction_error)?;
        self.reconcile_animation_session_after_mutation(instance_id, &mutation);
        Ok(true)
    }

    fn reconcile_animation_session_after_mutation(
        &self,
        instance_id: &ViewInstanceId,
        mutation: &AnimationDocumentMutation,
    ) {
        let mut sessions = self.lock_animation_editor_sessions();
        let Some(entry) = sessions.get_mut(instance_id) else {
            return;
        };
        match mutation {
            AnimationDocumentMutation::RemoveTrack { track_path } => {
                entry.session.clear_selected_timeline_track_if(track_path);
            }
            AnimationDocumentMutation::RebindTrack {
                from_track_path,
                to_track_path,
            } => {
                entry
                    .session
                    .rebind_selected_timeline_track_if(from_track_path, to_track_path);
            }
            _ => {}
        }
    }

    pub(super) fn animation_document_for_instance(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<crate::core::editor_message::DocumentId, EditorError> {
        let sessions = self.lock_animation_editor_sessions();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!(
                "missing animation editor session {}",
                instance_id.0
            ))
        })?;
        debug_assert_eq!(entry.document, entry.session.document().document_id());
        Ok(entry.document)
    }

    fn project_animation_document_commit(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditCommandError> {
        let (title, payload) = {
            let sessions = self.lock_animation_editor_sessions();
            let entry =
                sessions
                    .get(instance_id)
                    .ok_or_else(|| EditCommandError::ExternalEffect {
                        source: Box::new(EditorError::UiAsset(format!(
                            "missing animation editor session {}",
                            instance_id.0
                        ))),
                    })?;
            (
                entry.session.display_name(),
                serde_json::to_value(&entry.route).map_err(|error| {
                    EditCommandError::ExternalEffect {
                        source: Box::new(error),
                    }
                })?,
            )
        };
        self.update_view_instance_metadata(instance_id, Some(title), Some(true), Some(payload))
            .map_err(|error| EditCommandError::ExternalEffect {
                source: Box::new(error),
            })
    }

    fn with_animation_transient_session_mut<F>(
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
        let mut sessions = self.lock_animation_editor_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!(
                "missing animation editor session {}",
                instance_id.0
            ))
        })?;
        mutator(&mut entry.session)
    }

    fn focused_animation_sequence_instance(&self) -> Result<ViewInstanceId, EditorError> {
        let session = self.lock_session();
        let instance_id = session.focused_view.clone().ok_or_else(|| {
            EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    AnimationEditorTargetKind::Sequence,
                    AnimationEditorTargetUnavailableReason::NoFocusedView,
                ),
            }
        })?;
        let descriptor_id = session
            .open_view_instances
            .get(&instance_id)
            .map(|instance| instance.descriptor_id.0.as_str())
            .ok_or_else(|| EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    AnimationEditorTargetKind::Sequence,
                    AnimationEditorTargetUnavailableReason::MissingFocusedView,
                ),
            })?;
        if descriptor_id != "editor.animation_sequence" {
            return Err(EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    AnimationEditorTargetKind::Sequence,
                    AnimationEditorTargetUnavailableReason::WrongFocusedViewKind,
                ),
            });
        }
        Ok(instance_id)
    }

    fn resolve_animation_graph_instance(
        &self,
        asset_locator: Option<&str>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.resolve_animation_document_instance(
            asset_locator,
            AnimationEditorDocumentKind::Graph,
            AnimationEditorTargetKind::Graph,
        )
    }

    fn resolve_animation_state_machine_instance(
        &self,
        asset_locator: Option<&str>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.resolve_animation_document_instance(
            asset_locator,
            AnimationEditorDocumentKind::StateMachine,
            AnimationEditorTargetKind::StateMachine,
        )
    }

    fn resolve_animation_document_instance(
        &self,
        asset_locator: Option<&str>,
        expected_document_kind: AnimationEditorDocumentKind,
        target_kind: AnimationEditorTargetKind,
    ) -> Result<ViewInstanceId, EditorError> {
        if let Some(asset_locator) = asset_locator {
            if let Some(instance_id) =
                self.find_animation_editor_instance("editor.animation_graph", asset_locator)
            {
                return self.require_animation_document_kind(
                    instance_id,
                    expected_document_kind,
                    target_kind,
                );
            }
        }
        let session = self.lock_session();
        let instance_id = session.focused_view.clone().ok_or_else(|| {
            EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    target_kind,
                    AnimationEditorTargetUnavailableReason::NoFocusedView,
                ),
            }
        })?;
        let descriptor_id = session
            .open_view_instances
            .get(&instance_id)
            .map(|instance| instance.descriptor_id.0.as_str())
            .ok_or_else(|| EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    target_kind,
                    AnimationEditorTargetUnavailableReason::MissingFocusedView,
                ),
            })?;
        if descriptor_id != "editor.animation_graph" {
            return Err(EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    target_kind,
                    AnimationEditorTargetUnavailableReason::WrongFocusedViewKind,
                ),
            });
        }
        drop(session);
        self.require_animation_document_kind(instance_id, expected_document_kind, target_kind)
    }

    fn require_animation_document_kind(
        &self,
        instance_id: ViewInstanceId,
        expected_document_kind: AnimationEditorDocumentKind,
        target_kind: AnimationEditorTargetKind,
    ) -> Result<ViewInstanceId, EditorError> {
        self.ensure_animation_editor_session(&instance_id)?;
        let actual_document_kind = self
            .lock_animation_editor_sessions()
            .get(&instance_id)
            .map(|entry| entry.session.document_kind())
            .ok_or_else(|| EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    target_kind,
                    AnimationEditorTargetUnavailableReason::MissingFocusedView,
                ),
            })?;
        if actual_document_kind != expected_document_kind {
            return Err(EditorError::AnimationTargetUnavailable {
                diagnostic: AnimationEditorTargetDiagnostic::new(
                    target_kind,
                    AnimationEditorTargetUnavailableReason::WrongDocumentKind,
                ),
            });
        }
        Ok(instance_id)
    }

    fn find_animation_editor_instance(
        &self,
        descriptor_id: &str,
        asset_locator: &str,
    ) -> Option<ViewInstanceId> {
        let asset_locator = zircon_runtime::asset::AssetUri::parse(asset_locator).ok()?;
        self.lock_session()
            .open_view_instances
            .values()
            .find(|instance| {
                instance.descriptor_id.0 == descriptor_id
                    && serde_json::from_value::<crate::core::asset::AssetToolkitOpenRoute>(
                        instance.serializable_payload.clone(),
                    )
                    .is_ok_and(|route| route.asset_locator() == &asset_locator)
            })
            .map(|instance| instance.instance_id.clone())
    }
}

fn animation_transaction_error(error: EditCommandError) -> EditorError {
    EditorError::UiAsset(error.to_string())
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn persistent_animation_mutations_prepare_a_snapshot_before_opening_history() {
        let source = include_str!("editing.rs");
        let body = source
            .split("fn apply_animation_document_mutation")
            .nth(1)
            .expect("persistent animation mutation helper")
            .split("fn animation_document_for_instance")
            .next()
            .expect("persistent animation mutation helper body");

        assert!(body.contains("prepare_mutation"));
        assert!(body.contains("HistoryContextId::Document"));
        assert!(body.contains("commit_after_apply"));
        assert!(!body.contains("ensure_document_external_effect"));
    }
}
