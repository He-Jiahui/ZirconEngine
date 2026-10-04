use super::*;

#[test]
fn selection_identifier_round_trips_only_valid_core_ids() {
    let notification_id = NotificationId::parse("editor.activity.recovery").unwrap();
    let option_id = DecisionOptionId::parse("restore").unwrap();
    let identifier = ActivityDecisionSelectionId::new(&notification_id, &option_id);

    assert_eq!(identifier.as_str(), "editor.activity.recovery:restore");
    assert_eq!(
        identifier.selection().unwrap().notification_id(),
        &notification_id
    );
    assert_eq!(identifier.selection().unwrap().option_id(), &option_id);
    assert!(ActivityDecisionSelectionId::parse("editor.activity.recovery:restore:extra").is_err());
    assert!(ActivityDecisionSelectionId::parse("editor.activity.recovery:bad-option").is_err());
}
