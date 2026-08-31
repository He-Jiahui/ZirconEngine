use std::sync::{Arc, Mutex};

use thiserror::Error;

use zircon_runtime::scene::{LevelSystem, PreparedLevel, Scene};

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError, GatewaySessionIdentity,
    InProcessGateway, SharedEditorRuntimeGateway,
};

/// Core-owned construction input for an editor authoring world.
///
/// The runtime type never crosses into the workbench state. Once installed, every editor-side
/// borrowed-world access goes through the stable gateway handle carried by this facade.
pub(crate) enum AuthoringWorldSeed {
    Registered(LevelSystem),
    Prepared(PreparedLevel),
}

impl From<LevelSystem> for AuthoringWorldSeed {
    fn from(level: LevelSystem) -> Self {
        Self::Registered(level)
    }
}

impl From<PreparedLevel> for AuthoringWorldSeed {
    fn from(level: PreparedLevel) -> Self {
        Self::Prepared(level)
    }
}

/// UI-safe view of the edit domain's stable authoring gateway.
pub(crate) struct EditorAuthoringWorld {
    gateway: EditorRuntimeGatewayHandle,
    gateway_identity: GatewaySessionIdentity,
    loaded: bool,
    reported_access_failure: Mutex<Option<AuthoringWorldAccessError>>,
}

/// Failure projection for an authoring-world access request.
///
/// `Ok(None)` remains the only representation of an intentionally unloaded document. A gateway
/// failure is never converted into that state, so command, viewport, and inspection consumers
/// can preserve their distinct recovery policies.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum AuthoringWorldAccessError {
    #[error("the authoring runtime session was disconnected")]
    Disconnected {
        #[source]
        source: GatewayError,
    },
    #[error("the authoring gateway generation could not advance")]
    GenerationExhausted {
        #[source]
        source: GatewayError,
    },
    #[error(
        "the authoring scene belongs to gateway generation {expected_generation}, but generation {current_generation} is current"
    )]
    StaleGeneration {
        expected_generation: u64,
        current_generation: u64,
    },
    #[error("authoring world access requires a serialized gateway operation")]
    SerializedAccessRequired {
        #[source]
        source: GatewayError,
    },
    #[error("authoring world access cannot re-enter a borrowed-world callback")]
    ReentrantBorrowedWorldAccess {
        #[source]
        source: GatewayError,
    },
    #[error("the authoring gateway capability is unavailable")]
    CapabilityUnavailable {
        #[source]
        source: GatewayError,
    },
    #[error("the authoring runtime faulted")]
    RuntimeFault {
        #[source]
        source: GatewayError,
    },
    #[error("the authoring gateway violated its borrowed-world protocol")]
    ProtocolViolation {
        #[source]
        source: GatewayError,
    },
}

impl From<GatewayError> for AuthoringWorldAccessError {
    fn from(source: GatewayError) -> Self {
        match source {
            GatewayError::GenerationExhausted => Self::GenerationExhausted { source },
            GatewayError::StaleGeneration {
                expected_generation,
                current_generation,
            } => Self::StaleGeneration {
                expected_generation,
                current_generation,
            },
            GatewayError::SessionLost => Self::Disconnected { source },
            GatewayError::RequiresSerializedAccess => Self::SerializedAccessRequired { source },
            GatewayError::ReentrantBorrowedWorldAccess => {
                Self::ReentrantBorrowedWorldAccess { source }
            }
            GatewayError::CapabilityMissing { .. } => Self::CapabilityUnavailable { source },
            GatewayError::Runtime { .. } => Self::RuntimeFault { source },
            GatewayError::Protocol { .. } => Self::ProtocolViolation { source },
        }
    }
}

/// Result of a mutable borrowed-world callback.
///
/// The callback can complete before a gateway discovers a transport or protocol failure. Callers
/// must consume the post-callback error and compensate any editor-side preview state before they
/// report the operation as failed.
pub(crate) struct AuthoringWorldMutationOutcome<R> {
    result: R,
    post_callback_error: Option<AuthoringWorldAccessError>,
}

impl<R> AuthoringWorldMutationOutcome<R> {
    pub(crate) fn into_parts(self) -> (R, Option<AuthoringWorldAccessError>) {
        (self.result, self.post_callback_error)
    }
}

impl EditorAuthoringWorld {
    pub(crate) fn loaded(
        gateway: &EditorRuntimeGatewayHandle,
        seed: impl Into<AuthoringWorldSeed>,
    ) -> Result<Self, GatewayError> {
        let mut facade = Self {
            gateway: gateway.clone(),
            gateway_identity: gateway.identity(),
            loaded: false,
            reported_access_failure: Mutex::new(None),
        };
        facade.replace(seed)?;
        Ok(facade)
    }

    pub(crate) fn unloaded(gateway: &EditorRuntimeGatewayHandle) -> Result<Self, GatewayError> {
        let mut facade = Self {
            gateway: gateway.clone(),
            gateway_identity: gateway.identity(),
            loaded: false,
            reported_access_failure: Mutex::new(None),
        };
        facade.clear()?;
        Ok(facade)
    }

    pub(crate) fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub(crate) fn snapshot(&self) -> Result<Option<Scene>, AuthoringWorldAccessError> {
        self.with_world(Clone::clone)
    }

    pub(crate) fn with_world<R>(
        &self,
        read: impl FnOnce(&Scene) -> R,
    ) -> Result<Option<R>, AuthoringWorldAccessError> {
        if !self.loaded {
            self.clear_reported_access_failure();
            return Ok(None);
        }
        let mut read = Some(read);
        let mut result = None;
        let mut duplicate_callback = false;
        let dispatch = self
            .gateway
            .with_world_at_identity(&self.gateway_identity, &mut |scene| {
                let Some(read) = read.take() else {
                    duplicate_callback = true;
                    return;
                };
                result = Some(read(scene));
            });
        if duplicate_callback {
            return Err(gateway_callback_protocol_failure().into());
        }
        dispatch.map_err(AuthoringWorldAccessError::from)?;
        let result = result.ok_or_else(|| {
            AuthoringWorldAccessError::from(GatewayError::Protocol {
                message: "borrowed world callback was not invoked".to_owned(),
            })
        })?;
        self.clear_reported_access_failure();
        Ok(Some(result))
    }

    pub(crate) fn with_world_mut<R>(
        &self,
        write: impl FnOnce(&mut Scene) -> R,
    ) -> Result<Option<AuthoringWorldMutationOutcome<R>>, AuthoringWorldAccessError> {
        if !self.loaded {
            self.clear_reported_access_failure();
            return Ok(None);
        }
        let mut write = Some(write);
        let mut result = None;
        let mut duplicate_callback = false;
        let dispatch =
            self.gateway
                .with_world_mut_at_identity(&self.gateway_identity, &mut |scene| {
                    let Some(write) = write.take() else {
                        duplicate_callback = true;
                        return;
                    };
                    result = Some(write(scene));
                });
        let Some(result) = result else {
            return Err(match dispatch {
                Ok(()) => AuthoringWorldAccessError::from(GatewayError::Protocol {
                    message: "borrowed world callback was not invoked".to_owned(),
                }),
                Err(error) => error.into(),
            });
        };
        let post_callback_error = if duplicate_callback {
            Some(gateway_callback_protocol_failure().into())
        } else {
            dispatch.err().map(AuthoringWorldAccessError::from)
        };
        if post_callback_error.is_none() {
            self.clear_reported_access_failure();
        }
        Ok(Some(AuthoringWorldMutationOutcome {
            result,
            post_callback_error,
        }))
    }

    /// Returns true once for each distinct fault observed since the last successful access.
    /// Projection paths use this to avoid emitting one diagnostics record per render or UI frame.
    pub(crate) fn should_report_access_failure(&self, error: &AuthoringWorldAccessError) -> bool {
        let mut reported = self
            .reported_access_failure
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if reported.as_ref() == Some(error) {
            return false;
        }
        *reported = Some(error.clone());
        true
    }

    #[cfg(test)]
    pub(crate) fn expect_with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> R {
        self.with_world(read)
            .expect("editor world gateway must succeed")
            .expect("editor world is not loaded")
    }

    #[cfg(test)]
    pub(crate) fn replace_gateway_for_test(
        &mut self,
        gateway: SharedEditorRuntimeGateway,
    ) -> Result<(), GatewayError> {
        self.gateway.replace(gateway)?;
        self.gateway_identity = self.gateway.identity();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn expect_with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> R {
        let outcome = self
            .with_world_mut(write)
            .expect("editor world gateway must invoke the callback")
            .expect("editor world is not loaded");
        let (result, post_callback_error) = outcome.into_parts();
        assert!(
            post_callback_error.is_none(),
            "editor world gateway must not fail after callback: {post_callback_error:?}"
        );
        result
    }

    pub(crate) fn replace(
        &mut self,
        seed: impl Into<AuthoringWorldSeed>,
    ) -> Result<(), GatewayError> {
        match seed.into() {
            AuthoringWorldSeed::Registered(level) => {
                let gateway: SharedEditorRuntimeGateway =
                    Arc::new(InProcessGateway::for_authoring_level(level));
                self.gateway.replace(gateway)?;
            }
            AuthoringWorldSeed::Prepared(level) => {
                let publication = level.publish();
                let gateway: SharedEditorRuntimeGateway = Arc::new(
                    InProcessGateway::for_authoring_level(publication.level().clone()),
                );
                self.gateway.replace(gateway)?;
                publication.commit();
            }
        }
        self.gateway_identity = self.gateway.identity();
        self.loaded = true;
        self.clear_reported_access_failure();
        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Result<(), GatewayError> {
        self.gateway
            .replace(Arc::new(DetachedEditorRuntimeGateway))?;
        self.gateway_identity = self.gateway.identity();
        self.loaded = false;
        self.clear_reported_access_failure();
        Ok(())
    }

    fn clear_reported_access_failure(&self) {
        *self
            .reported_access_failure
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }
}

fn gateway_callback_protocol_failure() -> GatewayError {
    GatewayError::Protocol {
        message: "borrowed world callback was invoked more than once".to_owned(),
    }
}
