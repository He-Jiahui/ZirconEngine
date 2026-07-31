use std::sync::{Arc, Mutex, RwLock};

use crate::core::editing::operation::DeferredOperationInvocation;
use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, ModeMessage, PlayStateKind,
    SharedEditorMessageBus, TOPIC_MODE,
};

use super::{
    NoopPlayBackend, NoopPluginBridgeActivation, PendingEditApplyBudget, PendingEditApplyReport,
    PendingEditDecisionPrompt, PendingEditDiscardReport, PendingEditIntent, PendingEditPage,
    PendingEditPageCursor, PendingEditQueueSummary, PlayBackendPoll, PlayDomainLink,
    PlayDomainLinkError, PlayEditBeginError, PlayEditProtection, PlayEditResolutionError,
    PlayEditRoute, PlayEditRouteError, PlayEditTarget, PlayInstanceId, PlayMode, PlayModeKind,
    PlaySessionError, PlayStartRequest, PlayTransitionCause, PlayTransitionReport,
    PluginBridgeActivationReport, SharedPlayBackend, SharedPluginBridgeActivation, WorldDomain,
};
use crate::core::gateway::{EditorRuntimeGatewayHandle, SharedEditorRuntimeGateway};

pub struct PlaySessionController {
    transition_gate: Mutex<()>,
    mode: RwLock<PlayMode>,
    message_bus: SharedEditorMessageBus,
    plugin_activation: RwLock<SharedPluginBridgeActivation>,
    backend: RwLock<SharedPlayBackend>,
    edit_protection: PlayEditProtection,
    play_domain: PlayDomainLink,
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
        Self {
            transition_gate: Mutex::new(()),
            mode: RwLock::new(PlayMode::Edit),
            message_bus,
            plugin_activation: RwLock::new(Arc::new(NoopPluginBridgeActivation)),
            backend: RwLock::new(Arc::new(NoopPlayBackend)),
            edit_protection: PlayEditProtection::default(),
            play_domain: PlayDomainLink::default(),
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

    pub fn attach_play_gateway(
        &self,
        gateway: SharedEditorRuntimeGateway,
    ) -> Result<PlayInstanceId, PlayDomainLinkError> {
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.play_domain.attach(gateway)
    }

    pub fn detach_play_gateway(&self, instance: PlayInstanceId) -> Result<(), PlayDomainLinkError> {
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.play_domain.detach(instance)
    }

    pub fn attached_world_domain(&self) -> Option<WorldDomain> {
        self.play_domain.attached_domain()
    }

    pub fn play_gateway(&self, instance: PlayInstanceId) -> Option<EditorRuntimeGatewayHandle> {
        self.play_domain.gateway(instance)
    }

    pub(crate) fn play_gateway_handle(&self) -> EditorRuntimeGatewayHandle {
        self.play_domain.gateway_handle()
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
            if let Some(reason) = self.edit_protection.play_start_blocker() {
                drop(transition);
                return Err(Self::play_start_blocker_error(reason));
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
            PlayMode::Playing { .. } => Err(PlaySessionError::InvalidTransition {
                mode: PlayModeKind::Playing,
                event: "request_play",
            }),
        };
        drop(transition);
        if let Ok(report) = &result {
            self.publish_mode_transition(from, report.mode);
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
        drop(transition);
        if let Ok(report) = &result {
            self.publish_mode_transition(from, report.mode);
        }
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
            PlayMode::Playing { .. } => {
                let backend = self.backend();
                let backend_report = backend.stop().map_err(PlaySessionError::BackendStop)?;
                let activation = self.plugin_activation();
                let report = activation
                    .deactivate()
                    .map_err(PlaySessionError::PluginActivation)?;
                let pending_edit_prompt = self.edit_protection.end_play();
                self.replace_mode(PlayMode::Edit);
                Ok(PlayTransitionReport::changed(
                    PlayModeKind::Edit,
                    report,
                    backend_report.diagnostics,
                    false,
                    PlayTransitionCause::Stopped,
                )
                .with_pending_edit_prompt(pending_edit_prompt))
            }
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
        let backend = self.backend();
        let result = match backend.poll().map_err(PlaySessionError::BackendPoll)? {
            PlayBackendPoll::Running { diagnostics } => Ok(
                PlayTransitionReport::unchanged_with_backend(PlayModeKind::Playing, diagnostics),
            ),
            PlayBackendPoll::Exited {
                exit_code,
                diagnostics,
            } => {
                let activation = self.plugin_activation();
                let report = activation
                    .deactivate()
                    .map_err(PlaySessionError::PluginActivation)?;
                let pending_edit_prompt = self.edit_protection.end_play();
                self.replace_mode(PlayMode::Edit);
                Ok(PlayTransitionReport::changed(
                    PlayModeKind::Edit,
                    report,
                    diagnostics,
                    false,
                    PlayTransitionCause::Crashed { exit_code },
                )
                .with_pending_edit_prompt(pending_edit_prompt))
            }
        };
        drop(transition);
        if let Ok(report) = &result {
            self.publish_mode_transition(mode, report.mode);
        }
        result
    }

    pub fn route_edit(
        &self,
        target: PlayEditTarget,
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
            return Err(PlaySessionError::EditProtectionStart {
                reason,
                activation_rollback,
            });
        }
        let backend = self.backend();
        let backend_report = match backend.start(&request) {
            Ok(report) => report,
            Err(message) => {
                self.edit_protection.end_play();
                let activation_rollback = activation.deactivate().err();
                return Err(PlaySessionError::BackendStart {
                    message,
                    activation_rollback,
                });
            }
        };
        self.replace_mode(PlayMode::Playing { kind: request.kind });
        Ok(PlayTransitionReport::changed(
            PlayModeKind::Playing,
            report,
            backend_report.diagnostics,
            backend_report.attachable,
            PlayTransitionCause::Started,
        ))
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
    }
}

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn inactive_backend_poll_reads_the_mode_once() {
        let source = include_str!("controller.rs");
        let body = source
            .split("pub fn poll_backend")
            .nth(1)
            .and_then(|body| body.split("pub fn route_edit").next())
            .expect("poll backend body should remain available");

        assert_eq!(body.matches("self.mode()").count(), 1);
    }
}
