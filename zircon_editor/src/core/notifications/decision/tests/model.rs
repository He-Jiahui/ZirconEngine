use super::*;

fn notification() -> DecisionNotification {
    DecisionNotification::new(
        NotificationId::parse("editor.test.decision").unwrap(),
        NotificationSource::builtin("editor.test").unwrap(),
        "editor.test.title",
        "editor.test.message",
        vec![
            DecisionOption::new(DecisionOptionId::parse("apply").unwrap(), "editor.apply").unwrap(),
            DecisionOption::new(
                DecisionOptionId::parse("discard").unwrap(),
                "editor.discard",
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

#[test]
fn message_arguments_are_bounded_named_facts() {
    assert!(matches!(
        notification().with_message_argument("invalid-name", 1),
        Err(DecisionNotificationError::InvalidMessageArgumentName { .. })
    ));

    let duplicate = notification()
        .with_message_argument("pending_count", 1)
        .unwrap()
        .with_message_argument("pending_count", 2);
    assert!(matches!(
        duplicate,
        Err(DecisionNotificationError::DuplicateMessageArgument { .. })
    ));

    let mut bounded = notification();
    for name in [
        "one", "two", "three", "four", "five", "six", "seven", "eight",
    ] {
        bounded = bounded.with_message_argument(name, 1).unwrap();
    }
    assert!(matches!(
        bounded.with_message_argument("nine", 1),
        Err(DecisionNotificationError::TooManyMessageArguments {
            maximum: 8,
            actual: 9
        })
    ));
}

#[test]
fn display_subject_is_bounded_optional_operator_context() {
    let with_subject = notification()
        .with_display_subject("assets/scenes/main.zscene")
        .unwrap();

    assert_eq!(
        with_subject.display_subject(),
        Some("assets/scenes/main.zscene")
    );
    assert!(matches!(
        notification().with_display_subject("   "),
        Err(DecisionNotificationError::EmptyField {
            field: "display subject"
        })
    ));
    assert!(matches!(
        notification().with_display_subject("a".repeat(MAX_DECISION_DISPLAY_SUBJECT_BYTES + 1)),
        Err(DecisionNotificationError::FieldTooLong {
            field: "display subject",
            maximum: MAX_DECISION_DISPLAY_SUBJECT_BYTES,
            ..
        })
    ));
}
