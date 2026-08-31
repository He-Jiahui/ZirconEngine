use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::{UiTextByteRange, UiTextCaretAffinity};

mod model_update;
mod number_model_update;
mod rich_link_target;

pub use model_update::{
    UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION, UiTextDocumentKey, UiTextModelUpdateFailure,
    UiTextModelUpdateId, UiTextModelUpdateOrigin, UiTextModelUpdateReceipt,
    UiTextModelUpdateRequest, UiTextModelUpdateStatus,
};
pub use number_model_update::{
    UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION, UiNumberModelId, UiNumberModelKey,
    UiNumberModelRevision, UiNumberModelUpdateFailure, UiNumberModelUpdateId,
    UiNumberModelUpdateOrigin, UiNumberModelUpdateReceipt, UiNumberModelUpdateRequest,
    UiNumberModelUpdateStatus,
};
pub use rich_link_target::{UiRichLinkTarget, UiRichLinkTargetError};

pub const UI_TEXT_EDIT_RECEIPT_SCHEMA_VERSION: u32 = 1;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiTextDocumentId(Uuid);

impl UiTextDocumentId {
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiTextDocumentRevision(u64);

impl UiTextDocumentRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextEditSource {
    #[default]
    Keyboard,
    Pointer,
    Ime,
    Clipboard,
    Accessibility,
    Programmatic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextEditKind {
    Insert,
    Delete,
    Replace,
    CompositionCommit,
    Undo,
    Redo,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextChangedRanges {
    pub old: UiTextByteRange,
    pub new: UiTextByteRange,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextByteSelection {
    pub anchor_byte: u32,
    pub focus_byte: u32,
    pub focus_affinity: UiTextCaretAffinity,
}

impl UiTextByteSelection {
    pub const fn collapsed(offset: u32) -> Self {
        Self::collapsed_with_affinity(offset, UiTextCaretAffinity::Downstream)
    }

    pub const fn collapsed_with_affinity(offset: u32, focus_affinity: UiTextCaretAffinity) -> Self {
        Self {
            anchor_byte: offset,
            focus_byte: offset,
            focus_affinity,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UiTextEditReceiptWire")]
pub struct UiTextEditReceipt {
    pub schema_version: u32,
    pub node_id: UiNodeId,
    pub document_id: UiTextDocumentId,
    pub previous_revision: UiTextDocumentRevision,
    pub revision: UiTextDocumentRevision,
    pub source: UiTextEditSource,
    pub kind: UiTextEditKind,
    pub changed: UiTextChangedRanges,
    pub selection: UiTextByteSelection,
}

#[derive(Deserialize)]
struct UiTextEditReceiptWire {
    schema_version: u32,
    node_id: UiNodeId,
    document_id: UiTextDocumentId,
    previous_revision: UiTextDocumentRevision,
    revision: UiTextDocumentRevision,
    source: UiTextEditSource,
    kind: UiTextEditKind,
    changed: UiTextChangedRanges,
    selection: UiTextByteSelection,
}

impl TryFrom<UiTextEditReceiptWire> for UiTextEditReceipt {
    type Error = UiTextEditReceiptError;

    fn try_from(wire: UiTextEditReceiptWire) -> Result<Self, Self::Error> {
        let receipt = Self {
            schema_version: wire.schema_version,
            node_id: wire.node_id,
            document_id: wire.document_id,
            previous_revision: wire.previous_revision,
            revision: wire.revision,
            source: wire.source,
            kind: wire.kind,
            changed: wire.changed,
            selection: wire.selection,
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

impl UiTextEditReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: UiNodeId,
        document_id: UiTextDocumentId,
        previous_revision: UiTextDocumentRevision,
        source: UiTextEditSource,
        kind: UiTextEditKind,
        changed: UiTextChangedRanges,
        selection: UiTextByteSelection,
    ) -> Result<Self, UiTextEditReceiptError> {
        let revision = previous_revision
            .checked_next()
            .ok_or(UiTextEditReceiptError::RevisionExhausted)?;
        let receipt = Self {
            schema_version: UI_TEXT_EDIT_RECEIPT_SCHEMA_VERSION,
            node_id,
            document_id,
            previous_revision,
            revision,
            source,
            kind,
            changed,
            selection,
        };
        receipt.validate()?;
        Ok(receipt)
    }

    pub fn validate(&self) -> Result<(), UiTextEditReceiptError> {
        if self.schema_version != UI_TEXT_EDIT_RECEIPT_SCHEMA_VERSION {
            return Err(UiTextEditReceiptError::UnsupportedSchemaVersion);
        }
        if !self.document_id.is_valid() {
            return Err(UiTextEditReceiptError::InvalidDocumentId);
        }
        let Some(expected_revision) = self.previous_revision.checked_next() else {
            return Err(UiTextEditReceiptError::RevisionExhausted);
        };
        if expected_revision != self.revision {
            return Err(UiTextEditReceiptError::NonConsecutiveRevision);
        }
        if self.changed.old.start_byte > self.changed.old.end_byte {
            return Err(UiTextEditReceiptError::InvalidOldRange);
        }
        if self.changed.new.start_byte > self.changed.new.end_byte {
            return Err(UiTextEditReceiptError::InvalidNewRange);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextEditReceiptError {
    UnsupportedSchemaVersion,
    InvalidDocumentId,
    RevisionExhausted,
    NonConsecutiveRevision,
    InvalidOldRange,
    InvalidNewRange,
}

impl fmt::Display for UiTextEditReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedSchemaVersion => "unsupported text edit receipt schema version",
            Self::InvalidDocumentId => "text edit receipt has an invalid document id",
            Self::RevisionExhausted => "text edit receipt revision is exhausted",
            Self::NonConsecutiveRevision => "text edit receipt revision is not consecutive",
            Self::InvalidOldRange => "text edit receipt old range is out of order",
            Self::InvalidNewRange => "text edit receipt new range is out of order",
        })
    }
}

impl std::error::Error for UiTextEditReceiptError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextCursorStyle {
    pub width: f32,
    pub color: Option<String>,
    pub blink_period_millis: Option<u64>,
    pub visible: bool,
}

impl Default for UiTextCursorStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: None,
            blink_period_millis: None,
            visible: true,
        }
    }
}
