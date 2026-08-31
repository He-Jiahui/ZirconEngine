use crate::core::editing::engine::EditCommandError;
use crate::core::gateway::GatewaySessionIdentity;
use crate::core::play::{
    PlayCleanupFailure, PlayInstanceId, PlayModeKind, PlaySessionError,
    PlayTerminalGatewayDetachError, PlayTransitionCause, PlayTransitionReport, WorldDomain,
};
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerError;
use crate::core::sync::WorldSyncShutdownReceipt;
use crate::ui::workbench::state::EditorStateOperationError;

use super::EditorHostEventController;

pub type EditorTerminalPlayDetachError = PlayTerminalGatewayDetachError<EditCommandError>;

/// Local and remote outcome of retiring runtime event consumers during host shutdown.
#[derive(Debug)]
pub enum RuntimeEventConsumerShutdownDisposition {
    /// No runtime consumer session was active when shutdown began.
    NotActive,
    /// Every consumer reached the local terminal state and remote cleanup succeeded.
    Retired,
    /// Consumers reached the local terminal state, but remote cleanup remains observable.
    RetiredWithCleanupFailure {
        error: EditorRuntimeEventConsumerError,
    },
    /// A concurrent lifecycle operation prevented local terminal retirement in this attempt.
    RetirementDeferred {
        error: EditorRuntimeEventConsumerError,
    },
}

/// Outcome of explicitly stopping the play backend and plugin activation before link retirement.
#[derive(Debug)]
pub enum RuntimePlaySessionShutdownDisposition {
    /// The controller was already in Edit mode when shutdown began.
    NotPlaying,
    /// The backend and plugin activation stopped, leaving the controller in Edit mode.
    Stopped { report: PlayTransitionReport },
    /// The runtime backend stopped, but plugin restoration remains retryable in CleanupFailed.
    StoppedWithCleanupFailure {
        report: PlayTransitionReport,
        failure: PlayCleanupFailure,
    },
    /// The backend or plugin activation refused to stop; the play link stays attached for retry.
    RetirementDeferred {
        mode: PlayModeKind,
        error: PlaySessionError,
    },
}

impl RuntimePlaySessionShutdownDisposition {
    pub const fn is_locally_terminal(&self) -> bool {
        matches!(
            self,
            Self::NotPlaying | Self::Stopped { .. } | Self::StoppedWithCleanupFailure { .. }
        )
    }
}

/// Outcome of restoring the retained editor state after its runtime Play backend has stopped.
#[derive(Debug)]
pub enum EditorPlayStateShutdownDisposition {
    /// The retained shell had already restored its authoring state.
    NotPlaying,
    /// The retained shell restored its authoring state for the terminal Play session.
    Restored,
    /// The runtime stopped, but the retained shell could not restore its authoring state yet.
    RestorationDeferred { error: EditorStateOperationError },
}

/// Ordered Play-only retirement used before a project generation is released.
///
/// Unlike full runtime-session shutdown this does not retire the editor's authoring-world sync
/// owner, so the retained host can safely open a subsequent project in the same process.
#[derive(Debug)]
pub struct EditorPlaySessionShutdownReceipt {
    event_consumers: RuntimeEventConsumerShutdownDisposition,
    play_world_sync: WorldSyncShutdownReceipt,
    play_session: RuntimePlaySessionShutdownDisposition,
    editor_state: EditorPlayStateShutdownDisposition,
    play_gateway: RuntimePlayGatewayShutdownDisposition,
    play_backend_retirement: RuntimePlayBackendRetirementDisposition,
}

impl EditorPlaySessionShutdownReceipt {
    pub fn event_consumers(&self) -> &RuntimeEventConsumerShutdownDisposition {
        &self.event_consumers
    }

    pub fn play_world_sync(&self) -> &WorldSyncShutdownReceipt {
        &self.play_world_sync
    }

    pub fn play_session(&self) -> &RuntimePlaySessionShutdownDisposition {
        &self.play_session
    }

    pub fn editor_state(&self) -> &EditorPlayStateShutdownDisposition {
        &self.editor_state
    }

    pub fn play_gateway(&self) -> &RuntimePlayGatewayShutdownDisposition {
        &self.play_gateway
    }

    pub fn play_backend_retirement(&self) -> &RuntimePlayBackendRetirementDisposition {
        &self.play_backend_retirement
    }

    /// A project may be released only after every project-owned Play resource is terminal.
    pub fn is_ready_for_project_close(&self) -> bool {
        matches!(
            &self.event_consumers,
            RuntimeEventConsumerShutdownDisposition::NotActive
                | RuntimeEventConsumerShutdownDisposition::Retired
                | RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure { .. }
        ) && matches!(
            &self.play_session,
            RuntimePlaySessionShutdownDisposition::NotPlaying
                | RuntimePlaySessionShutdownDisposition::Stopped { .. }
        ) && matches!(
            &self.editor_state,
            EditorPlayStateShutdownDisposition::NotPlaying
                | EditorPlayStateShutdownDisposition::Restored
        ) && matches!(
            &self.play_gateway,
            RuntimePlayGatewayShutdownDisposition::NotAttached
                | RuntimePlayGatewayShutdownDisposition::Detached { .. }
        ) && matches!(
            &self.play_backend_retirement,
            RuntimePlayBackendRetirementDisposition::NotPending
                | RuntimePlayBackendRetirementDisposition::Retired { .. }
        )
    }
}

/// Outcome of detaching the play-domain gateway at the terminal session boundary.
#[derive(Debug)]
pub enum RuntimePlayGatewayShutdownDisposition {
    /// No play domain was attached when shutdown reached the detach stage.
    NotAttached,
    /// The exact captured play instance and gateway identity were detached.
    Detached {
        instance: PlayInstanceId,
        identity: GatewaySessionIdentity,
    },
    /// The play backend remains active, so its identity-qualified link must remain attached.
    RetainedForActivePlay { mode: PlayModeKind },
    /// History preparation failed, or the captured play link could not be detached.
    RetirementDeferred {
        instance: PlayInstanceId,
        identity: GatewaySessionIdentity,
        error: EditorTerminalPlayDetachError,
    },
}

/// Outcome of releasing the App-owned backend lease after the play gateway is unreachable.
#[derive(Debug)]
pub enum RuntimePlayBackendRetirementDisposition {
    NotPending,
    Retired {
        report: PlayTransitionReport,
    },
    RetainedForActivePlay {
        mode: PlayModeKind,
    },
    RetainedForGatewayRetirement,
    RetirementDeferred {
        report: PlayTransitionReport,
        failure: PlayCleanupFailure,
    },
    RetirementRejected {
        error: PlaySessionError,
    },
}

/// Ordered receipt for editor-owned runtime-session retirement.
///
/// Consumer and watch retirement happen before play teardown. The identity-qualified play link
/// is detached once the backend is locally terminal, including retryable plugin cleanup failure.
#[derive(Debug)]
pub struct EditorRuntimeSessionShutdownReceipt {
    event_consumers: RuntimeEventConsumerShutdownDisposition,
    edit_world_sync: WorldSyncShutdownReceipt,
    play_world_sync: WorldSyncShutdownReceipt,
    play_session: RuntimePlaySessionShutdownDisposition,
    play_gateway: RuntimePlayGatewayShutdownDisposition,
    play_backend_retirement: RuntimePlayBackendRetirementDisposition,
}

impl EditorRuntimeSessionShutdownReceipt {
    pub fn event_consumers(&self) -> &RuntimeEventConsumerShutdownDisposition {
        &self.event_consumers
    }

    pub fn edit_world_sync(&self) -> &WorldSyncShutdownReceipt {
        &self.edit_world_sync
    }

    pub fn play_world_sync(&self) -> &WorldSyncShutdownReceipt {
        &self.play_world_sync
    }

    pub fn play_session(&self) -> &RuntimePlaySessionShutdownDisposition {
        &self.play_session
    }

    pub fn play_gateway(&self) -> &RuntimePlayGatewayShutdownDisposition {
        &self.play_gateway
    }

    pub fn play_backend_retirement(&self) -> &RuntimePlayBackendRetirementDisposition {
        &self.play_backend_retirement
    }
}

impl EditorHostEventController {
    /// Detaches the exact play gateway after the runtime backend reaches a terminal mode.
    ///
    /// Normal stop, crash handling, project close, and host shutdown share this path so none of
    /// them can leave a stopped session reachable through the stable play-domain handle.
    pub(in crate::ui::host) fn detach_terminal_play_gateway(
        &self,
    ) -> Result<Option<(PlayInstanceId, GatewaySessionIdentity)>, EditorTerminalPlayDetachError>
    {
        self.retire_play_gizmo_local_state();
        let discard_history = |instance| {
            self.context
                .transactions()
                .discard_play_history(instance)
                .map(|_| ())
        };
        self.play_sessions
            .detach_terminal_play_gateway(discard_history)
    }

    /// Stops the project-owned Play session without retiring the editor's authoring runtime.
    ///
    /// Project Close calls this before releasing project documents, plugins, or session locks.
    /// A retryable plugin cleanup failure keeps the project generation open even though its
    /// process and gateway are already terminal.
    pub fn shutdown_play_session_for_project_close(&self) -> EditorPlaySessionShutdownReceipt {
        let event_consumers = self.shutdown_runtime_event_consumers();
        let play_world_sync = self.shutdown_play_world_sync();
        let play_session = self.shutdown_play_session();
        let editor_state = self.shutdown_editor_play_state(&play_session);
        let play_gateway = self.shutdown_play_gateway(&play_session);
        let play_backend_retirement = self.shutdown_play_backend_retirement(&play_gateway);

        EditorPlaySessionShutdownReceipt {
            event_consumers,
            play_world_sync,
            play_session,
            editor_state,
            play_gateway,
            play_backend_retirement,
        }
    }

    /// Retires editor-owned runtime session state in terminal dependency order.
    ///
    /// This is an explicit one-pass local retirement plus best-effort origin cleanup. It must be
    /// invoked by the retained host before it releases the App runtime owner; `Drop` remains
    /// limited to non-blocking local cleanup.
    pub fn shutdown_runtime_session(&self) -> EditorRuntimeSessionShutdownReceipt {
        let event_consumers = self.shutdown_runtime_event_consumers();
        let play_world_sync = self.shutdown_play_world_sync();
        let edit_world_sync = self
            .edit_world_sync
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shutdown(self.context().gateway());
        let play_session = self.shutdown_play_session();
        let play_gateway = self.shutdown_play_gateway(&play_session);
        let play_backend_retirement = self.shutdown_play_backend_retirement(&play_gateway);

        EditorRuntimeSessionShutdownReceipt {
            event_consumers,
            edit_world_sync,
            play_world_sync,
            play_session,
            play_gateway,
            play_backend_retirement,
        }
    }

    pub(in crate::ui::host) fn shutdown_runtime_event_consumers(
        &self,
    ) -> RuntimeEventConsumerShutdownDisposition {
        let had_active_session = self.runtime_event_consumer_session_active();
        let had_pending_remote_cleanup =
            self.runtime_event_consumers.pending_remote_cleanup_count() > 0;
        match self.runtime_event_consumers.shutdown() {
            Ok(()) if !had_active_session && !had_pending_remote_cleanup => {
                RuntimeEventConsumerShutdownDisposition::NotActive
            }
            Ok(()) => RuntimeEventConsumerShutdownDisposition::Retired,
            Err(error)
                if !self.runtime_event_consumer_session_active()
                    && self.runtime_event_consumers.active_consumer_count() == 0 =>
            {
                RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure { error }
            }
            Err(error) => RuntimeEventConsumerShutdownDisposition::RetirementDeferred { error },
        }
    }

    fn shutdown_play_session(&self) -> RuntimePlaySessionShutdownDisposition {
        let mode = self.play_sessions.mode();
        if mode == PlayModeKind::Edit {
            return RuntimePlaySessionShutdownDisposition::NotPlaying;
        }

        match self.play_sessions.request_stop() {
            Ok(report) => match &report.cause {
                PlayTransitionCause::CleanupFailed { failure } => {
                    RuntimePlaySessionShutdownDisposition::StoppedWithCleanupFailure {
                        failure: failure.clone(),
                        report,
                    }
                }
                _ => RuntimePlaySessionShutdownDisposition::Stopped { report },
            },
            Err(error) => RuntimePlaySessionShutdownDisposition::RetirementDeferred { mode, error },
        }
    }

    fn shutdown_editor_play_state(
        &self,
        play_session: &RuntimePlaySessionShutdownDisposition,
    ) -> EditorPlayStateShutdownDisposition {
        if !play_session.is_locally_terminal() {
            return EditorPlayStateShutdownDisposition::NotPlaying;
        }
        let mut shell = self.shell().lock();
        if !shell.state.is_playing() {
            return EditorPlayStateShutdownDisposition::NotPlaying;
        }
        match shell.state.exit_play_mode() {
            Ok(_) => EditorPlayStateShutdownDisposition::Restored,
            Err(error) => EditorPlayStateShutdownDisposition::RestorationDeferred { error },
        }
    }

    fn shutdown_play_gateway(
        &self,
        play_session: &RuntimePlaySessionShutdownDisposition,
    ) -> RuntimePlayGatewayShutdownDisposition {
        if !play_session.is_locally_terminal() {
            return RuntimePlayGatewayShutdownDisposition::RetainedForActivePlay {
                mode: self.play_sessions.mode(),
            };
        }
        match self.detach_terminal_play_gateway() {
            Ok(Some((instance, identity))) => {
                RuntimePlayGatewayShutdownDisposition::Detached { instance, identity }
            }
            Ok(None) => RuntimePlayGatewayShutdownDisposition::NotAttached,
            Err(error) => {
                let Some(WorldDomain::Play(instance)) = self.play_sessions.attached_world_domain()
                else {
                    return RuntimePlayGatewayShutdownDisposition::NotAttached;
                };
                let identity = self
                    .play_sessions
                    .play_gateway(instance)
                    .map(|gateway| gateway.identity())
                    .unwrap_or_else(GatewaySessionIdentity::detached);
                RuntimePlayGatewayShutdownDisposition::RetirementDeferred {
                    instance,
                    identity,
                    error,
                }
            }
        }
    }

    fn shutdown_play_backend_retirement(
        &self,
        play_gateway: &RuntimePlayGatewayShutdownDisposition,
    ) -> RuntimePlayBackendRetirementDisposition {
        if !self.play_sessions.terminal_backend_retirement_pending() {
            return RuntimePlayBackendRetirementDisposition::NotPending;
        }
        match play_gateway {
            RuntimePlayGatewayShutdownDisposition::RetainedForActivePlay { mode } => {
                return RuntimePlayBackendRetirementDisposition::RetainedForActivePlay {
                    mode: *mode,
                };
            }
            RuntimePlayGatewayShutdownDisposition::RetirementDeferred { .. } => {
                return RuntimePlayBackendRetirementDisposition::RetainedForGatewayRetirement;
            }
            RuntimePlayGatewayShutdownDisposition::NotAttached
            | RuntimePlayGatewayShutdownDisposition::Detached { .. } => {}
        }

        match self.play_sessions.retire_terminal_backend() {
            Ok(report) => {
                let retirement_failure = match &report.cause {
                    PlayTransitionCause::CleanupFailed {
                        failure: failure @ PlayCleanupFailure::BackendRetirement { .. },
                    } => Some(failure.clone()),
                    _ => None,
                };
                match retirement_failure {
                    Some(failure) => RuntimePlayBackendRetirementDisposition::RetirementDeferred {
                        failure,
                        report,
                    },
                    None => RuntimePlayBackendRetirementDisposition::Retired { report },
                }
            }
            Err(error) => RuntimePlayBackendRetirementDisposition::RetirementRejected { error },
        }
    }
}
