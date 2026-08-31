use thiserror::Error;

use crate::core::notifications::{
    DecisionOptionId, MAX_DECISION_OPTION_ID_BYTES, MAX_NOTIFICATION_ID_BYTES, NotificationId,
};

const SEPARATOR: char = ':';
const MAX_ACTIVITY_DECISION_SELECTION_ID_BYTES: usize =
    MAX_NOTIFICATION_ID_BYTES + SEPARATOR.len_utf8() + MAX_DECISION_OPTION_ID_BYTES;

/// Canonical UI route identity for one currently pending core Decision option.
///
/// The identifier carries stable core ids only. It intentionally omits ticket incarnation and is
/// re-bound to the current core snapshot immediately before resolution.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActivityDecisionSelectionId(String);

impl ActivityDecisionSelectionId {
    pub(crate) fn new(notification_id: &NotificationId, option_id: &DecisionOptionId) -> Self {
        Self(format!("{notification_id}{SEPARATOR}{option_id}"))
    }

    pub(crate) fn parse(value: &str) -> Result<Self, ActivityDecisionSelectionError> {
        if value.len() > MAX_ACTIVITY_DECISION_SELECTION_ID_BYTES {
            return Err(ActivityDecisionSelectionError::InvalidIdentifier);
        }
        ActivityDecisionSelection::parse(value)?;
        Ok(Self(value.to_string()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn selection(
        &self,
    ) -> Result<ActivityDecisionSelection, ActivityDecisionSelectionError> {
        ActivityDecisionSelection::parse(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityDecisionSelection {
    notification_id: NotificationId,
    option_id: DecisionOptionId,
}

impl ActivityDecisionSelection {
    fn parse(value: &str) -> Result<Self, ActivityDecisionSelectionError> {
        let Some((notification_id, option_id)) = value.split_once(SEPARATOR) else {
            return Err(ActivityDecisionSelectionError::InvalidIdentifier);
        };
        if option_id.contains(SEPARATOR) {
            return Err(ActivityDecisionSelectionError::InvalidIdentifier);
        }
        let notification_id = NotificationId::parse(notification_id)
            .map_err(|_| ActivityDecisionSelectionError::InvalidIdentifier)?;
        let option_id = DecisionOptionId::parse(option_id)
            .map_err(|_| ActivityDecisionSelectionError::InvalidIdentifier)?;
        Ok(Self {
            notification_id,
            option_id,
        })
    }

    pub(crate) fn notification_id(&self) -> &NotificationId {
        &self.notification_id
    }

    pub(crate) fn option_id(&self) -> &DecisionOptionId {
        &self.option_id
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum ActivityDecisionSelectionError {
    #[error("activity decision selection identifier is invalid")]
    InvalidIdentifier,
}

#[cfg(test)]
mod tests {
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
        assert!(
            ActivityDecisionSelectionId::parse("editor.activity.recovery:restore:extra").is_err()
        );
        assert!(ActivityDecisionSelectionId::parse("editor.activity.recovery:bad-option").is_err());
    }
}
