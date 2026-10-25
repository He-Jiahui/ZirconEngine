use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use thiserror::Error;
use zircon_runtime::core::runtime::tasks::BoundedKeyedIoShutdownGuard;

use crate::core::editor_operation::EditorOperationPath;

use super::{
    ResolvedSettingValue, ResolvedSettingsBatch, SettingChange, SettingValue, SettingsAuthority,
    SettingsError, SettingsFileGeneration, SettingsKey, SettingsPersistenceRetryError,
    SettingsPersistenceService, SettingsPersistenceShutdown, SettingsPersistenceSubmitError,
    SettingsPersistenceTerminal, SettingsPersistenceTicket, SettingsProjectLayerLoad,
    SettingsScope, SettingsStore,
};

#[path = "mutation/health.rs"]
mod health;

use health::SettingsPersistenceHealthAuthority;
pub(crate) use health::SettingsPersistenceHealthSubscriber;
pub use health::{
    SettingsPersistenceDocumentHealth, SettingsPersistenceHealthSnapshot,
    SettingsPersistenceHealthStatus,
};

const MAX_PENDING_SETTINGS_DOCUMENTS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsMutationDisposition {
    Unchanged,
    SessionApplied,
    PersistentQueued,
    AppliedPendingAdmission(SettingsPersistenceSubmitError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsMutationReceipt {
    key: SettingsKey,
    scope: SettingsScope,
    authority_generation: u64,
    persistence_generation: Option<SettingsFileGeneration>,
    requires_restart: bool,
    disposition: SettingsMutationDisposition,
}

impl SettingsMutationReceipt {
    pub fn key(&self) -> &SettingsKey {
        &self.key
    }

    pub const fn scope(&self) -> SettingsScope {
        self.scope
    }

    pub const fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    pub const fn persistence_generation(&self) -> Option<SettingsFileGeneration> {
        self.persistence_generation
    }

    pub const fn requires_restart(&self) -> bool {
        self.requires_restart
    }

    pub const fn disposition(&self) -> SettingsMutationDisposition {
        self.disposition
    }

    pub const fn changed(&self) -> bool {
        !matches!(self.disposition, SettingsMutationDisposition::Unchanged)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsProjectBindingReceipt {
    generation: u64,
    load: SettingsProjectLayerLoad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPersistenceRetryDisposition {
    NothingPending,
    AlreadyQueued,
    PersistentQueued,
    Durable,
    Superseded { successor: SettingsFileGeneration },
    PendingAdmission(SettingsPersistenceSubmitError),
    RetryRejected(SettingsPersistenceRetryError),
    TerminalNotRetryable(SettingsPersistenceTerminal),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsPersistenceRetryReceipt {
    scope: SettingsScope,
    file_generation: Option<SettingsFileGeneration>,
    disposition: SettingsPersistenceRetryDisposition,
}

impl SettingsPersistenceRetryReceipt {
    pub const fn scope(&self) -> SettingsScope {
        self.scope
    }

    pub const fn file_generation(&self) -> Option<SettingsFileGeneration> {
        self.file_generation
    }

    pub const fn disposition(&self) -> SettingsPersistenceRetryDisposition {
        self.disposition
    }
}

impl SettingsProjectBindingReceipt {
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn load(&self) -> &SettingsProjectLayerLoad {
        &self.load
    }
}

/// Values and project identity captured together for a displayed settings batch.
/// This is not an originating-frame or window-incarnation token.
#[derive(Clone, Debug)]
pub(crate) struct SettingsMutationContext {
    owner: Weak<Mutex<SettingsMutationState>>,
    values: ResolvedSettingsBatch,
    project_generation: Option<u64>,
    next_project_generation: u64,
}

impl PartialEq for SettingsMutationContext {
    fn eq(&self, other: &Self) -> bool {
        self.owner.ptr_eq(&other.owner)
            && self.values == other.values
            && self.project_generation == other.project_generation
            && self.next_project_generation == other.next_project_generation
    }
}

impl SettingsMutationContext {
    pub(crate) fn values(&self) -> &ResolvedSettingsBatch {
        &self.values
    }

    pub(crate) fn generation(&self) -> u64 {
        self.values.generation()
    }

    pub(crate) fn value(&self, key: &SettingsKey) -> Option<&ResolvedSettingValue> {
        self.values.values().iter().find(|value| value.key() == key)
    }
}

/// Original pending document captured while its health was prepared for display.
/// This does not identify an event's originating frame or window incarnation.
#[derive(Clone, Debug)]
pub(crate) struct SettingsPersistenceRetryContext {
    owner: Weak<Mutex<SettingsMutationState>>,
    document: SettingsDocumentIdentity,
    file_generation: SettingsFileGeneration,
}

impl PartialEq for SettingsPersistenceRetryContext {
    fn eq(&self, other: &Self) -> bool {
        self.owner.ptr_eq(&other.owner)
            && self.document == other.document
            && self.file_generation == other.file_generation
    }
}

impl SettingsPersistenceRetryContext {
    pub(crate) const fn scope(&self) -> SettingsScope {
        match self.document {
            SettingsDocumentIdentity::User => SettingsScope::User,
            SettingsDocumentIdentity::Project(_) => SettingsScope::Project,
        }
    }

    pub(crate) const fn file_generation(&self) -> SettingsFileGeneration {
        self.file_generation
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SettingsMutationError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error("settings display context belongs to another mutation coordinator")]
    ForeignMutationContext,
    #[error("project binding changed since the settings values were displayed")]
    StaleProjectBinding,
    #[error("pending settings document changed since its retry target was displayed")]
    StalePersistenceRetryContext,
    #[error("setting `{0}` was not in the displayed values batch")]
    KeyNotCaptured(SettingsKey),
    #[error("User settings have no writable source")]
    UserSourceUnavailable,
    #[error("Project settings require an active project binding")]
    ProjectNotBound,
    #[error("the active Project settings source is invalid and cannot be overwritten")]
    ProjectSourceInvalid,
    #[error("{0:?} settings do not have a persistence document")]
    NonPersistentScope(SettingsScope),
    #[error("settings operation `{active}` is already in progress")]
    OperationInProgress { active: &'static str },
    #[error("settings project-binding generation is exhausted")]
    ProjectBindingGenerationExhausted,
    #[error("settings file generation is exhausted")]
    FileGenerationExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SettingsDocumentIdentity {
    User,
    Project(u64),
}

enum SettingsMutationTarget {
    Session,
    Persistent {
        document: SettingsDocumentIdentity,
        store: SettingsStore,
        file_generation: SettingsFileGeneration,
    },
}

enum PendingSettingsDocument {
    Deferred {
        change: SettingChange,
        store: SettingsStore,
        file_generation: SettingsFileGeneration,
        admission_error: SettingsPersistenceSubmitError,
    },
    Ticket(SettingsPersistenceTicket),
}

impl PendingSettingsDocument {
    fn file_generation(&self) -> SettingsFileGeneration {
        match self {
            Self::Deferred {
                file_generation, ..
            } => *file_generation,
            Self::Ticket(ticket) => ticket.file_generation(),
        }
    }

    fn cancel_before_start(&self) {
        if let Self::Ticket(ticket) = self {
            let _ = ticket.cancel_before_start();
        }
    }
}

#[derive(Clone)]
struct ProjectSettingsBinding {
    generation: u64,
    store: SettingsStore,
    writable: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsCoordinatorPhase {
    #[default]
    Idle,
    ProjectTransition,
    Mutation,
}

impl SettingsCoordinatorPhase {
    const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::ProjectTransition => "project-transition",
            Self::Mutation => "mutation",
        }
    }
}

struct SettingsMutationState {
    phase: SettingsCoordinatorPhase,
    user_store: Option<SettingsStore>,
    project: Option<ProjectSettingsBinding>,
    next_project_generation: u64,
    pending: BTreeMap<SettingsDocumentIdentity, PendingSettingsDocument>,
}

/// Process-wide settings mutation, source binding, and persistence receipt owner.
pub struct SettingsMutationCoordinator {
    authority: Arc<SettingsAuthority>,
    persistence: SettingsPersistenceService,
    persistence_health: Arc<SettingsPersistenceHealthAuthority>,
    state: Arc<Mutex<SettingsMutationState>>,
}

impl SettingsMutationCoordinator {
    pub fn new(
        authority: Arc<SettingsAuthority>,
        persistence: SettingsPersistenceService,
        user_store: Option<SettingsStore>,
    ) -> Self {
        let persistence_health = Arc::new(SettingsPersistenceHealthAuthority::new(
            user_store.is_some(),
        ));
        Self {
            authority,
            persistence,
            persistence_health,
            state: Arc::new(Mutex::new(SettingsMutationState {
                phase: SettingsCoordinatorPhase::Idle,
                user_store,
                project: None,
                next_project_generation: 1,
                pending: BTreeMap::new(),
            })),
        }
    }

    pub fn authority(&self) -> &Arc<SettingsAuthority> {
        &self.authority
    }

    pub fn persistence_health_snapshot(&self) -> SettingsPersistenceHealthSnapshot {
        self.persistence_health.snapshot()
    }

    pub(crate) fn capture_persistence_health_with_retry(
        &self,
    ) -> Result<
        (
            SettingsPersistenceHealthSnapshot,
            Vec<SettingsPersistenceRetryContext>,
        ),
        SettingsMutationError,
    > {
        let state = self.lock_state();
        require_idle(&state)?;
        let health = self.persistence_health.snapshot();
        let mut contexts = Vec::with_capacity(MAX_PENDING_SETTINGS_DOCUMENTS);
        for observed in [health.project(), health.user()] {
            if !observed.status().is_retryable() {
                continue;
            }
            let document = match observed.scope() {
                SettingsScope::User if state.user_store.is_some() => SettingsDocumentIdentity::User,
                SettingsScope::Project => match state.project.as_ref() {
                    Some(project)
                        if project.writable
                            && observed.binding_generation() == Some(project.generation) =>
                    {
                        SettingsDocumentIdentity::Project(project.generation)
                    }
                    _ => continue,
                },
                _ => continue,
            };
            let Some(pending) = state.pending.get(&document) else {
                continue;
            };
            let file_generation = pending.file_generation();
            if observed.file_generation() == Some(file_generation) {
                contexts.push(SettingsPersistenceRetryContext {
                    owner: Arc::downgrade(&self.state),
                    document,
                    file_generation,
                });
            }
        }
        Ok((health, contexts))
    }

    pub(crate) fn configure_persistence_health_subscriber(
        &self,
        subscriber: Arc<dyn SettingsPersistenceHealthSubscriber>,
    ) {
        self.persistence_health.configure_subscriber(subscriber);
    }

    #[cfg(test)]
    pub(crate) fn in_memory_with_defaults() -> Self {
        let authority = Arc::new(SettingsAuthority::with_defaults());
        let persistence = SettingsPersistenceService::new(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
        );
        Self::new(authority, persistence, None)
    }

    pub fn set(
        &self,
        scope: SettingsScope,
        key: &SettingsKey,
        value: SettingValue,
    ) -> Result<SettingsMutationReceipt, SettingsMutationError> {
        let target = self.begin_mutation(scope)?;
        let change = self.authority.set(scope, key, value);
        self.complete_mutation(scope, key, target, change)
    }

    pub(crate) fn capture_values<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a SettingsKey>,
    ) -> Result<SettingsMutationContext, SettingsMutationError> {
        self.capture_values_with_generation(None, keys)
    }

    pub(crate) fn capture_values_at_generation<'a>(
        &self,
        expected_generation: u64,
        keys: impl IntoIterator<Item = &'a SettingsKey>,
    ) -> Result<SettingsMutationContext, SettingsMutationError> {
        self.capture_values_with_generation(Some(expected_generation), keys)
    }

    fn capture_values_with_generation<'a>(
        &self,
        expected_generation: Option<u64>,
        keys: impl IntoIterator<Item = &'a SettingsKey>,
    ) -> Result<SettingsMutationContext, SettingsMutationError> {
        let state = self.lock_state();
        require_idle(&state)?;
        // Hold the existing operation-state guard through the authority batch read.
        // A project transition cannot straddle the values and their binding identity.
        let values = match expected_generation {
            Some(expected) => self
                .authority
                .resolved_settings_from_iter_at_generation(expected, keys)?,
            None => self.authority.resolved_settings_from_iter(keys)?,
        };
        Ok(SettingsMutationContext {
            owner: Arc::downgrade(&self.state),
            values,
            project_generation: state.project.as_ref().map(|project| project.generation),
            next_project_generation: state.next_project_generation,
        })
    }

    pub(crate) fn context_is_current(&self, context: &SettingsMutationContext) -> bool {
        self.check_values_context(context).is_ok()
    }

    pub(crate) fn check_values_context(
        &self,
        context: &SettingsMutationContext,
    ) -> Result<(), SettingsMutationError> {
        let state = self.lock_state();
        require_idle(&state)?;
        check_project_context(&state, context, &self.state)?;
        // Read-only UI transitions use the same original binding/value fence. This does not
        // reserve a later mutation or identify the event's originating frame/window lifetime.
        self.authority
            .check_generation(context.generation())
            .map_err(Into::into)
    }

    pub(crate) fn set_from_context(
        &self,
        context: &SettingsMutationContext,
        scope: SettingsScope,
        key: &SettingsKey,
        value: SettingValue,
    ) -> Result<SettingsMutationReceipt, SettingsMutationError> {
        if context.value(key).is_none() {
            return Err(SettingsMutationError::KeyNotCaptured(key.clone()));
        }
        let target = self.begin_mutation_with_context(scope, Some(context))?;
        let change = self
            .authority
            .set_if_generation(context.generation(), scope, key, value);
        self.complete_mutation(scope, key, target, change)
    }

    pub(crate) fn clear_from_context(
        &self,
        context: &SettingsMutationContext,
        scope: SettingsScope,
        key: &SettingsKey,
    ) -> Result<Option<SettingsMutationReceipt>, SettingsMutationError> {
        let displayed = context
            .value(key)
            .ok_or_else(|| SettingsMutationError::KeyNotCaptured(key.clone()))?;
        // This snapshot supplies definition scope only; it does not replace display provenance.
        let snapshot = self.authority.snapshot();
        let definition = snapshot
            .catalog()
            .definition(key)
            .ok_or_else(|| SettingsError::UnknownKey(key.as_str().to_owned()))?;
        if !definition.scope.allows_write(scope) {
            return Err(SettingsError::ScopeNotAllowed {
                key: key.as_str().to_owned(),
                requested: scope,
                defined: definition.scope,
            }
            .into());
        }
        if displayed.override_in_scope(scope).is_none() {
            // Preserve reset's absent-layer no-op without requiring a writable store.
            // It still has a linearization point under the original value lock.
            {
                let mut state = self.lock_state();
                begin_phase(&mut state, SettingsCoordinatorPhase::Mutation)?;
                if let Err(error) = check_project_context(&state, context, &self.state) {
                    state.phase = SettingsCoordinatorPhase::Idle;
                    return Err(error);
                }
            }
            let result = self.authority.check_generation(context.generation());
            self.finish_operation();
            result?;
            return Ok(None);
        }
        let target = self.begin_mutation_with_context(scope, Some(context))?;
        let change = self
            .authority
            .clear_if_generation(context.generation(), scope, key);
        let receipt = self.complete_mutation(scope, key, target, change)?;
        Ok(receipt.changed().then_some(receipt))
    }

    pub fn clear(
        &self,
        scope: SettingsScope,
        key: &SettingsKey,
    ) -> Result<SettingsMutationReceipt, SettingsMutationError> {
        let target = self.begin_mutation(scope)?;
        let change = self.authority.clear(scope, key);
        self.complete_mutation(scope, key, target, change)
    }

    pub fn record_command_palette_usage(
        &self,
        command: EditorOperationPath,
    ) -> Result<Option<SettingsMutationReceipt>, SettingsMutationError> {
        let _target = self.begin_mutation(SettingsScope::Session)?;
        let change = self.authority.record_command_palette_usage(command);
        let change = match change {
            Ok(change) => change,
            Err(error) => {
                self.finish_operation();
                return Err(error.into());
            }
        };
        self.finish_operation();
        Ok(change.as_ref().map(|change| {
            receipt_for_change(change, None, SettingsMutationDisposition::SessionApplied)
        }))
    }

    fn complete_mutation(
        &self,
        scope: SettingsScope,
        key: &SettingsKey,
        target: SettingsMutationTarget,
        change: Result<Option<SettingChange>, SettingsError>,
    ) -> Result<SettingsMutationReceipt, SettingsMutationError> {
        let change = match change {
            Ok(change) => change,
            Err(error) => {
                self.finish_operation();
                return Err(error.into());
            }
        };
        let Some(change) = change else {
            let generation = self.authority.snapshot().generation();
            self.finish_operation();
            return Ok(SettingsMutationReceipt {
                key: key.clone(),
                scope,
                authority_generation: generation,
                persistence_generation: None,
                requires_restart: false,
                disposition: SettingsMutationDisposition::Unchanged,
            });
        };

        let SettingsMutationTarget::Persistent {
            document,
            store,
            file_generation,
        } = target
        else {
            self.finish_operation();
            return Ok(receipt_for_change(
                &change,
                None,
                SettingsMutationDisposition::SessionApplied,
            ));
        };

        let submit =
            self.submit_persistent_change(document, &change, file_generation, store.clone());
        let disposition = {
            let mut state = self.lock_state();
            replace_pending_document(
                &mut state,
                document,
                &change,
                store,
                file_generation,
                submit,
            )
        };
        self.finish_operation();
        Ok(receipt_for_change(
            &change,
            Some(file_generation),
            disposition,
        ))
    }

    pub fn retry_pending(
        &self,
        scope: SettingsScope,
    ) -> Result<SettingsPersistenceRetryReceipt, SettingsMutationError> {
        self.retry_pending_with_context(scope, None)
    }

    pub(crate) fn retry_pending_from_context(
        &self,
        context: &SettingsPersistenceRetryContext,
        scope: SettingsScope,
    ) -> Result<SettingsPersistenceRetryReceipt, SettingsMutationError> {
        self.retry_pending_with_context(scope, Some(context))
    }

    fn retry_pending_with_context(
        &self,
        scope: SettingsScope,
        context: Option<&SettingsPersistenceRetryContext>,
    ) -> Result<SettingsPersistenceRetryReceipt, SettingsMutationError> {
        if !scope.is_persistent() {
            return Err(SettingsMutationError::NonPersistentScope(scope));
        }
        let document = self.begin_persistence_retry(scope, context)?;
        let pending = self.lock_state().pending.remove(&document);
        let (file_generation, disposition, replacement) = match pending {
            None => (
                None,
                SettingsPersistenceRetryDisposition::NothingPending,
                None,
            ),
            Some(PendingSettingsDocument::Deferred {
                change,
                store,
                file_generation,
                admission_error: _,
            }) => match self.submit_persistent_change(
                document,
                &change,
                file_generation,
                store.clone(),
            ) {
                Ok(ticket) => (
                    Some(file_generation),
                    SettingsPersistenceRetryDisposition::PersistentQueued,
                    Some(PendingSettingsDocument::Ticket(ticket)),
                ),
                Err(error) => (
                    Some(file_generation),
                    SettingsPersistenceRetryDisposition::PendingAdmission(error),
                    Some(PendingSettingsDocument::Deferred {
                        change,
                        store,
                        file_generation,
                        admission_error: error,
                    }),
                ),
            },
            Some(PendingSettingsDocument::Ticket(ticket)) => {
                let generation = ticket.file_generation();
                match ticket.persistence_terminal() {
                    None => (
                        Some(generation),
                        SettingsPersistenceRetryDisposition::AlreadyQueued,
                        Some(PendingSettingsDocument::Ticket(ticket)),
                    ),
                    Some(SettingsPersistenceTerminal::Written) => (
                        Some(generation),
                        SettingsPersistenceRetryDisposition::Durable,
                        None,
                    ),
                    Some(SettingsPersistenceTerminal::Superseded { successor }) => (
                        Some(generation),
                        SettingsPersistenceRetryDisposition::Superseded {
                            successor: SettingsFileGeneration::from_raw(successor),
                        },
                        None,
                    ),
                    Some(SettingsPersistenceTerminal::Failed(_)) => {
                        match self.retry_persistence_ticket(document, &ticket) {
                            Ok(retry) => (
                                Some(generation),
                                SettingsPersistenceRetryDisposition::PersistentQueued,
                                Some(PendingSettingsDocument::Ticket(retry)),
                            ),
                            Err(error) => (
                                Some(generation),
                                SettingsPersistenceRetryDisposition::RetryRejected(error),
                                Some(PendingSettingsDocument::Ticket(ticket)),
                            ),
                        }
                    }
                    Some(terminal) => (
                        Some(generation),
                        SettingsPersistenceRetryDisposition::TerminalNotRetryable(terminal),
                        Some(PendingSettingsDocument::Ticket(ticket)),
                    ),
                }
            }
        };
        if let Some(replacement) = replacement {
            self.lock_state().pending.insert(document, replacement);
        }
        self.finish_operation();
        Ok(SettingsPersistenceRetryReceipt {
            scope,
            file_generation,
            disposition,
        })
    }

    pub fn bind_project(
        &self,
        project_root: &Path,
    ) -> Result<SettingsProjectBindingReceipt, SettingsMutationError> {
        let (generation, store, retired) = {
            let mut state = self.lock_state();
            begin_phase(&mut state, SettingsCoordinatorPhase::ProjectTransition)?;
            let generation = state.next_project_generation;
            let Some(next_generation) = generation.checked_add(1) else {
                state.phase = SettingsCoordinatorPhase::Idle;
                return Err(SettingsMutationError::ProjectBindingGenerationExhausted);
            };
            state.next_project_generation = next_generation;
            let store = state.user_store.as_ref().map_or_else(
                || SettingsStore::from_roots(project_root, Some(project_root)),
                |user_store| user_store.with_project_root(project_root),
            );
            let retired = take_project_pending(&mut state.pending);
            state.project = None;
            (generation, store, retired)
        };
        retired.cancel_before_start();

        let load = self.authority.load_project_layer_from_store(&store);
        let writable = !matches!(load, SettingsProjectLayerLoad::Invalid { .. });
        {
            let mut state = self.lock_state();
            state.project = Some(ProjectSettingsBinding {
                generation,
                store,
                writable,
            });
            state.phase = SettingsCoordinatorPhase::Idle;
        }
        self.persistence_health.bind_project(generation, writable);
        Ok(SettingsProjectBindingReceipt { generation, load })
    }

    pub fn clear_project(&self) -> Result<bool, SettingsMutationError> {
        let (had_project, retired) = {
            let mut state = self.lock_state();
            begin_phase(&mut state, SettingsCoordinatorPhase::ProjectTransition)?;
            let had_project = state.project.take().is_some();
            let retired = take_project_pending(&mut state.pending);
            (had_project, retired)
        };
        retired.cancel_before_start();
        self.authority.clear_project_layer();
        self.finish_operation();
        self.persistence_health.clear_project();
        Ok(had_project)
    }

    pub fn flush_then_shutdown(
        &self,
    ) -> Result<SettingsPersistenceShutdown, SettingsPersistenceSubmitError> {
        if let Some(error) = self.lock_state().pending.values().find_map(|pending| {
            if let PendingSettingsDocument::Deferred {
                admission_error, ..
            } = pending
            {
                Some(*admission_error)
            } else {
                None
            }
        }) {
            return Err(error);
        }
        self.persistence.flush_then_shutdown()
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        self.persistence.shutdown()
    }

    fn submit_persistent_change(
        &self,
        document: SettingsDocumentIdentity,
        change: &SettingChange,
        file_generation: SettingsFileGeneration,
        store: SettingsStore,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceSubmitError> {
        let observation = self
            .persistence_health
            .begin_submission(document, file_generation);
        let health = Arc::clone(&self.persistence_health);
        let submit =
            self.persistence
                .submit_observed(change, file_generation, store, move |terminal| {
                    health.observe_terminal(observation, terminal);
                });
        if let Err(error) = submit.as_ref() {
            self.persistence_health
                .admission_rejected(observation, *error);
        }
        submit
    }

    fn retry_persistence_ticket(
        &self,
        document: SettingsDocumentIdentity,
        ticket: &SettingsPersistenceTicket,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceRetryError> {
        let observation = self
            .persistence_health
            .begin_submission(document, ticket.file_generation());
        let health = Arc::clone(&self.persistence_health);
        let retry = self.persistence.retry_observed(ticket, move |terminal| {
            health.observe_terminal(observation, terminal);
        });
        if let Err(SettingsPersistenceRetryError::LaneAdmission(error)) = retry.as_ref() {
            self.persistence_health.admission_rejected(
                observation,
                SettingsPersistenceSubmitError::LaneAdmission(*error),
            );
        }
        retry
    }

    fn begin_mutation(
        &self,
        scope: SettingsScope,
    ) -> Result<SettingsMutationTarget, SettingsMutationError> {
        self.begin_mutation_with_context(scope, None)
    }

    fn begin_mutation_with_context(
        &self,
        scope: SettingsScope,
        context: Option<&SettingsMutationContext>,
    ) -> Result<SettingsMutationTarget, SettingsMutationError> {
        let mut state = self.lock_state();
        begin_phase(&mut state, SettingsCoordinatorPhase::Mutation)?;
        if let Some(context) = context {
            if let Err(error) = check_project_context(&state, context, &self.state) {
                state.phase = SettingsCoordinatorPhase::Idle;
                return Err(error);
            }
        }
        let result = match scope {
            SettingsScope::Session => Ok(SettingsMutationTarget::Session),
            SettingsScope::User => match state.user_store.clone() {
                Some(store) => self.allocate_file_generation().map(|file_generation| {
                    SettingsMutationTarget::Persistent {
                        document: SettingsDocumentIdentity::User,
                        store,
                        file_generation,
                    }
                }),
                None => Err(SettingsMutationError::UserSourceUnavailable),
            },
            SettingsScope::Project => match state.project.as_ref() {
                Some(project) if project.writable => {
                    self.allocate_file_generation().map(|file_generation| {
                        SettingsMutationTarget::Persistent {
                            document: SettingsDocumentIdentity::Project(project.generation),
                            store: project.store.clone(),
                            file_generation,
                        }
                    })
                }
                Some(_) => Err(SettingsMutationError::ProjectSourceInvalid),
                None => Err(SettingsMutationError::ProjectNotBound),
            },
        };
        if result.is_err() {
            state.phase = SettingsCoordinatorPhase::Idle;
        }
        result
    }

    fn allocate_file_generation(&self) -> Result<SettingsFileGeneration, SettingsMutationError> {
        match self.persistence.allocate_file_generation() {
            Ok(generation) => Ok(generation),
            Err(SettingsPersistenceSubmitError::FileGenerationExhausted) => {
                Err(SettingsMutationError::FileGenerationExhausted)
            }
            Err(_) => unreachable!("file generation allocation has no target or lane work"),
        }
    }

    fn begin_persistence_retry(
        &self,
        scope: SettingsScope,
        context: Option<&SettingsPersistenceRetryContext>,
    ) -> Result<SettingsDocumentIdentity, SettingsMutationError> {
        let mut state = self.lock_state();
        begin_phase(&mut state, SettingsCoordinatorPhase::Mutation)?;
        let result = (|| {
            if let Some(context) = context {
                if !std::ptr::eq(context.owner.as_ptr(), Arc::as_ptr(&self.state)) {
                    return Err(SettingsMutationError::ForeignMutationContext);
                }
                if context.scope() != scope {
                    return Err(SettingsMutationError::StalePersistenceRetryContext);
                }
            }
            let document = match scope {
                SettingsScope::User => state
                    .user_store
                    .as_ref()
                    .map(|_| SettingsDocumentIdentity::User)
                    .ok_or(SettingsMutationError::UserSourceUnavailable),
                SettingsScope::Project => match state.project.as_ref() {
                    Some(project) if project.writable => {
                        Ok(SettingsDocumentIdentity::Project(project.generation))
                    }
                    Some(_) => Err(SettingsMutationError::ProjectSourceInvalid),
                    None => Err(SettingsMutationError::ProjectNotBound),
                },
                SettingsScope::Session => Err(SettingsMutationError::NonPersistentScope(scope)),
            }?;
            if let Some(context) = context {
                if context.document != document {
                    return Err(SettingsMutationError::StaleProjectBinding);
                }
                if state
                    .pending
                    .get(&document)
                    .map(PendingSettingsDocument::file_generation)
                    != Some(context.file_generation())
                {
                    return Err(SettingsMutationError::StalePersistenceRetryContext);
                }
            }
            Ok(document)
        })();
        if result.is_err() {
            state.phase = SettingsCoordinatorPhase::Idle;
        }
        result
    }

    fn finish_operation(&self) {
        self.lock_state().phase = SettingsCoordinatorPhase::Idle;
    }

    fn lock_state(&self) -> MutexGuard<'_, SettingsMutationState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn require_idle(state: &SettingsMutationState) -> Result<(), SettingsMutationError> {
    if state.phase != SettingsCoordinatorPhase::Idle {
        return Err(SettingsMutationError::OperationInProgress {
            active: state.phase.name(),
        });
    }
    Ok(())
}

fn check_project_context(
    state: &SettingsMutationState,
    context: &SettingsMutationContext,
    owner: &Arc<Mutex<SettingsMutationState>>,
) -> Result<(), SettingsMutationError> {
    // The Weak keeps the original allocation identity from being reused without retaining its
    // state or authority. Local binding counters alone can coincide in another coordinator.
    // This verifies the captured source, not an event's originating frame or window lifetime.
    if !std::ptr::eq(context.owner.as_ptr(), Arc::as_ptr(owner)) {
        return Err(SettingsMutationError::ForeignMutationContext);
    }
    if context.project_generation != state.project.as_ref().map(|project| project.generation)
        || context.next_project_generation != state.next_project_generation
    {
        return Err(SettingsMutationError::StaleProjectBinding);
    }
    Ok(())
}

fn begin_phase(
    state: &mut SettingsMutationState,
    next: SettingsCoordinatorPhase,
) -> Result<(), SettingsMutationError> {
    require_idle(state)?;
    state.phase = next;
    Ok(())
}

fn replace_pending_document(
    state: &mut SettingsMutationState,
    document: SettingsDocumentIdentity,
    change: &SettingChange,
    store: SettingsStore,
    file_generation: SettingsFileGeneration,
    submit: Result<SettingsPersistenceTicket, SettingsPersistenceSubmitError>,
) -> SettingsMutationDisposition {
    if let Some(previous) = state.pending.remove(&document) {
        previous.cancel_before_start();
    }
    debug_assert!(state.pending.len() < MAX_PENDING_SETTINGS_DOCUMENTS);
    match submit {
        Ok(ticket) => {
            state
                .pending
                .insert(document, PendingSettingsDocument::Ticket(ticket));
            SettingsMutationDisposition::PersistentQueued
        }
        Err(error) => {
            state.pending.insert(
                document,
                PendingSettingsDocument::Deferred {
                    change: change.clone(),
                    store,
                    file_generation,
                    admission_error: error,
                },
            );
            SettingsMutationDisposition::AppliedPendingAdmission(error)
        }
    }
}

fn take_project_pending(
    pending: &mut BTreeMap<SettingsDocumentIdentity, PendingSettingsDocument>,
) -> PendingSettingsDocuments {
    let retired = pending
        .keys()
        .copied()
        .filter(|identity| matches!(identity, SettingsDocumentIdentity::Project(_)))
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|identity| pending.remove(&identity))
        .collect();
    PendingSettingsDocuments(retired)
}

struct PendingSettingsDocuments(Vec<PendingSettingsDocument>);

impl PendingSettingsDocuments {
    fn cancel_before_start(self) {
        for pending in self.0 {
            pending.cancel_before_start();
        }
    }
}

fn receipt_for_change(
    change: &SettingChange,
    persistence_generation: Option<SettingsFileGeneration>,
    disposition: SettingsMutationDisposition,
) -> SettingsMutationReceipt {
    SettingsMutationReceipt {
        key: change.key.clone(),
        scope: change.scope,
        authority_generation: change.revision,
        persistence_generation,
        requires_restart: change.requires_restart,
        disposition,
    }
}
