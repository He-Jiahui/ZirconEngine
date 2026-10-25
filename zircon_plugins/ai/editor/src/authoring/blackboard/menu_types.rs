use thiserror::Error;
use zircon_runtime::core::framework::ai::{
    AiBlackboardKeyDescriptor, AiBlackboardValueType, AiManagerError,
};

use super::schema_edit::AiBlackboardSchemaEditCandidate;
use super::{AiBlackboardReferenceBlocker, AiBlackboardRenameCandidate, AiBlackboardRenameError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBlackboardMenuCommand {
    CreateKey,
    RenameKey,
    ChangeType,
    DeleteKey,
    EditRuntimeValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBlackboardMenuDisabledReason {
    ReadOnly,
    NoSelection,
    MultipleSelection,
    MissingKey,
    UnknownReferences,
    ReferencedTreeReadOnly,
    KeyReferenced,
    SchemaEvolutionUnsupported,
    RuntimeValuesReadOnly,
}

/// A count is scoped to this supplied plan, not a claim of global index completeness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBlackboardReferenceKnowledge<'a> {
    Unknown(&'a [AiBlackboardReferenceBlocker]),
    InSuppliedScope(usize),
}

/// `disabled=None` permits domain preparation only. The original host also gates
/// mode/debugger, command capability, document/source/revision/lease and budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiBlackboardMenuRow {
    pub command: AiBlackboardMenuCommand,
    pub text: &'static str,
    pub value: &'static str,
    pub disabled: Option<AiBlackboardMenuDisabledReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiBlackboardKeyMenuRow<'a> {
    pub key: &'a AiBlackboardKeyDescriptor,
    pub selected: bool,
    pub references: AiBlackboardReferenceKnowledge<'a>,
}

#[derive(Clone, Copy, Debug)]
pub enum AiBlackboardMenuRequest<'a> {
    CreateKey {
        index: usize,
        key: &'a AiBlackboardKeyDescriptor,
    },
    RenameKey {
        new_name: &'a str,
    },
    ChangeType {
        value_type: AiBlackboardValueType,
    },
    DeleteKey,
    EditRuntimeValue,
}

impl AiBlackboardMenuRequest<'_> {
    pub fn command(self) -> AiBlackboardMenuCommand {
        match self {
            Self::CreateKey { .. } => AiBlackboardMenuCommand::CreateKey,
            Self::RenameKey { .. } => AiBlackboardMenuCommand::RenameKey,
            Self::ChangeType { .. } => AiBlackboardMenuCommand::ChangeType,
            Self::DeleteKey => AiBlackboardMenuCommand::DeleteKey,
            Self::EditRuntimeValue => AiBlackboardMenuCommand::EditRuntimeValue,
        }
    }
}

/// Values only: the host retains all original lease, CAS, history and save authority.
#[derive(Debug)]
pub enum AiBlackboardMenuCandidate {
    CreateKey(AiBlackboardSchemaEditCandidate),
    RenameKey(AiBlackboardRenameCandidate),
}

#[derive(Debug, Error)]
pub enum AiBlackboardMenuError {
    #[error("Blackboard menu action is unavailable: {0:?}")]
    Disabled(AiBlackboardMenuDisabledReason),
    #[error("Blackboard key index {index} exceeds {count} slots")]
    KeyIndex { index: usize, count: usize },
    #[error("Blackboard schema validation failed: {0:?}")]
    InvalidSchema(AiManagerError),
    #[error(transparent)]
    Rename(#[from] AiBlackboardRenameError),
}
