use std::time::Instant;

use super::WindowCommandId;
use crate::core::framework::window::WindowId;

/// Required authority and deadline fields for every platform-thread window
/// side effect. A desired payload cannot be routed without this header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowCommandHeader {
    target: WindowId,
    request_id: WindowCommandId,
    deadline: Instant,
}

impl WindowCommandHeader {
    pub(crate) const fn new(
        target: WindowId,
        request_id: WindowCommandId,
        deadline: Instant,
    ) -> Self {
        Self {
            target,
            request_id,
            deadline,
        }
    }

    pub const fn target(self) -> WindowId {
        self.target
    }

    pub const fn request_id(self) -> WindowCommandId {
        self.request_id
    }

    pub const fn deadline(self) -> Instant {
        self.deadline
    }
}

/// A generation-qualified desired window change. The concrete desired-state
/// schema stays separate from this transport contract so legacy descriptor
/// fields cannot silently re-enter the runtime command path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowCommand<Desired> {
    header: WindowCommandHeader,
    desired: Desired,
}

impl<Desired> WindowCommand<Desired> {
    pub(crate) const fn new(header: WindowCommandHeader, desired: Desired) -> Self {
        Self { header, desired }
    }

    pub const fn header(&self) -> WindowCommandHeader {
        self.header
    }

    pub const fn target(&self) -> WindowId {
        self.header.target()
    }

    pub const fn request_id(&self) -> WindowCommandId {
        self.header.request_id()
    }

    pub const fn deadline(&self) -> Instant {
        self.header.deadline()
    }

    pub const fn desired(&self) -> &Desired {
        &self.desired
    }

    pub fn into_desired(self) -> Desired {
        self.desired
    }
}
