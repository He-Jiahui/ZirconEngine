use std::sync::atomic::Ordering;

use crate::core::gateway::EditorRuntimeFrameDemand;
use crate::core::play::{PlayTransitionCause, WorldDomain};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistry,
};

use super::super::EditorRuntimeEventPumpError;
use super::EditorHostEventController;

impl EditorHostEventController {
    pub fn register_runtime_event_consumers(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _registration_guard = self
            .plugin_registration_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.runtime_event_consumers.register(registry)
    }

    pub fn begin_runtime_event_consumers(&self) -> Result<(), EditorRuntimeEventConsumerError> {
        if self.play_sessions.attached_world_domain().is_none() {
            return Err(EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "play.domain".to_string(),
                message: "no play gateway is attached".to_string(),
            });
        }
        let enabled_capabilities = self
            .shell
            .lock()
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let play_session_id = self
            .next_play_session_generation
            .fetch_add(1, Ordering::Relaxed);
        self.runtime_event_consumers
            .begin_play_session(play_session_id, &enabled_capabilities)
    }

    pub fn pump_runtime_event_consumers(
        &self,
    ) -> Result<EditorRuntimeFrameDemand, EditorRuntimeEventPumpError> {
        self.pump_pending_play_decision_receipts()?;
        let backend_transition = self
            .play_sessions
            .poll_backend()
            .map_err(|source| EditorRuntimeEventPumpError::BackendPoll { source })?;
        let terminal_gateway_pending = !backend_transition.mode.has_active_runtime()
            && self.play_sessions.attached_world_domain().is_some();
        if (backend_transition.changed && !backend_transition.mode.has_active_runtime())
            || terminal_gateway_pending
        {
            if self.runtime_event_consumer_session_active() {
                self.end_runtime_event_consumers()?;
            }
            let play_world_sync = self.shutdown_play_world_sync();
            if play_world_sync.failed_count() > 0 {
                self.log_play_backend_diagnostics(&[format!(
                    "play.world_sync: {} remote watch cleanup operation(s) failed",
                    play_world_sync.failed_count()
                )]);
            }
            self.detach_terminal_play_gateway()
                .map_err(|source| EditorRuntimeEventPumpError::PlayGatewayDetach { source })?;
            let retirement_transition = self
                .play_sessions
                .retire_terminal_backend()
                .map_err(|source| EditorRuntimeEventPumpError::PlayBackendRetirement { source })?;
            self.log_play_backend_diagnostics(&retirement_transition.backend_diagnostics);
            let terminal_transition = if matches!(
                retirement_transition.cause,
                PlayTransitionCause::CleanupFailed { .. }
            ) {
                &retirement_transition
            } else {
                &backend_transition
            };
            let pending_edit_decision_error =
                self.reconcile_pending_play_decision_from_controller().err();
            let editor_state_exit_error = {
                let mut shell = self.shell.lock();
                match shell.state.exit_play_mode() {
                    Ok(_) => {
                        let preview_restore_suffix = shell
                            .restore_pre_play_view()
                            .err()
                            .map(|error| {
                                format!("; previous editor view could not be restored: {error}")
                            })
                            .unwrap_or_default();
                        match &terminal_transition.cause {
                            PlayTransitionCause::Crashed { exit_code } => {
                                shell.state.set_status_line(format!(
                                    "Runtime preview exited unexpectedly (code {exit_code:?}){preview_restore_suffix}"
                                ))
                            }
                            PlayTransitionCause::CleanupFailed { failure } => {
                                shell.state.set_status_line(format!(
                                    "Runtime preview exited, but editor plugin cleanup is pending: {failure}{preview_restore_suffix}"
                                ))
                            }
                            _ => shell.state.set_status_line(format!(
                                "Runtime preview stopped{preview_restore_suffix}"
                            )),
                        }
                        None
                    }
                    Err(error) => {
                        shell.state.set_status_line(format!(
                            "Runtime preview stopped, but editor state remains in play mode for retry: {error}"
                        ));
                        Some(error)
                    }
                }
            };
            self.refresh_reflection();
            if let Some(decision_error) = pending_edit_decision_error {
                if let Some(exit_error) = editor_state_exit_error {
                    return Err(
                        EditorRuntimeEventPumpError::PendingDecisionPublishAndStateRestore {
                            decision: decision_error,
                            restore: exit_error,
                        },
                    );
                }
                return Err(EditorRuntimeEventPumpError::PendingDecisionPublish(
                    decision_error,
                ));
            }
            if let Some(source) = editor_state_exit_error {
                return Err(EditorRuntimeEventPumpError::StateRestore { source });
            }
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        }
        if self
            .runtime_event_consumers
            .active_play_session_id()
            .is_none()
        {
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        }
        let enabled_capabilities = self
            .shell
            .lock()
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        self.runtime_event_consumers
            .reconcile_enabled_capabilities(&enabled_capabilities)?;
        let Some(WorldDomain::Play(instance)) = self.play_sessions.attached_world_domain() else {
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        };
        let Some(play_gateway) = self.play_sessions.play_gateway(instance) else {
            return Ok(EditorRuntimeFrameDemand::OnDemand);
        };
        let frame_demand = play_gateway.tick_frame().map_err(|message| {
            EditorRuntimeEventConsumerError::Gateway {
                consumer_id: "runtime.frame.tick".to_string(),
                message: message.to_string(),
            }
        })?;
        self.runtime_event_consumers.pump()?;
        Ok(frame_demand)
    }

    pub fn end_runtime_event_consumers(&self) -> Result<(), EditorRuntimeEventConsumerError> {
        let play_session_id = self
            .runtime_event_consumers
            .active_play_session_id()
            .ok_or(EditorRuntimeEventConsumerError::NoActiveSession)?;
        self.runtime_event_consumers
            .end_play_session(play_session_id)
    }

    pub fn runtime_event_consumer_session_active(&self) -> bool {
        self.runtime_event_consumers
            .active_play_session_id()
            .is_some()
    }
}
