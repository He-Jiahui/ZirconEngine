//! Save As changes a source identity without replacing the document or its edit history.
use super::{
    active_scene_identity_from_state, ActiveSceneDocumentIdentity, DocumentLifecycleAuthority,
    SceneDocumentKey, SceneDocumentLifecycleError, ScenePickerTicket, SceneSourceRevision,
};

pub(in crate::core::document) struct SceneSaveAsReservation {
    identity: ActiveSceneDocumentIdentity,
    target: SceneDocumentKey,
    activation_revision: u64,
}

impl DocumentLifecycleAuthority {
    pub(in crate::core::document) fn prepare_scene_save_as_while_routed(
        &self,
        ticket: &ScenePickerTicket,
        scene_uri: &str,
    ) -> Result<SceneSaveAsReservation, SceneDocumentLifecycleError> {
        self.validate_scene_picker_ticket_while_routed(ticket, ticket.project_root())?;
        let mut state = self.lock_state();
        let identity = active_scene_identity_from_state(&state, Some(ticket.project_root()))
            .ok_or(SceneDocumentLifecycleError::NoActiveScene)?;
        if !ticket
            .active_scene
            .as_ref()
            .is_some_and(|picked| picked.same_activation(&identity))
        {
            return Err(SceneDocumentLifecycleError::StaleSaveAsScene);
        }
        let activation_revision = state
            .next_scene_activation_revision
            .checked_add(1)
            .ok_or(SceneDocumentLifecycleError::SceneActivationRevisionExhausted)?;
        state.next_scene_activation_revision = activation_revision;
        Ok(SceneSaveAsReservation {
            target: SceneDocumentKey {
                project_root: identity.project_root.clone(),
                scene_uri: scene_uri.to_string(),
            },
            identity,
            activation_revision,
        })
    }

    /// The route holds admission throughout publication, making this final identity move infallible.
    pub(in crate::core::document) fn commit_scene_save_as_while_routed(
        &self,
        reservation: SceneSaveAsReservation,
        source_revision: SceneSourceRevision,
    ) -> ActiveSceneDocumentIdentity {
        let mut state = self.lock_state();
        debug_assert_eq!(
            active_scene_identity_from_state(&state, Some(reservation.identity.project_root())),
            Some(reservation.identity.clone())
        );
        let previous = state
            .active_scene_key
            .take()
            .expect("reserved active scene");
        state.ids_by_scene_key.remove(&previous);
        if let Some(displaced) = state
            .ids_by_scene_key
            .insert(reservation.target.clone(), reservation.identity.document())
        {
            // A deleted target may retain an inactive ID. Keep only one key per occupied ID.
            if displaced != reservation.identity.document() {
                state.occupied_document_ids.remove(&displaced);
            }
        }
        state.active_scene_key = Some(reservation.target);
        state.active_scene_activation_revision = Some(reservation.activation_revision);
        state.active_scene_source_revision = Some(source_revision);
        state.trim_closed_scene_documents();
        active_scene_identity_from_state(&state, Some(reservation.identity.project_root()))
            .expect("Save As preserves its active document")
    }
}
