use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::{UiNodeId, UiTreeId};

pub const UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION: u32 = 1;

/// Opaque identity used to correlate one numeric model update and its receipt.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiNumberModelUpdateId(Uuid);

impl UiNumberModelUpdateId {
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

/// Manager-owned identity for one NumberField canonical value authority.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiNumberModelId(Uuid);

impl UiNumberModelId {
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

/// Monotonic revision of a NumberField canonical Float value.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiNumberModelRevision(u64);

impl UiNumberModelRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_supported(self) -> bool {
        self.0 <= i64::MAX as u64
    }
}

/// Compare-and-swap key for one NumberField canonical value revision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNumberModelKey {
    pub model_id: UiNumberModelId,
    pub revision: UiNumberModelRevision,
}

impl UiNumberModelKey {
    pub fn validate(self) -> Result<(), UiNumberModelUpdateFailure> {
        if !self.model_id.is_valid() {
            return Err(UiNumberModelUpdateFailure::InvalidModelId);
        }
        if !self.revision.is_supported() {
            return Err(UiNumberModelUpdateFailure::RevisionOutOfRange);
        }
        Ok(())
    }
}

/// Declares whether an update preserves or replaces a focused numeric edit buffer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberModelUpdateOrigin {
    /// A bound model refresh that updates canonical value without replacing focused text.
    #[default]
    BoundRefresh,
    /// An explicit typed replacement that closes an active edit buffer.
    ExplicitSetValue,
}

/// Versioned Float model update qualified by an expected numeric model revision.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UiNumberModelUpdateRequest {
    pub schema_version: u32,
    pub request_id: UiNumberModelUpdateId,
    pub tree_id: UiTreeId,
    pub node_id: UiNodeId,
    pub expected_model: UiNumberModelKey,
    pub origin: UiNumberModelUpdateOrigin,
    pub value: f64,
}

impl UiNumberModelUpdateRequest {
    pub fn new(
        tree_id: UiTreeId,
        node_id: UiNodeId,
        expected_model: UiNumberModelKey,
        origin: UiNumberModelUpdateOrigin,
        value: f64,
    ) -> Self {
        Self {
            schema_version: UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
            request_id: UiNumberModelUpdateId::issue(),
            tree_id,
            node_id,
            expected_model,
            origin,
            value,
        }
    }

    pub fn validate(&self) -> Result<(), UiNumberModelUpdateFailure> {
        if self.schema_version != UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION {
            return Err(UiNumberModelUpdateFailure::UnsupportedSchemaVersion);
        }
        if !self.request_id.is_valid() {
            return Err(UiNumberModelUpdateFailure::InvalidRequestId);
        }
        self.expected_model.validate()?;
        if !self.value.is_finite() {
            return Err(UiNumberModelUpdateFailure::NonFiniteValue);
        }
        Ok(())
    }
}

impl fmt::Debug for UiNumberModelUpdateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiNumberModelUpdateRequest")
            .field("schema_version", &self.schema_version)
            .field("request_id", &self.request_id)
            .field("tree_id", &self.tree_id)
            .field("node_id", &self.node_id)
            .field("expected_model", &self.expected_model)
            .field("origin", &self.origin)
            .field("value_is_finite", &self.value.is_finite())
            .finish()
    }
}

/// Immediate outcome of a numeric model update request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberModelUpdateStatus {
    Applied,
    Unchanged,
    Conflict,
    #[default]
    Rejected,
}

/// Stable fail-closed reason carried by a content-free numeric model receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberModelUpdateFailure {
    UnsupportedSchemaVersion,
    InvalidRequestId,
    InvalidModelId,
    RevisionOutOfRange,
    NonFiniteValue,
    WrongTree,
    InvalidTarget,
    StaleModel,
    PropertyRejected,
    RevisionExhausted,
}

/// Content-free acknowledgement for one numeric model update request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNumberModelUpdateReceipt {
    pub schema_version: u32,
    pub request_id: UiNumberModelUpdateId,
    pub tree_id: UiTreeId,
    pub node_id: UiNodeId,
    pub origin: UiNumberModelUpdateOrigin,
    pub status: UiNumberModelUpdateStatus,
    pub expected_model: UiNumberModelKey,
    pub current_model: Option<UiNumberModelKey>,
    pub failure: Option<UiNumberModelUpdateFailure>,
}

impl UiNumberModelUpdateReceipt {
    pub fn validate(&self) -> Result<(), UiNumberModelUpdateFailure> {
        if self.schema_version != UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION {
            return Err(UiNumberModelUpdateFailure::UnsupportedSchemaVersion);
        }
        let rejection_is = |failure| {
            self.status == UiNumberModelUpdateStatus::Rejected && self.failure == Some(failure)
        };
        let rejects_unsupported =
            rejection_is(UiNumberModelUpdateFailure::UnsupportedSchemaVersion);
        let rejects_invalid_request = rejection_is(UiNumberModelUpdateFailure::InvalidRequestId);
        let rejects_invalid_model = rejection_is(UiNumberModelUpdateFailure::InvalidModelId);
        let rejects_revision_range = rejection_is(UiNumberModelUpdateFailure::RevisionOutOfRange);
        if !self.request_id.is_valid() && !(rejects_unsupported || rejects_invalid_request)
            || (self.request_id.is_valid() && rejects_invalid_request)
        {
            return Err(UiNumberModelUpdateFailure::InvalidRequestId);
        }
        if !self.expected_model.model_id.is_valid()
            && !(rejects_unsupported || rejects_invalid_request || rejects_invalid_model)
            || (self.expected_model.model_id.is_valid() && rejects_invalid_model)
        {
            return Err(UiNumberModelUpdateFailure::InvalidModelId);
        }
        if !self.expected_model.revision.is_supported()
            && !(rejects_unsupported
                || rejects_invalid_request
                || rejects_invalid_model
                || rejects_revision_range)
            || (self.expected_model.revision.is_supported() && rejects_revision_range)
        {
            return Err(UiNumberModelUpdateFailure::RevisionOutOfRange);
        }
        if let Some(current) = self.current_model {
            current.validate()?;
        }
        let valid_status = match self.status {
            UiNumberModelUpdateStatus::Applied => {
                self.failure.is_none() && self.applied_key_valid()
            }
            UiNumberModelUpdateStatus::Unchanged => {
                self.failure.is_none() && self.current_model == Some(self.expected_model)
            }
            UiNumberModelUpdateStatus::Conflict => {
                self.failure == Some(UiNumberModelUpdateFailure::StaleModel)
                    && self
                        .current_model
                        .is_some_and(|current| current != self.expected_model)
            }
            UiNumberModelUpdateStatus::Rejected => self.failure.is_some(),
        };
        if !valid_status {
            return Err(UiNumberModelUpdateFailure::PropertyRejected);
        }
        Ok(())
    }

    fn applied_key_valid(&self) -> bool {
        self.current_model.is_some_and(|current| {
            current.model_id == self.expected_model.model_id
                && (current.revision == self.expected_model.revision
                    || self
                        .expected_model
                        .revision
                        .get()
                        .checked_add(1)
                        .is_some_and(|next| current.revision.get() == next))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_validation_rejects_non_finite_values_and_debug_omits_value() {
        let key = UiNumberModelKey {
            model_id: UiNumberModelId::issue(),
            revision: UiNumberModelRevision::new(3),
        };
        let mut request = UiNumberModelUpdateRequest::new(
            UiTreeId::new("number.model.update"),
            UiNodeId::new(7),
            key,
            UiNumberModelUpdateOrigin::BoundRefresh,
            12345.5,
        );
        assert_eq!(request.validate(), Ok(()));
        assert!(!format!("{request:?}").contains("12345.5"));

        request.value = f64::NAN;
        assert_eq!(
            request.validate(),
            Err(UiNumberModelUpdateFailure::NonFiniteValue)
        );
    }

    #[test]
    fn receipt_validation_requires_current_key_for_terminal_success() {
        let key = UiNumberModelKey {
            model_id: UiNumberModelId::issue(),
            revision: UiNumberModelRevision::new(3),
        };
        let mut receipt = UiNumberModelUpdateReceipt {
            schema_version: UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
            request_id: UiNumberModelUpdateId::issue(),
            tree_id: UiTreeId::new("number.model.update"),
            node_id: UiNodeId::new(7),
            origin: UiNumberModelUpdateOrigin::BoundRefresh,
            status: UiNumberModelUpdateStatus::Unchanged,
            expected_model: key,
            current_model: Some(key),
            failure: None,
        };
        assert_eq!(receipt.validate(), Ok(()));

        receipt.current_model = None;
        assert_eq!(
            receipt.validate(),
            Err(UiNumberModelUpdateFailure::PropertyRejected)
        );
    }

    #[test]
    fn receipt_validation_rejects_inconsistent_failures_and_revision_relations() {
        let key = UiNumberModelKey {
            model_id: UiNumberModelId::issue(),
            revision: UiNumberModelRevision::new(3),
        };
        let mut receipt = UiNumberModelUpdateReceipt {
            schema_version: UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
            request_id: UiNumberModelUpdateId::issue(),
            tree_id: UiTreeId::new("number.model.update"),
            node_id: UiNodeId::new(7),
            origin: UiNumberModelUpdateOrigin::BoundRefresh,
            status: UiNumberModelUpdateStatus::Rejected,
            expected_model: key,
            current_model: Some(key),
            failure: Some(UiNumberModelUpdateFailure::InvalidRequestId),
        };
        assert_eq!(
            receipt.validate(),
            Err(UiNumberModelUpdateFailure::InvalidRequestId)
        );

        receipt.status = UiNumberModelUpdateStatus::Applied;
        receipt.failure = None;
        receipt.current_model = Some(UiNumberModelKey {
            revision: UiNumberModelRevision::new(5),
            ..key
        });
        assert_eq!(
            receipt.validate(),
            Err(UiNumberModelUpdateFailure::PropertyRejected)
        );

        receipt.status = UiNumberModelUpdateStatus::Conflict;
        receipt.failure = Some(UiNumberModelUpdateFailure::StaleModel);
        receipt.current_model = Some(key);
        assert_eq!(
            receipt.validate(),
            Err(UiNumberModelUpdateFailure::PropertyRejected)
        );
    }
}
