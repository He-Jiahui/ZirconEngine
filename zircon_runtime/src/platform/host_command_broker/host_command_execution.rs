use crate::core::framework::window::{
    WindowCommand, WindowCommandHeader, WindowCommandId, WindowId, WindowRequestedGeneration,
    WindowRequestedState,
};

/// One dispatched command paired with the requested-state publication it is
/// allowed to make effective. The platform host returns this identity with its
/// native completion instead of inferring recency from queue order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCommandExecution {
    command: WindowCommand<WindowRequestedState>,
    requested_generation: WindowRequestedGeneration,
}

impl HostCommandExecution {
    pub(super) const fn new(
        command: WindowCommand<WindowRequestedState>,
        requested_generation: WindowRequestedGeneration,
    ) -> Self {
        Self {
            command,
            requested_generation,
        }
    }

    pub const fn command(&self) -> &WindowCommand<WindowRequestedState> {
        &self.command
    }

    pub const fn header(&self) -> WindowCommandHeader {
        self.command.header()
    }

    pub const fn target(&self) -> WindowId {
        self.command.target()
    }

    pub const fn request_id(&self) -> WindowCommandId {
        self.command.request_id()
    }

    pub const fn requested_generation(&self) -> WindowRequestedGeneration {
        self.requested_generation
    }
}
