use crate::core::framework::window::{NativeWindowId, WindowId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

/// A versioned transition of the registry-owned primary window role.
///
/// This is deliberately separate from `WindowId::generation()`: replacing a
/// primary role does not recreate either window, while a destroyed primary is
/// invalidated before the native generation is torn down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrimaryWindowRoleChange {
    previous: Option<WindowId>,
    current: Option<WindowId>,
    generation: u64,
}

impl PrimaryWindowRoleChange {
    pub(super) const fn new(
        previous: Option<WindowId>,
        current: Option<WindowId>,
        generation: u64,
    ) -> Self {
        Self {
            previous,
            current,
            generation,
        }
    }

    pub(crate) const fn previous(self) -> Option<WindowId> {
        self.previous
    }

    pub(crate) const fn current(self) -> Option<WindowId> {
        self.current
    }

    pub(crate) const fn generation(self) -> u64 {
        self.generation
    }
}

/// The first phase of a window destroy transaction.
///
/// The host must retire generation-qualified external resources before it
/// calls `WindowRegistry::finish_destroy`. A primary role change is returned
/// synchronously so later command-broker work can publish one ordered event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WindowCloseBegin {
    window: WindowId,
    native_window: NativeWindowId,
    primary_role_change: Option<PrimaryWindowRoleChange>,
    viewports: Vec<ZrRuntimeViewportHandle>,
}

impl WindowCloseBegin {
    pub(super) const fn new(
        window: WindowId,
        native_window: NativeWindowId,
        primary_role_change: Option<PrimaryWindowRoleChange>,
        viewports: Vec<ZrRuntimeViewportHandle>,
    ) -> Self {
        Self {
            window,
            native_window,
            primary_role_change,
            viewports,
        }
    }

    pub(crate) const fn window(&self) -> WindowId {
        self.window
    }

    pub(crate) const fn native_window(&self) -> NativeWindowId {
        self.native_window
    }

    pub(crate) const fn primary_role_change(&self) -> Option<PrimaryWindowRoleChange> {
        self.primary_role_change
    }

    /// Viewports detached before native teardown, in their binding order.
    pub(crate) fn viewports(&self) -> &[ZrRuntimeViewportHandle] {
        &self.viewports
    }
}
