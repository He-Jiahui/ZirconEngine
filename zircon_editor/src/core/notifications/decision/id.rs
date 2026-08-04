use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::core::notifications::NotificationId;

use super::DecisionNotificationError;

pub const MAX_DECISION_OPTION_ID_BYTES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionOptionId(Arc<str>);

impl DecisionOptionId {
    pub fn parse(value: impl Into<String>) -> Result<Self, DecisionNotificationError> {
        let value = value.into();
        require_max_bytes("decision option id", &value, MAX_DECISION_OPTION_ID_BYTES)?;
        if !valid_segment(&value) {
            return Err(DecisionNotificationError::InvalidOptionId(value));
        }
        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DecisionOptionId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionCenterInstanceId(u64);

impl DecisionCenterInstanceId {
    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionTicket {
    center_instance: DecisionCenterInstanceId,
    notification_id: NotificationId,
    incarnation: u64,
}

impl DecisionTicket {
    pub(super) const fn new(
        center_instance: DecisionCenterInstanceId,
        notification_id: NotificationId,
        incarnation: u64,
    ) -> Self {
        Self {
            center_instance,
            notification_id,
            incarnation,
        }
    }

    pub const fn center_instance(&self) -> DecisionCenterInstanceId {
        self.center_instance
    }

    pub const fn notification_id(&self) -> &NotificationId {
        &self.notification_id
    }

    pub const fn incarnation(&self) -> u64 {
        self.incarnation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionReceiptSequence(u64);

impl DecisionReceiptSequence {
    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionReceiptCursor {
    center_instance: DecisionCenterInstanceId,
    value: u64,
}

impl DecisionReceiptCursor {
    pub(super) const fn start(center_instance: DecisionCenterInstanceId) -> Self {
        Self {
            center_instance,
            value: 0,
        }
    }

    pub(super) const fn after(
        center_instance: DecisionCenterInstanceId,
        sequence: DecisionReceiptSequence,
    ) -> Self {
        Self {
            center_instance,
            value: sequence.value(),
        }
    }

    pub(super) const fn before(
        center_instance: DecisionCenterInstanceId,
        sequence: DecisionReceiptSequence,
    ) -> Self {
        Self {
            center_instance,
            value: sequence.value().saturating_sub(1),
        }
    }

    pub const fn center_instance(self) -> DecisionCenterInstanceId {
        self.center_instance
    }

    pub const fn value(self) -> u64 {
        self.value
    }
}

fn require_max_bytes(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), DecisionNotificationError> {
    if value.len() > maximum {
        Err(DecisionNotificationError::FieldTooLong {
            field,
            maximum,
            actual: value.len(),
        })
    } else {
        Ok(())
    }
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}
