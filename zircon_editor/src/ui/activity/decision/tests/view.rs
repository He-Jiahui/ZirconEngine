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
                        DecisionOptionId::parse("apply").expect("apply option id should be valid"),
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
    assert!(options
        .iter()
        .all(|option| option.title() == "Resolve queued play edits"));
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
                        DecisionOptionId::parse("apply").expect("apply option id should be valid"),
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
    assert!(options
        .iter()
        .all(|option| option.message().contains("assets/scenes/main.zscene")));
}
