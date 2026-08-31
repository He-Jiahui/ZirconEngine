use crate::core::document::ActiveSceneDocumentIdentity;
use crate::core::notifications::{
    DecisionNotification, DecisionOption, DecisionOptionId, DecisionTicket, NotificationId,
    NotificationSource, MAX_DECISION_DISPLAY_SUBJECT_BYTES,
};
use crate::ui::host::PreparedActiveSceneReloadDirtyPolicy;
use crate::ui::retained_host::app::RetainedEditorHost;
use zircon_runtime::asset::pipeline::manager::ProjectAssetGenerationToken;

const ACTIVE_SCENE_RELOAD_NOTIFICATION_PREFIX: &str = "editor.scene.active_reload_conflict";
const ACTIVE_SCENE_RELOAD_SAVE_OPTION: &str = "save";
const ACTIVE_SCENE_RELOAD_DISCARD_OPTION: &str = "discard";
const ACTIVE_SCENE_RELOAD_KEEP_EDITING_OPTION: &str = "keep_editing";

pub(in crate::ui::retained_host::app) struct ActiveSceneReloadConflict {
    pub(super) identity: ActiveSceneDocumentIdentity,
    pub(super) generation: ProjectAssetGenerationToken,
    pub(super) state: ActiveSceneReloadConflictState,
}

pub(in crate::ui::retained_host::app) enum ActiveSceneReloadConflictState {
    AwaitingDecision { ticket: Option<DecisionTicket> },
    DiscardRequested,
    Cancelled,
}

enum ActiveSceneReloadDecisionLookup {
    Missing,
    Pending,
    Resolved(String),
}

impl ActiveSceneReloadConflict {
    pub(super) fn awaiting_decision(
        identity: ActiveSceneDocumentIdentity,
        generation: ProjectAssetGenerationToken,
    ) -> Self {
        Self {
            identity,
            generation,
            state: ActiveSceneReloadConflictState::AwaitingDecision { ticket: None },
        }
    }
}

impl RetainedEditorHost {
    pub(super) fn active_scene_reload_conflict_dirty_policy(
        &mut self,
        identity: &ActiveSceneDocumentIdentity,
        generation: &ProjectAssetGenerationToken,
    ) -> Option<PreparedActiveSceneReloadDirtyPolicy> {
        let Some(mut conflict) = self.active_scene_reload_conflict.take() else {
            return Some(PreparedActiveSceneReloadDirtyPolicy::Reject);
        };
        if conflict.identity != *identity {
            self.dismiss_active_scene_reload_conflict_decision(&conflict);
            return Some(PreparedActiveSceneReloadDirtyPolicy::Reject);
        }

        let same_generation = conflict.generation == *generation;
        match &conflict.state {
            ActiveSceneReloadConflictState::DiscardRequested => {
                if !same_generation {
                    conflict.generation = generation.clone();
                }
                self.active_scene_reload_conflict = Some(conflict);
                Some(PreparedActiveSceneReloadDirtyPolicy::Discard)
            }
            _ if same_generation => {
                self.active_scene_reload_conflict = Some(conflict);
                None
            }
            _ => {
                self.dismiss_active_scene_reload_conflict_decision(&conflict);
                Some(PreparedActiveSceneReloadDirtyPolicy::Reject)
            }
        }
    }

    pub(super) fn reconcile_active_scene_reload_conflict(&mut self) {
        let Some(mut conflict) = self.active_scene_reload_conflict.take() else {
            return;
        };
        if self
            .editor_manager
            .active_scene_identity_for_session()
            .as_ref()
            != Some(&conflict.identity)
        {
            self.dismiss_active_scene_reload_conflict_decision(&conflict);
            return;
        }
        match self.dirty_project_scene_generation() {
            Ok(Some(_)) => {}
            Ok(None) => {
                self.dismiss_active_scene_reload_conflict_decision(&conflict);
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.active_scene_reload_conflict_resolved",
                    1
                );
                self.queue_active_scene_reload_retry();
                return;
            }
            Err(error) => {
                self.active_scene_reload_conflict = Some(conflict);
                self.set_status_line(error);
                return;
            }
        }

        match &conflict.state {
            ActiveSceneReloadConflictState::AwaitingDecision { ticket } => {
                let ticket = ticket.clone();
                let selected_option = match ticket {
                    Some(ticket) => match self.active_scene_reload_decision(&ticket) {
                        Ok(ActiveSceneReloadDecisionLookup::Resolved(option)) => Some(option),
                        Ok(ActiveSceneReloadDecisionLookup::Pending) => {
                            self.active_scene_reload_conflict = Some(conflict);
                            return;
                        }
                        Ok(ActiveSceneReloadDecisionLookup::Missing) => {
                            conflict.state =
                                ActiveSceneReloadConflictState::AwaitingDecision { ticket: None };
                            None
                        }
                        Err(error) => {
                            conflict.state =
                                ActiveSceneReloadConflictState::AwaitingDecision { ticket: None };
                            self.active_scene_reload_conflict = Some(conflict);
                            self.set_status_line(error);
                            return;
                        }
                    },
                    None => None,
                };

                let Some(selected_option) = selected_option else {
                    if let Err(error) =
                        self.publish_active_scene_reload_conflict_decision(&mut conflict)
                    {
                        self.set_status_line(error);
                    }
                    self.active_scene_reload_conflict = Some(conflict);
                    return;
                };
                self.apply_active_scene_reload_conflict_decision(conflict, &selected_option);
            }
            ActiveSceneReloadConflictState::DiscardRequested => {
                self.active_scene_reload_conflict = Some(conflict);
            }
            ActiveSceneReloadConflictState::Cancelled => {
                self.active_scene_reload_conflict = Some(conflict);
            }
        }
    }

    pub(super) fn install_active_scene_reload_conflict(
        &mut self,
        identity: ActiveSceneDocumentIdentity,
        generation: ProjectAssetGenerationToken,
    ) {
        if let Some(previous) = self.active_scene_reload_conflict.take() {
            self.dismiss_active_scene_reload_conflict_decision(&previous);
        }
        self.active_scene_reload_conflict = Some(ActiveSceneReloadConflict::awaiting_decision(
            identity, generation,
        ));
        self.reconcile_active_scene_reload_conflict();
    }

    pub(super) fn restore_active_scene_reload_conflict_after_discard_failure(
        &mut self,
        identity: ActiveSceneDocumentIdentity,
        generation: ProjectAssetGenerationToken,
    ) {
        if self
            .editor_manager
            .active_scene_identity_for_session()
            .as_ref()
            == Some(&identity)
        {
            self.install_active_scene_reload_conflict(identity, generation);
        }
    }

    pub(super) fn clear_active_scene_reload_conflict_for_identity(
        &mut self,
        identity: &ActiveSceneDocumentIdentity,
    ) {
        let Some(conflict) = self.active_scene_reload_conflict.take() else {
            return;
        };
        if conflict.identity == *identity {
            self.dismiss_active_scene_reload_conflict_decision(&conflict);
        } else {
            self.active_scene_reload_conflict = Some(conflict);
        }
    }

    pub(super) fn dismiss_active_scene_reload_conflict(&mut self) {
        if let Some(conflict) = self.active_scene_reload_conflict.take() {
            self.dismiss_active_scene_reload_conflict_decision(&conflict);
        }
    }

    fn active_scene_reload_decision(
        &self,
        ticket: &DecisionTicket,
    ) -> Result<ActiveSceneReloadDecisionLookup, String> {
        let center = self
            .runtime
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        let Some(snapshot) = center
            .snapshot()
            .into_iter()
            .find(|snapshot| snapshot.ticket() == ticket)
        else {
            return Ok(ActiveSceneReloadDecisionLookup::Missing);
        };
        Ok(match snapshot.resolved() {
            Some(receipt) => {
                ActiveSceneReloadDecisionLookup::Resolved(receipt.option_id().as_str().to_owned())
            }
            None => ActiveSceneReloadDecisionLookup::Pending,
        })
    }

    fn publish_active_scene_reload_conflict_decision(
        &mut self,
        conflict: &mut ActiveSceneReloadConflict,
    ) -> Result<(), String> {
        let sequence = self
            .active_scene_reload_decision_sequence
            .checked_add(1)
            .ok_or_else(|| "active scene reload decision sequence is exhausted".to_owned())?;
        let save_option = DecisionOptionId::parse("save").map_err(|error| error.to_string())?;
        let discard_option =
            DecisionOptionId::parse("discard").map_err(|error| error.to_string())?;
        let keep_editing_option =
            DecisionOptionId::parse("keep_editing").map_err(|error| error.to_string())?;
        let notification = DecisionNotification::new(
            NotificationId::parse(format!(
                "{ACTIVE_SCENE_RELOAD_NOTIFICATION_PREFIX}.{sequence}"
            ))
            .map_err(|error| error.to_string())?,
            NotificationSource::builtin("editor.scene").map_err(|error| error.to_string())?,
            "editor.scene.reload_conflict.title",
            "editor.scene.reload_conflict.message",
            vec![
                DecisionOption::new(save_option.clone(), "editor.scene.reload_conflict.save")
                    .map_err(|error| error.to_string())?,
                DecisionOption::new(discard_option, "editor.scene.reload_conflict.discard")
                    .map_err(|error| error.to_string())?,
                DecisionOption::new(
                    keep_editing_option.clone(),
                    "editor.scene.reload_conflict.keep_editing",
                )
                .map_err(|error| error.to_string())?,
            ],
        )
        .map_err(|error| error.to_string())?
        .with_default_option(save_option)
        .map_err(|error| error.to_string())?
        .with_cancel_option(keep_editing_option)
        .map_err(|error| error.to_string())?
        .with_display_subject(active_scene_reload_display_subject(
            conflict.identity.scene_uri(),
        ))
        .map_err(|error| error.to_string())?;
        let ticket = self
            .runtime
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?
            .publish(notification)
            .map_err(|error| error.to_string())?;
        self.active_scene_reload_decision_sequence = sequence;
        conflict.state = ActiveSceneReloadConflictState::AwaitingDecision {
            ticket: Some(ticket),
        };
        self.refresh_activity_notification_presentation();
        Ok(())
    }

    fn apply_active_scene_reload_conflict_decision(
        &mut self,
        mut conflict: ActiveSceneReloadConflict,
        selected_option: &str,
    ) {
        match selected_option {
            ACTIVE_SCENE_RELOAD_SAVE_OPTION => {
                self.save_active_scene_reload_conflict(conflict);
            }
            ACTIVE_SCENE_RELOAD_DISCARD_OPTION => {
                conflict.state = ActiveSceneReloadConflictState::DiscardRequested;
                self.active_scene_reload_conflict = Some(conflict);
                if let Err(error) = self.submit_active_scene_reload(None) {
                    self.set_status_line(error);
                }
            }
            ACTIVE_SCENE_RELOAD_KEEP_EDITING_OPTION => {
                conflict.state = ActiveSceneReloadConflictState::Cancelled;
                self.active_scene_reload_conflict = Some(conflict);
                self.set_status_line(
                    "Kept local scene edits; the current external reload was dismissed.".to_owned(),
                );
            }
            _ => {
                conflict.state = ActiveSceneReloadConflictState::AwaitingDecision { ticket: None };
                self.active_scene_reload_conflict = Some(conflict);
                self.set_status_line(format!(
                    "Unknown active scene reload decision option `{selected_option}`"
                ));
            }
        }
    }

    fn save_active_scene_reload_conflict(&mut self, mut conflict: ActiveSceneReloadConflict) {
        match self.save_project_scene() {
            Ok(()) => match self.dirty_project_scene_generation() {
                Ok(None)
                    if self
                        .editor_manager
                        .active_scene_identity_for_session()
                        .as_ref()
                        == Some(&conflict.identity) =>
                {
                    self.queue_active_scene_reload_retry();
                    self.set_status_line(
                        "Saved local scene edits; reloading the latest scene source.".to_owned(),
                    );
                }
                Ok(None) => {}
                Ok(Some(_)) => {
                    conflict.state =
                        ActiveSceneReloadConflictState::AwaitingDecision { ticket: None };
                    self.active_scene_reload_conflict = Some(conflict);
                    self.set_status_line(
                        "The active scene changed while it was being saved; review the reload decision again."
                            .to_owned(),
                    );
                }
                Err(error) => {
                    conflict.state =
                        ActiveSceneReloadConflictState::AwaitingDecision { ticket: None };
                    self.active_scene_reload_conflict = Some(conflict);
                    self.set_status_line(error);
                }
            },
            Err(error) => {
                conflict.state = ActiveSceneReloadConflictState::AwaitingDecision { ticket: None };
                self.active_scene_reload_conflict = Some(conflict);
                self.set_status_line(format!("The active scene could not be saved: {error}"));
            }
        }
    }

    fn dismiss_active_scene_reload_conflict_decision(
        &mut self,
        conflict: &ActiveSceneReloadConflict,
    ) {
        let ActiveSceneReloadConflictState::AwaitingDecision {
            ticket: Some(ticket),
        } = &conflict.state
        else {
            return;
        };
        if let Ok(center) = self.runtime.context().notifications().decisions() {
            let _ = center.cancel(ticket);
        }
        self.refresh_activity_notification_presentation();
    }
}

fn active_scene_reload_display_subject(scene_uri: &str) -> String {
    if scene_uri.len() <= MAX_DECISION_DISPLAY_SUBJECT_BYTES {
        return scene_uri.to_owned();
    }
    const PREFIX: &str = "...";
    let tail_bytes = MAX_DECISION_DISPLAY_SUBJECT_BYTES.saturating_sub(PREFIX.len());
    let mut tail_start = scene_uri.len().saturating_sub(tail_bytes);
    while !scene_uri.is_char_boundary(tail_start) {
        tail_start += 1;
    }
    format!("{PREFIX}{}", &scene_uri[tail_start..])
}

#[cfg(test)]
mod tests {
    use super::active_scene_reload_display_subject;
    use crate::core::notifications::MAX_DECISION_DISPLAY_SUBJECT_BYTES;

    #[test]
    fn reload_conflict_display_subject_preserves_a_bounded_utf8_tail() {
        let uri = format!("res://scenes/{}final.scene.toml", "场景/".repeat(80));

        let subject = active_scene_reload_display_subject(&uri);

        assert!(subject.len() <= MAX_DECISION_DISPLAY_SUBJECT_BYTES);
        assert!(subject.starts_with("..."));
        assert!(subject.ends_with("final.scene.toml"));
    }
}
