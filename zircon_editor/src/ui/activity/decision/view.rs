use crate::core::i18n::EditorI18nService;
use crate::core::notifications::{DecisionNotificationSnapshot, present_decision};

use super::ActivityDecisionSelectionId;

/// Read-only Activity projection for one option in the current operator Decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityDecisionOption {
    selection_id: ActivityDecisionSelectionId,
    title: String,
    message: String,
}

impl ActivityDecisionOption {
    fn new(
        selection_id: ActivityDecisionSelectionId,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            selection_id,
            title: title.into(),
            message: message.into(),
        }
    }

    pub(crate) fn selection_id(&self) -> &ActivityDecisionSelectionId {
        &self.selection_id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

/// Projects the oldest pending Decision as one complete option group.
///
/// Presenting only the first group prevents a bounded history surface from truncating a later
/// Decision's option set. The core center owns the remaining FIFO backlog.
pub(crate) fn activity_decision_options(
    snapshots: &[DecisionNotificationSnapshot],
    i18n: &EditorI18nService,
) -> Vec<ActivityDecisionOption> {
    let Some(snapshot) = snapshots.first() else {
        return Vec::new();
    };
    let decision = present_decision(snapshot, i18n);
    decision
        .options()
        .iter()
        .map(|option| {
            let message = match decision.display_subject() {
                Some(subject) => format!("{} ({subject}) [{}]", decision.message(), option.label()),
                None => format!("{} [{}]", decision.message(), option.label()),
            };
            ActivityDecisionOption::new(
                ActivityDecisionSelectionId::new(decision.id(), option.id()),
                decision.title(),
                message,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::core::i18n::EditorI18nService;
    use crate::core::notifications::{
        DecisionCenterConfig, DecisionNotification, DecisionNotificationCenter, DecisionOption,
        DecisionOptionId, NotificationId, NotificationSource,
    };

    use super::activity_decision_options;

    fn publish(center: &DecisionNotificationCenter, id: &str, title_key: &str, message_key: &str) {
        center
            .publish(
                DecisionNotification::new(
                    NotificationId::parse(id).expect("test notification id should be valid"),
                    NotificationSource::builtin("editor.test")
                        .expect("test notification source should be valid"),
                    title_key,
                    message_key,
                    vec![
                        DecisionOption::new(
                            DecisionOptionId::parse("apply")
                                .expect("apply option id should be valid"),
                            "editor.play.pending_edits.apply",
                        )
                        .expect("apply option should construct"),
                        DecisionOption::new(
                            DecisionOptionId::parse("discard")
                                .expect("discard option id should be valid"),
                            "editor.play.pending_edits.discard",
                        )
                        .expect("discard option should construct"),
                    ],
                )
                .expect("test decision should construct"),
            )
            .expect("test decision should publish");
    }

    #[test]
    fn activity_projection_keeps_the_oldest_decision_complete() {
        let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
            .expect("test decision center should construct");
        publish(
            &center,
            "editor.decision.first",
            "editor.play.pending_edits.title",
            "editor.play.pending_edits.message",
        );
        publish(
            &center,
            "editor.decision.second",
            "editor.play.pending_edits.title",
            "editor.play.pending_edits.message",
        );

        let options =
            activity_decision_options(&center.pending_snapshot(), &EditorI18nService::default());

        assert_eq!(options.len(), 2);
        assert_eq!(
            options[0].selection_id().as_str(),
            "editor.decision.first:apply"
        );
        assert_eq!(
            options[1].selection_id().as_str(),
            "editor.decision.first:discard"
        );
        assert!(
            options
                .iter()
                .all(|option| option.title() == "Resolve queued play edits")
        );
    }

    #[test]
    fn activity_projection_keeps_optional_decision_subject_with_each_option() {
        let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
            .expect("test decision center should construct");
        center
            .publish(
                DecisionNotification::new(
                    NotificationId::parse("editor.recovery.candidate")
                        .expect("test notification id should be valid"),
                    NotificationSource::builtin("editor.recovery")
                        .expect("test notification source should be valid"),
                    "editor.play.pending_edits.title",
                    "editor.play.pending_edits.message",
                    vec![
                        DecisionOption::new(
                            DecisionOptionId::parse("apply")
                                .expect("apply option id should be valid"),
                            "editor.play.pending_edits.apply",
                        )
                        .expect("apply option should construct"),
                        DecisionOption::new(
                            DecisionOptionId::parse("discard")
                                .expect("discard option id should be valid"),
                            "editor.play.pending_edits.discard",
                        )
                        .expect("discard option should construct"),
                    ],
                )
                .expect("test decision should construct")
                .with_display_subject("assets/scenes/main.zscene")
                .expect("test subject should construct"),
            )
            .expect("test decision should publish");

        let options =
            activity_decision_options(&center.pending_snapshot(), &EditorI18nService::default());

        assert_eq!(options.len(), 2);
        assert!(
            options
                .iter()
                .all(|option| option.message().contains("assets/scenes/main.zscene"))
        );
    }
}
