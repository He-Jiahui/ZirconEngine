use std::sync::Arc;

use crate::handles::ZrRuntimeSessionHandle;

/// Immutable owner-qualified identity for one editor-visible runtime transport.
///
/// App creates the base descriptor when it creates a runtime session. Consumers must retain the
/// complete value with any opaque runtime handle; a raw ABI handle has no meaning on its own.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GatewaySessionIdentity {
    runtime_instance: u64,
    runtime_session: ZrRuntimeSessionHandle,
    gateway_generation: u64,
    transport_epoch: u64,
    project: Option<Arc<str>>,
    play_instance: Option<u64>,
}

impl GatewaySessionIdentity {
    /// Constructs the App-owned base identity for one successfully created runtime session.
    pub fn new(
        runtime_instance: u64,
        runtime_session: ZrRuntimeSessionHandle,
        transport_epoch: u64,
        project: Option<Arc<str>>,
    ) -> Self {
        Self {
            runtime_instance,
            runtime_session,
            gateway_generation: 0,
            transport_epoch,
            project,
            play_instance: None,
        }
    }

    /// Describes an editor transport that is intentionally detached from every runtime session.
    pub const fn detached() -> Self {
        Self {
            runtime_instance: 0,
            runtime_session: ZrRuntimeSessionHandle::invalid(),
            gateway_generation: 0,
            transport_epoch: 0,
            project: None,
            play_instance: None,
        }
    }

    pub const fn runtime_instance(&self) -> u64 {
        self.runtime_instance
    }

    pub const fn runtime_session(&self) -> ZrRuntimeSessionHandle {
        self.runtime_session
    }

    pub const fn gateway_generation(&self) -> u64 {
        self.gateway_generation
    }

    pub const fn transport_epoch(&self) -> u64 {
        self.transport_epoch
    }

    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    pub const fn play_instance(&self) -> Option<u64> {
        self.play_instance
    }

    /// Adds the generation assigned when an Editor gateway publishes this transport.
    pub fn with_gateway_generation(mut self, gateway_generation: u64) -> Self {
        self.gateway_generation = gateway_generation;
        self
    }

    /// Adds the Editor Play-in-Editor instance that owns this published transport.
    pub fn with_play_instance(mut self, play_instance: Option<u64>) -> Self {
        self.play_instance = play_instance;
        self
    }
}
