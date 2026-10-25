use thiserror::Error;
use zircon_plugin_ai_runtime::behavior_tree::BehaviorNodeDescriptor;
use zircon_runtime::core::framework::ai::{AiBehaviorNodeDescriptor, AiBehaviorNodeKind};

use super::super::AiBehaviorTreeEditError;

/// Domain intents, not registered editor operation paths or execution authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBehaviorTreeMenuCommand {
    CreateChild,
    WrapDecorator,
    UnwrapDecorator,
    DeleteSubtree,
    AddService,
    ToggleBreakpoint,
    StepInto,
    StepOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBehaviorTreeMenuDisabledReason {
    ReadOnly,
    NoSelection,
    MultipleSelection,
    MissingNode,
    RootDeletion,
    LeafHasNoChildren,
    RequiredDecoratorChild,
    NotDecorator,
    WeightedTopology,
    MissingProviderIdentity,
    UnknownProvider,
    UnsupportedProvider,
    NodeKindMismatch,
    ServiceAttachmentUnsupported,
    RuntimeTraceControlUnsupported,
}

/// `text`/`value` project to the existing ListRow contract. A missing disabled
/// reason means preparation may be attempted, not that a host permit was granted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiBehaviorTreeMenuRow {
    pub command: AiBehaviorTreeMenuCommand,
    pub text: &'static str,
    pub value: &'static str,
    pub disabled: Option<AiBehaviorTreeMenuDisabledReason>,
}

/// Borrows the actual provider descriptor. The host supplies node IDs and all
/// typed parameters; a display category is never used to infer the creation kind.
#[derive(Clone, Copy, Debug)]
pub struct AiBehaviorTreeNodeMenuRow<'a> {
    pub descriptor: &'a BehaviorNodeDescriptor,
    pub kind: Option<AiBehaviorNodeKind>,
    pub disabled: Option<AiBehaviorTreeMenuDisabledReason>,
}

#[derive(Clone, Copy, Debug)]
pub enum AiBehaviorTreeMenuRequest<'a> {
    CreateChild {
        index: usize,
        node: &'a AiBehaviorNodeDescriptor,
    },
    WrapDecorator {
        decorator: &'a AiBehaviorNodeDescriptor,
    },
    UnwrapDecorator,
    DeleteSubtree,
    AddService,
    ToggleBreakpoint,
    StepInto,
    StepOver,
}

impl AiBehaviorTreeMenuRequest<'_> {
    pub fn command(self) -> AiBehaviorTreeMenuCommand {
        match self {
            Self::CreateChild { .. } => AiBehaviorTreeMenuCommand::CreateChild,
            Self::WrapDecorator { .. } => AiBehaviorTreeMenuCommand::WrapDecorator,
            Self::UnwrapDecorator => AiBehaviorTreeMenuCommand::UnwrapDecorator,
            Self::DeleteSubtree => AiBehaviorTreeMenuCommand::DeleteSubtree,
            Self::AddService => AiBehaviorTreeMenuCommand::AddService,
            Self::ToggleBreakpoint => AiBehaviorTreeMenuCommand::ToggleBreakpoint,
            Self::StepInto => AiBehaviorTreeMenuCommand::StepInto,
            Self::StepOver => AiBehaviorTreeMenuCommand::StepOver,
        }
    }
}

#[derive(Debug, Error)]
pub enum AiBehaviorTreeMenuError {
    #[error("behavior-tree menu action is unavailable: {0:?}")]
    Disabled(AiBehaviorTreeMenuDisabledReason),
    #[error(transparent)]
    Candidate(#[from] AiBehaviorTreeEditError),
}
