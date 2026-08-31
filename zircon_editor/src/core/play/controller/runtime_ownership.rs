use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewaySessionIdentity};

use super::{
    PlayCleanupFailure, PlayDomainLinkError, PlayInstanceId, PlayMode, PlayModeKind,
    PlaySessionController, PlaySessionError, PlayTerminalGatewayDetachError, PlayTransitionCause,
    PlayTransitionReport, PluginBridgeActivationReport, WorldDomain,
};

struct TerminalGatewayDetachReservation<'a> {
    reserved: &'a AtomicBool,
}

impl<'a> TerminalGatewayDetachReservation<'a> {
    fn acquire(reserved: &'a AtomicBool) -> Option<Self> {
        reserved
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then_some(Self { reserved })
    }
}

impl Drop for TerminalGatewayDetachReservation<'_> {
    fn drop(&mut self) {
        self.reserved.store(false, Ordering::Release);
    }
}

impl PlaySessionController {
    pub(crate) fn detach_terminal_play_gateway<E>(
        &self,
        prepare: impl FnOnce(PlayInstanceId) -> Result<(), E>,
    ) -> Result<Option<(PlayInstanceId, GatewaySessionIdentity)>, PlayTerminalGatewayDetachError<E>>
    {
        let transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(_reservation) =
            TerminalGatewayDetachReservation::acquire(&self.terminal_gateway_detach)
        else {
            return Err(PlayTerminalGatewayDetachError::Domain(
                PlayDomainLinkError::TerminalDetachInProgress,
            ));
        };
        let mode = self.mode();
        if mode.has_active_runtime() {
            return Err(PlayTerminalGatewayDetachError::Domain(
                PlayDomainLinkError::RuntimeStillActive { mode },
            ));
        }
        let Some(WorldDomain::Play(instance)) = self.play_domain.attached_domain() else {
            return Ok(None);
        };
        let Some(gateway) = self.play_domain.gateway(instance) else {
            return Ok(None);
        };
        let identity = gateway.identity();
        drop(transition);
        prepare(instance).map_err(PlayTerminalGatewayDetachError::Preparation)?;
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.play_domain
            .detach_matching_identity(instance, &identity)
            .map_err(PlayTerminalGatewayDetachError::Domain)?;
        Ok(Some((instance, identity)))
    }

    pub fn attached_world_domain(&self) -> Option<WorldDomain> {
        self.play_domain.attached_domain()
    }

    pub fn terminal_backend_retirement_pending(&self) -> bool {
        matches!(
            self.session_ownership
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref(),
            Some(super::PlaySessionOwnership::Terminal {
                backend: Some(_),
                ..
            })
        )
    }

    pub fn play_gateway(&self, instance: PlayInstanceId) -> Option<EditorRuntimeGatewayHandle> {
        self.play_domain.gateway(instance)
    }

    /// Releases terminal backend ownership only after runtime consumers and the play gateway
    /// have retired. A failed App lease remains owned by the backend and is retried here.
    pub fn retire_terminal_backend(&self) -> Result<PlayTransitionReport, PlaySessionError> {
        let transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let from = self.mode();
        if from.has_active_runtime() {
            drop(transition);
            return Err(PlaySessionError::InvalidTransition {
                mode: from,
                event: "retire_terminal_backend",
            });
        }
        if self.terminal_gateway_detach.load(Ordering::Acquire) {
            drop(transition);
            return Err(PlaySessionError::InvalidTransition {
                mode: from,
                event: "retire_terminal_backend_during_gateway_detach",
            });
        }
        if self.play_domain.attached_domain().is_some() {
            drop(transition);
            return Err(PlaySessionError::InvalidTransition {
                mode: from,
                event: "retire_terminal_backend_with_attached_gateway",
            });
        }
        let pending = {
            let ownership = self
                .session_ownership
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match ownership.as_ref() {
                Some(super::PlaySessionOwnership::Terminal {
                    kind,
                    backend: Some(backend),
                    ..
                }) => Some((*kind, std::sync::Arc::clone(backend))),
                _ => None,
            }
        };
        let Some((kind, backend)) = pending else {
            drop(transition);
            return Ok(PlayTransitionReport::unchanged(from));
        };

        let result = match backend.retire() {
            Ok(report) => {
                let mut ownership = self
                    .session_ownership
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let clear_ownership = match ownership.as_mut() {
                    Some(super::PlaySessionOwnership::Terminal {
                        kind: owned_kind,
                        backend,
                        activation,
                    }) if *owned_kind == kind => {
                        *backend = None;
                        activation.is_none()
                    }
                    _ => false,
                };
                if clear_ownership {
                    *ownership = None;
                }
                drop(ownership);
                match self.mode_snapshot() {
                    PlayMode::CleanupFailed {
                        failure:
                            PlayCleanupFailure::BackendRetirement {
                                plugin_deactivation: Some(message),
                                ..
                            },
                        ..
                    } => {
                        let failure = PlayCleanupFailure::PluginDeactivation { message };
                        self.replace_mode(PlayMode::CleanupFailed {
                            kind,
                            failure: failure.clone(),
                        });
                        PlayTransitionReport::cleanup_failed(false, report.diagnostics, failure)
                    }
                    PlayMode::CleanupFailed {
                        failure: PlayCleanupFailure::BackendRetirement { .. },
                        ..
                    } => {
                        self.replace_mode(PlayMode::Edit);
                        PlayTransitionReport::changed(
                            PlayModeKind::Edit,
                            PluginBridgeActivationReport::default(),
                            report.diagnostics,
                            false,
                            PlayTransitionCause::Stopped,
                        )
                    }
                    _ => PlayTransitionReport::unchanged_with_backend(from, report.diagnostics),
                }
            }
            Err(message) => {
                let plugin_deactivation = match self.mode_snapshot() {
                    PlayMode::CleanupFailed {
                        failure: PlayCleanupFailure::PluginDeactivation { message },
                        ..
                    } => Some(message),
                    PlayMode::CleanupFailed {
                        failure:
                            PlayCleanupFailure::BackendRetirement {
                                plugin_deactivation,
                                ..
                            },
                        ..
                    } => plugin_deactivation,
                    _ => None,
                };
                let failure = PlayCleanupFailure::BackendRetirement {
                    message,
                    plugin_deactivation,
                };
                self.replace_mode(PlayMode::CleanupFailed {
                    kind,
                    failure: failure.clone(),
                });
                PlayTransitionReport::cleanup_failed(
                    from != PlayModeKind::CleanupFailed,
                    Vec::new(),
                    failure,
                )
            }
        };
        let to = result.mode;
        drop(transition);
        self.publish_mode_transition(from, to);
        Ok(result)
    }
}
