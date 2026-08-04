use std::collections::BTreeSet;
use std::sync::Arc;

use crate::core::notifications::{NotificationId, NotificationSource};

use super::{DecisionNotificationError, DecisionOptionId, DecisionReceipt, DecisionTicket};

pub const MAX_DECISION_OPTIONS: usize = 16;
pub const MAX_LOCALIZATION_KEY_BYTES: usize = 256;
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
