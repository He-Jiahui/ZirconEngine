use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::core::notifications::{NotificationId, NotificationSource};

use super::{DecisionNotificationError, DecisionOptionId, DecisionReceipt, DecisionTicket};

pub const MAX_DECISION_OPTIONS: usize = 16;
pub const MAX_LOCALIZATION_KEY_BYTES: usize = 256;
/// Maximum UTF-8 bytes retained for non-localized, operator-specific display context.
///
/// Identity and action routing remain typed IDs. This field is display-only, for example a
/// project-relative document name selected by a recovery producer.
pub const MAX_DECISION_DISPLAY_SUBJECT_BYTES: usize = 256;
const MAX_DECISION_MESSAGE_ARGUMENTS: usize = 8;
const MAX_DECISION_MESSAGE_ARGUMENT_NAME_BYTES: usize = 64;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionOption {
    id: DecisionOptionId,
    label_key: Arc<str>,
}

impl DecisionOption {
    pub fn new(
        id: DecisionOptionId,
        label_key: impl Into<String>,
    ) -> Result<Self, DecisionNotificationError> {
        Ok(Self {
            id,
            label_key: bounded_non_empty(
                "option label key",
                label_key,
                MAX_LOCALIZATION_KEY_BYTES,
            )?,
        })
    }

    pub fn id(&self) -> &DecisionOptionId {
        &self.id
    }

    pub fn label_key(&self) -> &str {
        &self.label_key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionNotification(Arc<DecisionNotificationData>);

#[derive(Clone, Debug, PartialEq, Eq)]
struct DecisionNotificationData {
    id: NotificationId,
    source: NotificationSource,
    title_key: Arc<str>,
    message_key: Arc<str>,
    display_subject: Option<Arc<str>>,
    message_arguments: BTreeMap<&'static str, u64>,
    options: Vec<DecisionOption>,
    default_option: Option<DecisionOptionId>,
    cancel_option: Option<DecisionOptionId>,
}

impl DecisionNotification {
    pub fn new(
        id: NotificationId,
        source: NotificationSource,
        title_key: impl Into<String>,
        message_key: impl Into<String>,
        options: Vec<DecisionOption>,
    ) -> Result<Self, DecisionNotificationError> {
        if options.len() < 2 {
            return Err(DecisionNotificationError::AtLeastTwoOptionsRequired);
        }
        if options.len() > MAX_DECISION_OPTIONS {
            return Err(DecisionNotificationError::TooManyOptions {
                maximum: MAX_DECISION_OPTIONS,
                actual: options.len(),
            });
        }
        let mut option_ids = BTreeSet::new();
        for option in &options {
            if !option_ids.insert(option.id().clone()) {
                return Err(DecisionNotificationError::DuplicateOption {
                    option: option.id().clone(),
                });
            }
        }
        Ok(Self(Arc::new(DecisionNotificationData {
            id,
            source,
            title_key: bounded_non_empty("title key", title_key, MAX_LOCALIZATION_KEY_BYTES)?,
            message_key: bounded_non_empty("message key", message_key, MAX_LOCALIZATION_KEY_BYTES)?,
            display_subject: None,
            message_arguments: BTreeMap::new(),
            options,
            default_option: None,
            cancel_option: None,
        })))
    }

    pub fn with_default_option(
        mut self,
        option: DecisionOptionId,
    ) -> Result<Self, DecisionNotificationError> {
        self.require_option(&option)?;
        Arc::make_mut(&mut self.0).default_option = Some(option);
        Ok(self)
    }

    pub fn with_cancel_option(
        mut self,
        option: DecisionOptionId,
    ) -> Result<Self, DecisionNotificationError> {
        self.require_option(&option)?;
        Arc::make_mut(&mut self.0).cancel_option = Some(option);
        Ok(self)
    }

    /// Adds bounded operator context that presentation may place beside localized text.
    ///
    /// This is not a localization argument and must never be used as a command or persistence
    /// identity. Producers should pass a project-relative, privacy-safe label rather than an
    /// absolute filesystem path.
    pub fn with_display_subject(
        mut self,
        subject: impl Into<String>,
    ) -> Result<Self, DecisionNotificationError> {
        Arc::make_mut(&mut self.0).display_subject = Some(bounded_non_empty(
            "display subject",
            subject,
            MAX_DECISION_DISPLAY_SUBJECT_BYTES,
        )?);
        Ok(self)
    }

    /// Adds one bounded, immutable fact for message-key placeholder formatting.
    pub fn with_message_argument(
        mut self,
        name: &'static str,
        value: u64,
    ) -> Result<Self, DecisionNotificationError> {
        if !valid_message_argument_name(name) {
            return Err(DecisionNotificationError::InvalidMessageArgumentName { name });
        }
        let data = Arc::make_mut(&mut self.0);
        if data.message_arguments.contains_key(name) {
            return Err(DecisionNotificationError::DuplicateMessageArgument { name });
        }
        if data.message_arguments.len() >= MAX_DECISION_MESSAGE_ARGUMENTS {
            return Err(DecisionNotificationError::TooManyMessageArguments {
                maximum: MAX_DECISION_MESSAGE_ARGUMENTS,
                actual: data.message_arguments.len() + 1,
            });
        }
        data.message_arguments.insert(name, value);
        Ok(self)
    }

    pub fn has_option(&self, option: &DecisionOptionId) -> bool {
        self.0
            .options
            .iter()
            .any(|candidate| candidate.id() == option)
    }

    pub fn id(&self) -> &NotificationId {
        &self.0.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.0.source
    }

    pub fn title_key(&self) -> &str {
        &self.0.title_key
    }

    pub fn message_key(&self) -> &str {
        &self.0.message_key
    }

    pub fn display_subject(&self) -> Option<&str> {
        self.0.display_subject.as_deref()
    }

    pub fn message_arguments(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        self.0
            .message_arguments
            .iter()
            .map(|(&name, &value)| (name, value))
    }

    pub fn options(&self) -> &[DecisionOption] {
        &self.0.options
    }

    pub fn default_option(&self) -> Option<&DecisionOptionId> {
        self.0.default_option.as_ref()
    }

    pub fn cancel_option(&self) -> Option<&DecisionOptionId> {
        self.0.cancel_option.as_ref()
    }

    fn require_option(&self, option: &DecisionOptionId) -> Result<(), DecisionNotificationError> {
        if self.has_option(option) {
            Ok(())
        } else {
            Err(DecisionNotificationError::OptionNotFound {
                notification: self.0.id.clone(),
                option: option.clone(),
            })
        }
    }
}

fn valid_message_argument_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_DECISION_MESSAGE_ARGUMENT_NAME_BYTES
        && name.as_bytes()[0].is_ascii_lowercase()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionNotificationSnapshot {
    ticket: DecisionTicket,
    notification: DecisionNotification,
    resolved: Option<DecisionReceipt>,
}

impl DecisionNotificationSnapshot {
    pub(super) fn new(
        ticket: DecisionTicket,
        notification: DecisionNotification,
        resolved: Option<DecisionReceipt>,
    ) -> Self {
        Self {
            ticket,
            notification,
            resolved,
        }
    }

    pub fn ticket(&self) -> &DecisionTicket {
        &self.ticket
    }

    pub fn notification(&self) -> &DecisionNotification {
        &self.notification
    }

    pub fn resolved(&self) -> Option<&DecisionReceipt> {
        self.resolved.as_ref()
    }
}

fn bounded_non_empty(
    field: &'static str,
    value: impl Into<String>,
    maximum: usize,
) -> Result<Arc<str>, DecisionNotificationError> {
    let value = value.into();
    if value.trim().is_empty() {
        Err(DecisionNotificationError::EmptyField { field })
    } else if value.len() > maximum {
        Err(DecisionNotificationError::FieldTooLong {
            field,
            maximum,
            actual: value.len(),
        })
    } else {
        Ok(Arc::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification() -> DecisionNotification {
        DecisionNotification::new(
            NotificationId::parse("editor.test.decision").unwrap(),
            NotificationSource::builtin("editor.test").unwrap(),
            "editor.test.title",
            "editor.test.message",
            vec![
                DecisionOption::new(DecisionOptionId::parse("apply").unwrap(), "editor.apply")
                    .unwrap(),
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
        let notification = notification()
            .with_display_subject("assets/scenes/main.zscene")
            .unwrap();

        assert_eq!(
            notification.display_subject(),
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
}
