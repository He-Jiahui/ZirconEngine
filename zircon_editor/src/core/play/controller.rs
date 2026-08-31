use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::editing::operation::{DeferredOperationInvocation, EditOperationTarget};
use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, ModeMessage, PlayStateKind,
    SharedEditorMessageBus, TOPIC_MODE,
};

use super::{
    NoopPlayBackend, NoopPluginBridgeActivation, PendingEditApplyBudget, PendingEditApplyReport,
    PendingEditDecisionPrompt, PendingEditDiscardReport, PendingEditIntent, PendingEditPage,
    PendingEditPageCursor, PendingEditQueueSummary, PlayBackendPoll, PlayCleanupFailure,
    PlayDomainLink, PlayDomainLinkError, PlayEditBeginError, PlayEditProtection,
    PlayEditResolutionError, PlayEditRoute, PlayEditRouteError, PlayKind, PlayMode, PlayModeKind,
    PlaySessionError, PlayStartRequest, PlayTransitionCause, PlayTransitionReport,
    PluginBridgeActivationReport, SharedPlayBackend, SharedPluginBridgeActivation,
};
use crate::core::gateway::EditorRuntimeGatewayHandle;

mod preview_routing;
mod runtime_ownership;

#[cfg(test)]
mod source_guards;

pub struct PlaySessionController {
    transition_gate: Mutex<()>,
    terminal_gateway_detach: AtomicBool,
    mode: RwLock<PlayMode>,
    preferred_kind: RwLock<PlayKind>,
    message_bus: SharedEditorMessageBus,
    plugin_activation: RwLock<SharedPluginBridgeActivation>,
    backend: RwLock<SharedPlayBackend>,
    session_ownership: Mutex<Option<PlaySessionOwnership>>,
    edit_protection: PlayEditProtection,
    play_domain: PlayDomainLink,
}

#[derive(Clone)]
enum PlaySessionOwnership {
    Active {
        kind: PlayKind,
        backend: SharedPlayBackend,
        activation: SharedPluginBridgeActivation,
    },
    Terminal {
        kind: PlayKind,
        backend: Option<SharedPlayBackend>,
        activation: Option<SharedPluginBridgeActivation>,
    },
}

impl Default for PlaySessionController {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaySessionController {
    pub fn new() -> Self {
        Self::with_message_bus(SharedEditorMessageBus::default())
    }

    pub fn with_message_bus(message_bus: SharedEditorMessageBus) -> Self {
        Self::with_message_bus_and_play_gateway(message_bus, EditorRuntimeGatewayHandle::detached())
    }

    pub(crate) fn with_message_bus_and_play_gateway(
        message_bus: SharedEditorMessageBus,
        play_gateway: EditorRuntimeGatewayHandle,
    ) -> Self {
        Self {
            transition_gate: Mutex::new(()),
            terminal_gateway_detach: AtomicBool::new(false),
            mode: RwLock::new(PlayMode::Edit),
            preferred_kind: RwLock::new(PlayKind::Play),
            message_bus,
            plugin_activation: RwLock::new(Arc::new(NoopPluginBridgeActivation)),
            backend: RwLock::new(Arc::new(NoopPlayBackend)),
            session_ownership: Mutex::new(None),
            edit_protection: PlayEditProtection::default(),
            play_domain: PlayDomainLink::with_gateway_handle(play_gateway),
        }
    }

    pub fn mode(&self) -> PlayModeKind {
        self.mode
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .kind()
    }

    pub fn mode_snapshot(&self) -> PlayMode {
        self.mode
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn preferred_kind(&self) -> PlayKind {
        *self
            .preferred_kind
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_preferred_kind(&self, kind: PlayKind) -> bool {
        let mut preferred = self
            .preferred_kind
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *preferred == kind {
            return false;
        }
        *preferred = kind;
        true
    }

    pub fn set_plugin_activation(&self, activation: SharedPluginBridgeActivation) {
        *self
            .plugin_activation
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = activation;
    }

    pub fn set_play_backend(&self, backend: SharedPlayBackend) {
        *self
            .backend
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = backend;
    }

    pub fn request_play(
        &self,
        request: PlayStartRequest,
    ) -> Result<PlayTransitionReport, PlaySessionError> {
        let transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let from = self.mode();
        if from == PlayModeKind::Edit {
            if self.session_ownership_pending() {
                drop(transition);
                return Err(PlaySessionError::InvalidTransition {
                    mode: from,
                    event: "request_play_with_pending_backend_retirement",
                });
            }
            if let Some(reason) = self.edit_protection.play_start_blocker() {
                drop(transition);
                return Err(Self::play_start_blocker_error(reason));
            }
            if self.play_domain.attached_domain().is_some() {
                drop(transition);
                return Err(PlaySessionError::InvalidTransition {
                    mode: from,
                    event: "request_play_with_attached_gateway",
                });
            }
        }
        let result = match self.mode_snapshot() {
            PlayMode::Edit if request.requires_build => {
                self.replace_mode(PlayMode::Building {
                    request,
                    play_after_build: true,
                });
                Ok(PlayTransitionReport::changed(
                    PlayModeKind::Building,
                    PluginBridgeActivationReport::default(),
                    Vec::new(),
                    false,
                    PlayTransitionCause::Started,
                ))
            }
            PlayMode::Edit => self.activate_and_enter_playing(request),
            PlayMode::Building {
                request,
                play_after_build,
            } => {
                if !play_after_build {
                    self.replace_mode(PlayMode::Building {
                        request,
                        play_after_build: true,
                    });
                    let report = PlayTransitionReport::changed(
                        PlayModeKind::Building,
                        PluginBridgeActivationReport::default(),
                        Vec::new(),
                        false,
                        PlayTransitionCause::Started,
                    );
                    drop(transition);
                    return Ok(report);
                }
                Ok(PlayTransitionReport::unchanged(PlayModeKind::Building))
            }
            PlayMode::Playing { .. } | PlayMode::CleanupFailed { .. } => {
                Err(PlaySessionError::InvalidTransition {
                    mode: self.mode(),
                    event: "request_play",
                })
            }
        };
        drop(transition);
        match &result {
            Ok(report) => self.publish_mode_transition(from, report.mode),
            Err(_) => {
                let to = self.mode();
                if to != from {
                    self.publish_mode_transition(from, to);
                }
            }
        }
        result
    }

    pub fn on_build_finished(
        &self,
        succeeded: bool,
    ) -> Result<PlayTransitionReport, PlaySessionError> {
        let transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let from = self.mode();
        let PlayMode::Building {
            request,
            play_after_build,
        } = self.mode_snapshot()
        else {
            drop(transition);
            return Err(PlaySessionError::InvalidTransition {
                mode: from,
                event: "build_finished",
            });
        };

        if !succeeded || !play_after_build {
            self.replace_mode(PlayMode::Edit);
            let report = PlayTransitionReport::changed(
                PlayModeKind::Edit,
                PluginBridgeActivationReport::default(),
                Vec::new(),
                false,
                PlayTransitionCause::BuildFailed,
            );
            drop(transition);
            self.publish_mode_transition(from, report.mode);
            return Ok(report);
        }

        let result = self.activate_and_enter_playing(request);
        let transition_to = match &result {
            Ok(report) => report.mode,
            Err(_) => {
                // Ordinary activation/start failures do not commit a mode. Attachment rollback
                // may commit Playing or CleanupFailed when runtime ownership is still live.
                if self.mode() == PlayModeKind::Building {
                    self.replace_mode(PlayMode::Edit);
                }
                self.mode()
            }
        };
        drop(transition);
        self.publish_mode_transition(from, transition_to);
        result
    }

    pub fn request_stop(&self) -> Result<PlayTransitionReport, PlaySessionError> {
        let transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let from = self.mode();
        let result = match self.mode_snapshot() {
            PlayMode::Edit => Ok(PlayTransitionReport::unchanged(PlayModeKind::Edit)),
            PlayMode::Building { .. } => {
                self.replace_mode(PlayMode::Edit);
                Ok(PlayTransitionReport::changed(
                    PlayModeKind::Edit,
                    PluginBridgeActivationReport::default(),
                    Vec::new(),
                    false,
                    PlayTransitionCause::Stopped,
                ))
            }
            PlayMode::Playing { kind } => {
                let Some((owned_kind, backend, activation)) = self.active_session_owners() else {
                    return Err(PlaySessionError::InvalidTransition {
                        mode: from,
                        event: "request_stop_without_active_session_owner",
                    });
                };
                if owned_kind != kind {
                    return Err(PlaySessionError::InvalidTransition {
                        mode: from,
                        event: "request_stop_with_mismatched_session_owner",
                    });
                }
                let backend_report = backend.stop().map_err(PlaySessionError::BackendStop)?;
                Ok(self.complete_runtime_stop(
                    kind,
                    backend,
                    activation,
                    backend_report.diagnostics,
                    backend_report.retirement_pending,
                    PlayTransitionCause::Stopped,
                ))
            }
            PlayMode::CleanupFailed { kind, failure } => Ok(self.retry_cleanup(kind, failure)),
        };
        drop(transition);
        if let Ok(report) = &result {
            self.publish_mode_transition(from, report.mode);
        }
        result
    }

    pub fn poll_backend(&self) -> Result<PlayTransitionReport, PlaySessionError> {
        let transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mode = self.mode();
        if mode != PlayModeKind::Playing {
            drop(transition);
            return Ok(PlayTransitionReport::unchanged(mode));
        }
        let PlayMode::Playing { kind } = self.mode_snapshot() else {
            unreachable!("playing mode changed while its transition gate was held");
        };
        let Some((owned_kind, backend, activation)) = self.active_session_owners() else {
            return Err(PlaySessionError::InvalidTransition {
                mode,
                event: "poll_backend_without_active_session_owner",
            });
        };
        if owned_kind != kind {
            return Err(PlaySessionError::InvalidTransition {
                mode,
                event: "poll_backend_with_mismatched_session_owner",
            });
        }
        let result = match backend.poll().map_err(PlaySessionError::BackendPoll)? {
            PlayBackendPoll::Running { diagnostics } => Ok(
                PlayTransitionReport::unchanged_with_backend(PlayModeKind::Playing, diagnostics),
            ),
            PlayBackendPoll::Exited {
                exit_code,
                diagnostics,
            } => Ok(self.complete_runtime_stop(
                kind,
                backend,
                activation,
                diagnostics,
                false,
                PlayTransitionCause::Crashed { exit_code },
            )),
        };
        drop(transition);
        if let Ok(report) = &result {
            self.publish_mode_transition(mode, report.mode);
        }
        result
    }

    pub fn route_edit(
        &self,
        target: EditOperationTarget,
        deferred: DeferredOperationInvocation,
    ) -> Result<PlayEditRoute, PlayEditRouteError> {
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.edit_protection.route(target, deferred)
    }

    pub fn pending_edits_summary(&self) -> PendingEditQueueSummary {
        self.edit_protection.pending_summary()
    }

    pub fn pending_edits_page(
        &self,
        after: Option<PendingEditPageCursor>,
        limit: usize,
    ) -> PendingEditPage {
        self.edit_protection.pending_page(after, limit)
    }

    pub fn pending_edit_decision_prompt(&self) -> Option<PendingEditDecisionPrompt> {
        self.edit_protection.pending_decision_prompt()
    }

    pub(crate) fn with_pending_edit_decision_prompt<E>(
        &self,
        publish: impl FnOnce(&PendingEditDecisionPrompt) -> Result<(), E>,
    ) -> Result<bool, E> {
        self.edit_protection.with_pending_decision_prompt(publish)
    }

    pub fn apply_pending_edits<E>(
        &self,
        budget: PendingEditApplyBudget,
        apply: impl FnMut(&PendingEditIntent) -> Result<(), E>,
    ) -> Result<PendingEditApplyReport<E>, PlayEditResolutionError> {
        self.edit_protection.apply_pending(budget, apply)
    }

    pub fn discard_pending_edits(
        &self,
    ) -> Result<PendingEditDiscardReport, PlayEditResolutionError> {
        self.edit_protection.discard_pending()
    }

    fn activate_and_enter_playing(
        &self,
        request: PlayStartRequest,
    ) -> Result<PlayTransitionReport, PlaySessionError> {
        let activation = self.plugin_activation();
        let report = activation
            .activate(request.project_root.as_deref())
            .map_err(PlaySessionError::PluginActivation)?;
        if let Err(reason) = self.edit_protection.begin_play(request.running_document) {
            let activation_rollback = activation.deactivate().err();
            if let Some(message) = &activation_rollback {
                self.retain_terminal_owners(request.kind, None, Some(Arc::clone(&activation)));
                self.replace_mode(PlayMode::CleanupFailed {
                    kind: request.kind,
                    failure: PlayCleanupFailure::PluginDeactivation {
                        message: message.clone(),
                    },
                });
            }
            return Err(PlaySessionError::EditProtectionStart {
                reason,
                activation_rollback,
            });
        }
        let backend = self.backend();
        let mut backend_report = match backend.start(&request) {
            Ok(report) => report,
            Err(failure) => {
                let (message, retirement_pending) = failure.into_parts();
                self.edit_protection.end_play();
                let activation_rollback = activation.deactivate().err();
                let retained_backend = retirement_pending.then(|| Arc::clone(&backend));
                let retained_activation = activation_rollback
                    .as_ref()
                    .map(|_| Arc::clone(&activation));
                self.retain_terminal_owners(request.kind, retained_backend, retained_activation);
                if retirement_pending {
                    self.replace_mode(PlayMode::CleanupFailed {
                        kind: request.kind,
                        failure: PlayCleanupFailure::BackendRetirement {
                            message: message.clone(),
                            plugin_deactivation: activation_rollback.clone(),
                        },
                    });
                } else if let Some(rollback) = &activation_rollback {
                    self.replace_mode(PlayMode::CleanupFailed {
                        kind: request.kind,
                        failure: PlayCleanupFailure::PluginDeactivation {
                            message: rollback.clone(),
                        },
                    });
                }
                return Err(PlaySessionError::BackendStart {
                    message,
                    activation_rollback,
                });
            }
        };
        let backend_attachable = backend_report.attachable();
        if let Some(gateway) = backend_report.take_gateway() {
            if let Err(error) = self.play_domain.attach(gateway) {
                return Err(self.rollback_failed_gateway_attach(
                    request.kind,
                    &backend,
                    &activation,
                    error,
                ));
            }
        }
        self.replace_session_ownership(Some(PlaySessionOwnership::Active {
            kind: request.kind,
            backend,
            activation,
        }));
        self.replace_mode(PlayMode::Playing { kind: request.kind });
        Ok(PlayTransitionReport::changed(
            PlayModeKind::Playing,
            report,
            backend_report.diagnostics,
            backend_attachable,
            PlayTransitionCause::Started,
        ))
    }

    fn rollback_failed_gateway_attach(
        &self,
        kind: PlayKind,
        backend: &SharedPlayBackend,
        activation: &SharedPluginBridgeActivation,
        error: PlayDomainLinkError,
    ) -> PlaySessionError {
        let mut message = format!("failed to attach embedded play gateway: {error}");
        let stop_report = match backend.stop() {
            Ok(report) => report,
            Err(stop_error) => {
                // Starting the backend already succeeded. If stop fails, its runtime remains the
                // physical truth and request_stop must be allowed to retry that operation.
                self.replace_session_ownership(Some(PlaySessionOwnership::Active {
                    kind,
                    backend: Arc::clone(backend),
                    activation: Arc::clone(activation),
                }));
                self.replace_mode(PlayMode::Playing { kind });
                message.push_str(&format!(
                    "; backend remains live because stop rollback failed: {stop_error}"
                ));
                return PlaySessionError::BackendStart {
                    message,
                    activation_rollback: None,
                };
            }
        };

        self.edit_protection.end_play();
        let activation_rollback = activation.deactivate().err();
        let (retained_backend, retirement_error) = if stop_report.retirement_pending {
            match backend.retire() {
                Ok(_) => (None, None),
                Err(error) => (Some(Arc::clone(backend)), Some(error)),
            }
        } else {
            (None, None)
        };
        let retained_activation = activation_rollback.as_ref().map(|_| Arc::clone(activation));
        self.retain_terminal_owners(kind, retained_backend, retained_activation);

        if let Some(retirement_error) = retirement_error {
            let failure = PlayCleanupFailure::BackendRetirement {
                message: retirement_error.clone(),
                plugin_deactivation: activation_rollback.clone(),
            };
            self.replace_mode(PlayMode::CleanupFailed { kind, failure });
            message.push_str(&format!(
                "; backend retirement rollback remains pending: {retirement_error}"
            ));
        } else if let Some(activation_rollback) = &activation_rollback {
            self.replace_mode(PlayMode::CleanupFailed {
                kind,
                failure: PlayCleanupFailure::PluginDeactivation {
                    message: activation_rollback.clone(),
                },
            });
        } else {
            self.replace_mode(PlayMode::Edit);
        }

        PlaySessionError::BackendStart {
            message,
            activation_rollback,
        }
    }

    /// Completes the local runtime stop before attempting editor-plugin restoration.
    ///
    /// Once the backend has stopped, edit-domain protection must be released even if plugin
    /// restoration needs a later retry. Keeping that physical terminal fact in a distinct mode
    /// prevents the UI from presenting an exited runtime as still playing.
    fn complete_runtime_stop(
        &self,
        kind: PlayKind,
        backend: SharedPlayBackend,
        activation: SharedPluginBridgeActivation,
        backend_diagnostics: Vec<String>,
        retirement_pending: bool,
        terminal_cause: PlayTransitionCause,
    ) -> PlayTransitionReport {
        let pending_edit_prompt = self.edit_protection.end_play();
        match activation.deactivate() {
            Ok(report) => {
                self.retain_terminal_owners(kind, retirement_pending.then_some(backend), None);
                self.replace_mode(PlayMode::Edit);
                PlayTransitionReport::changed(
                    PlayModeKind::Edit,
                    report,
                    backend_diagnostics,
                    false,
                    terminal_cause,
                )
                .with_pending_edit_prompt(pending_edit_prompt)
            }
            Err(message) => {
                self.retain_terminal_owners(
                    kind,
                    retirement_pending.then_some(backend),
                    Some(activation),
                );
                let failure = PlayCleanupFailure::PluginDeactivation { message };
                self.replace_mode(PlayMode::CleanupFailed {
                    kind,
                    failure: failure.clone(),
                });
                PlayTransitionReport::changed(
                    PlayModeKind::CleanupFailed,
                    PluginBridgeActivationReport::default(),
                    backend_diagnostics,
                    false,
                    PlayTransitionCause::CleanupFailed { failure },
                )
                .with_pending_edit_prompt(pending_edit_prompt)
            }
        }
    }

    fn retry_cleanup(&self, kind: PlayKind, failure: PlayCleanupFailure) -> PlayTransitionReport {
        if matches!(failure, PlayCleanupFailure::BackendRetirement { .. }) {
            return PlayTransitionReport::cleanup_failed(false, Vec::new(), failure);
        }
        self.retry_plugin_cleanup(kind, failure)
    }

    fn retry_plugin_cleanup(
        &self,
        kind: PlayKind,
        prior_failure: PlayCleanupFailure,
    ) -> PlayTransitionReport {
        let Some(activation) = self.terminal_plugin_activation(kind) else {
            return PlayTransitionReport::cleanup_failed(false, Vec::new(), prior_failure);
        };
        match activation.deactivate() {
            Ok(report) => {
                self.clear_terminal_plugin_activation(kind);
                self.replace_mode(PlayMode::Edit);
                PlayTransitionReport::changed(
                    PlayModeKind::Edit,
                    report,
                    Vec::new(),
                    false,
                    PlayTransitionCause::Stopped,
                )
            }
            Err(message) => {
                let failure = PlayCleanupFailure::PluginDeactivation { message };
                self.replace_mode(PlayMode::CleanupFailed {
                    kind,
                    failure: failure.clone(),
                });
                PlayTransitionReport::cleanup_failed(false, Vec::new(), failure)
            }
        }
    }

    fn active_session_owners(
        &self,
    ) -> Option<(PlayKind, SharedPlayBackend, SharedPluginBridgeActivation)> {
        let ownership = self
            .session_ownership
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match ownership.as_ref()? {
            PlaySessionOwnership::Active {
                kind,
                backend,
                activation,
            } => Some((*kind, Arc::clone(backend), Arc::clone(activation))),
            PlaySessionOwnership::Terminal { .. } => None,
        }
    }

    fn session_ownership_pending(&self) -> bool {
        self.session_ownership
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    fn terminal_plugin_activation(
        &self,
        expected_kind: PlayKind,
    ) -> Option<SharedPluginBridgeActivation> {
        let ownership = self
            .session_ownership
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match ownership.as_ref()? {
            PlaySessionOwnership::Terminal {
                kind, activation, ..
            } if *kind == expected_kind => activation.as_ref().map(Arc::clone),
            _ => None,
        }
    }

    fn clear_terminal_plugin_activation(&self, expected_kind: PlayKind) {
        let mut ownership = self
            .session_ownership
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let should_clear = match ownership.as_mut() {
            Some(PlaySessionOwnership::Terminal {
                kind,
                backend,
                activation,
            }) if *kind == expected_kind => {
                *activation = None;
                backend.is_none()
            }
            _ => false,
        };
        if should_clear {
            *ownership = None;
        }
    }

    fn retain_terminal_owners(
        &self,
        kind: PlayKind,
        backend: Option<SharedPlayBackend>,
        activation: Option<SharedPluginBridgeActivation>,
    ) {
        self.replace_session_ownership((backend.is_some() || activation.is_some()).then_some(
            PlaySessionOwnership::Terminal {
                kind,
                backend,
                activation,
            },
        ));
    }

    fn replace_session_ownership(&self, ownership: Option<PlaySessionOwnership>) {
        *self
            .session_ownership
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = ownership;
    }

    fn plugin_activation(&self) -> SharedPluginBridgeActivation {
        Arc::clone(
            &self
                .plugin_activation
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    fn backend(&self) -> SharedPlayBackend {
        Arc::clone(
            &self
                .backend
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    fn replace_mode(&self, mode: PlayMode) {
        *self
            .mode
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = mode;
    }

    // Callers release transition_gate before this notification so subscribers cannot extend a mode change.
    fn publish_mode_transition(&self, from: PlayModeKind, to: PlayModeKind) {
        if from == to {
            return;
        }
        let topic = EditorTopic::parse(TOPIC_MODE).expect("the built-in mode topic must be valid");
        self.message_bus.publish(
            topic,
            EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
                from: play_state_kind(from),
                to: play_state_kind(to),
            })),
        );
    }

    fn play_start_blocker_error(reason: PlayEditBeginError) -> PlaySessionError {
        match reason {
            PlayEditBeginError::PendingDecisionRequired { pending_count } => {
                PlaySessionError::PendingEditDecisionRequired { pending_count }
            }
            PlayEditBeginError::ResolutionInProgress => {
                PlaySessionError::PendingEditResolutionInProgress
            }
            PlayEditBeginError::AlreadyPlaying => PlaySessionError::InvalidTransition {
                mode: PlayModeKind::Playing,
                event: "request_play",
            },
        }
    }
}

fn play_state_kind(mode: PlayModeKind) -> PlayStateKind {
    match mode {
        PlayModeKind::Edit => PlayStateKind::Edit,
        PlayModeKind::Building => PlayStateKind::Building,
        PlayModeKind::Playing => PlayStateKind::Playing,
        PlayModeKind::CleanupFailed => PlayStateKind::CleanupFailed,
    }
}
