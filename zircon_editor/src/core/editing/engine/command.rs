use std::any::Any;
use std::error::Error;

use thiserror::Error;
use zircon_runtime::scene::SceneError;

use super::journal::{CommandJournalPayload, CommandJournalUnavailable};
use super::{HistoryContextId, TransactionId};
use crate::core::editing::selection::SelectionSnapshot;
use crate::core::gateway::{EditorRuntimeOperationRoute, GatewaySessionIdentity};
use crate::core::play::WorldDomain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditWorldRoute {
    world_domain: WorldDomain,
    gateway_identity: Option<GatewaySessionIdentity>,
}

impl EditWorldRoute {
    pub fn logical(world_domain: WorldDomain) -> Self {
        Self {
            world_domain,
            gateway_identity: None,
        }
    }

    /// Captures a route pinned to one concrete runtime-session generation.
    pub fn runtime(world_domain: WorldDomain, gateway_identity: GatewaySessionIdentity) -> Self {
        Self {
            world_domain,
            gateway_identity: Some(gateway_identity),
        }
    }

    pub const fn world_domain(&self) -> WorldDomain {
        self.world_domain
    }

    pub fn gateway_identity(&self) -> Option<&GatewaySessionIdentity> {
        self.gateway_identity.as_ref()
    }
}

pub trait EditContext: Any + Send {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError>;

    fn activate_world_route(&mut self, route: &EditWorldRoute) -> Result<(), EditCommandError>;

    fn retire_world_route(&mut self, world_domain: WorldDomain) -> Result<(), EditCommandError>;

    fn runtime_operations(&self) -> Result<EditorRuntimeOperationRoute, EditCommandError> {
        Err(EditCommandError::InvariantViolation {
            invariant: "the edit context does not expose runtime operation routing",
        })
    }

    /// Borrows the concrete selection owner; this proof must not create a snapshot.
    fn selection_retention_bound_v1(
        &self,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, EditCommandError>;

    fn selection_snapshot(&self) -> SelectionSnapshot;

    fn restore_selection(&mut self, snapshot: &SelectionSnapshot) -> Result<(), EditCommandError>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    Reject,
    Merged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandEffect {
    Unchanged,
    Applied,
    /// The lifecycle target changed; automatic compensation is no longer safe.
    Interrupted,
}

#[derive(Debug, Error)]
#[error("edit command failed with effect {effect:?}")]
pub struct CommandExecutionError {
    pub effect: CommandEffect,
    #[source]
    pub source: EditCommandError,
}

impl CommandExecutionError {
    pub fn unchanged(source: EditCommandError) -> Self {
        Self {
            effect: CommandEffect::Unchanged,
            source,
        }
    }

    pub fn applied(source: EditCommandError) -> Self {
        Self {
            effect: CommandEffect::Applied,
            source,
        }
    }

    pub fn interrupted(source: EditCommandError) -> Self {
        Self::interrupted_with_post_callback_error(source, None)
    }

    pub fn interrupted_with_post_callback_error(
        source: EditCommandError,
        post_callback_error: Option<EditCommandError>,
    ) -> Self {
        let source = match source {
            EditCommandError::LifecycleInterrupted {
                source,
                mut post_callback_errors,
            } => {
                post_callback_errors.extend(post_callback_error);
                EditCommandError::LifecycleInterrupted {
                    source,
                    post_callback_errors,
                }
            }
            source if post_callback_error.is_none() && source.is_interrupted() => source,
            source => EditCommandError::LifecycleInterrupted {
                source: Box::new(source),
                post_callback_errors: post_callback_error.into_iter().collect(),
            },
        };
        Self {
            effect: CommandEffect::Interrupted,
            source,
        }
    }

    /// Keeps interruption persistent even for a legal public struct literal.
    pub fn into_source(self) -> EditCommandError {
        let source = self.source;
        if self.effect == CommandEffect::Interrupted && !source.is_interrupted() {
            EditCommandError::LifecycleInterrupted {
                source: Box::new(source),
                post_callback_errors: Vec::new(),
            }
        } else {
            source
        }
    }
}

pub trait EditCommand:
    Any + Send + zircon_runtime_interface::retained_memory::RetainedOwnerV1
{
    /// Read-only full-state and preparation peak proof, before inverse allocation or mutation.
    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: super::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<super::CommandRetentionPlanV1, EditCommandError>;
    /// Uses the engine's measured residual record and preparation allowance.
    /// Pure bounded producers can use the default; protocol receivers may
    /// split their finite future payload and transport ceiling explicitly.
    fn retention_plan_with_limits_v1(
        &self,
        context: &dyn EditContext,
        phase: super::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limits: super::CommandRetentionLimitsV1,
    ) -> Result<super::CommandRetentionPlanV1, EditCommandError> {
        self.retention_plan_v1(context, phase, model, limits.retained_bytes)
    }

    /// Materializes only under the engine's one-use permit. Direct apply has no history-budget claim.
    fn prepare_retention_v1(
        &mut self,
        context: &mut dyn EditContext,
        phase: super::RetentionPhaseV1,
        permit: &mut super::RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError>;

    fn label(&self) -> &str;

    fn is_significant(&self) -> bool {
        true
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError>;

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError>;

    fn finalize(&mut self, _context: &mut dyn EditContext) {}

    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome {
        MergeOutcome::Reject
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        Err(CommandJournalUnavailable::new(self.label()))
    }

    fn as_any(&self) -> &dyn Any;
}

pub type CommandBox = Box<dyn EditCommand>;

#[derive(Debug, Error)]
pub enum EditCommandError {
    #[error(transparent)]
    Retention(#[from] zircon_runtime_interface::retained_memory::RetentionContractError),
    #[error("history admission refused before publication: {reason}")]
    RetentionAdmissionRefused { reason: &'static str },
    #[error("retained failure report: {category}; interrupted={interrupted}; recovery_failed={recovery_failed}")]
    RetainedFailureReport {
        category: &'static str,
        interrupted: bool,
        recovery_failed: bool,
    },
    #[error("edit target is missing: {target}")]
    TargetMissing { target: String },
    #[error("edit invariant was violated: {invariant}")]
    InvariantViolation { invariant: &'static str },
    #[error("reflection edit failed")]
    ReflectError {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("external edit effect failed")]
    ExternalEffect {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("scene mutation failed during {operation}")]
    SceneMutation {
        operation: &'static str,
        #[source]
        source: SceneError,
    },
    #[error("edit context has the wrong concrete type; expected {expected}")]
    ContextTypeMismatch { expected: &'static str },
    #[error("history capacity must be greater than zero")]
    InvalidHistoryCapacity,
    #[error("cannot nest history {requested:?} inside active history {active:?}")]
    CrossContextNested {
        active: HistoryContextId,
        requested: HistoryContextId,
    },
    #[error("history {requested:?} does not belong to world domain {world_domain:?}")]
    CrossWorldHistory {
        world_domain: WorldDomain,
        requested: HistoryContextId,
    },
    #[error("edit world route {world_domain:?} is unavailable")]
    WorldRouteUnavailable { world_domain: WorldDomain },
    #[error("edit world route {world_domain:?} changed before it could be activated")]
    WorldRouteStale { world_domain: WorldDomain },
    #[error("cannot {operation} volatile history {history:?}")]
    VolatileHistoryPersistenceUnsupported {
        history: HistoryContextId,
        operation: &'static str,
    },
    #[error("transaction scope is no longer active")]
    ScopeClosed,
    #[error("editor transaction engine is busy with {active}; cannot start {requested}")]
    EngineBusy {
        active: &'static str,
        requested: &'static str,
    },
    #[error("editor transaction engine is faulted during {operation}")]
    EngineFaulted { operation: &'static str },
    #[error("transaction identifier space is exhausted")]
    TransactionIdExhausted,
    #[error("history generation space is exhausted for {history:?}")]
    HistoryGenerationExhausted { history: HistoryContextId },
    #[error("history dirty-state generation space is exhausted")]
    HistoryDirtyGenerationExhausted,
    #[error("applied command revision space is exhausted")]
    AppliedCommandRevisionExhausted,
    #[error("history decision changed for {history:?}")]
    HistoryDecisionChanged { history: HistoryContextId },
    #[error("selection generation space is exhausted")]
    SelectionGenerationExhausted,
    #[error("history detail page size {requested} exceeds the maximum {maximum}")]
    HistoryPageSizeOutOfRange { requested: usize, maximum: usize },
    #[error("history detail cursor belongs to another transaction engine instance")]
    HistoryPageCursorEngineMismatch,
    #[error(
        "history detail cursor belongs to history {cursor_history:?}, not requested history {requested_history:?}"
    )]
    HistoryPageCursorHistoryMismatch {
        cursor_history: HistoryContextId,
        requested_history: HistoryContextId,
    },
    #[error(
        "history {history:?} changed while paging: cursor generation {cursor_generation}, current generation {current_generation}"
    )]
    HistoryPageCursorStale {
        history: HistoryContextId,
        cursor_generation: u64,
        current_generation: u64,
    },
    #[error("history dirty-state cursor belongs to another transaction engine instance")]
    HistoryDirtyCursorEngineMismatch,
    #[error(
        "save token belongs to history {token_history:?}, not requested history {requested_history:?}"
    )]
    SaveTokenHistoryMismatch {
        token_history: HistoryContextId,
        requested_history: HistoryContextId,
    },
    #[error("save token belongs to another transaction engine instance")]
    SaveTokenEngineMismatch,
    #[error(
        "cannot {operation} while transaction {transaction:?} is active in history {active_history:?}"
    )]
    SaveTokenActiveTransaction {
        operation: &'static str,
        active_history: HistoryContextId,
        transaction: TransactionId,
    },
    #[error(
        "history {history:?} changed during save: generation {expected_generation} at {expected_transaction:?} became generation {current_generation} at {current_transaction:?}"
    )]
    HistoryChangedDuringSave {
        history: HistoryContextId,
        expected_generation: u64,
        current_generation: u64,
        expected_transaction: Option<TransactionId>,
        current_transaction: Option<TransactionId>,
    },
    #[error("edit lifecycle was interrupted")]
    LifecycleInterrupted {
        #[source]
        source: Box<EditCommandError>,
        post_callback_errors: Vec<EditCommandError>,
    },
    #[error("command rollback failed after an earlier command error")]
    RollbackFailed {
        command_error: Box<EditCommandError>,
        #[source]
        rollback_error: Box<EditCommandError>,
    },
}

impl EditCommandError {
    pub fn is_interrupted(&self) -> bool {
        // Public error trees may be hand-built. Never recursively traverse an untrusted depth.
        let mut pending = [None; 64];
        pending[0] = Some(self);
        let mut len = 1usize;
        let mut work = 0usize;
        while len != 0 {
            len -= 1;
            let error = pending[len].take().expect("occupied error slot");
            work += 1;
            if work > 4096 {
                return true;
            }
            match error {
                Self::LifecycleInterrupted { .. } => return true,
                Self::RetainedFailureReport { interrupted, .. } if *interrupted => return true,
                Self::RollbackFailed {
                    command_error,
                    rollback_error,
                } => {
                    if len + 2 > pending.len() {
                        return true;
                    }
                    pending[len] = Some(command_error.as_ref());
                    pending[len + 1] = Some(rollback_error.as_ref());
                    len += 2;
                }
                _ => {}
            }
        }
        false
    }

    pub(crate) fn bounded_failure_report_v1(&self) -> Self {
        let category = match self {
            Self::RetainedFailureReport { category, .. } => *category,
            Self::Retention(_) | Self::RetentionAdmissionRefused { .. } => "retention",
            Self::TargetMissing { .. } => "target",
            Self::ReflectError { .. } => "reflection",
            Self::ExternalEffect { .. } => "external",
            Self::SceneMutation { .. } => "scene",
            Self::RollbackFailed { .. } => "rollback",
            Self::LifecycleInterrupted { .. } => "lifecycle",
            _ => "engine",
        };
        Self::RetainedFailureReport {
            category,
            interrupted: self.is_interrupted(),
            recovery_failed: matches!(self, Self::RollbackFailed { .. })
                || matches!(
                    self,
                    Self::RetainedFailureReport {
                        recovery_failed: true,
                        ..
                    }
                ),
        }
    }
}
