use crate::core::framework::window::SurfaceLease;

use super::super::window_registry::WindowCloseBegin;

/// Platform-owned proof that a child-first window close order and every
/// affected surface lease entered their non-routable retirement states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlatformWindowCloseTransaction {
    windows: Vec<WindowCloseBegin>,
    retiring_leases: Vec<SurfaceLease>,
}

impl PlatformWindowCloseTransaction {
    pub(super) const fn new(
        windows: Vec<WindowCloseBegin>,
        retiring_leases: Vec<SurfaceLease>,
    ) -> Self {
        Self {
            windows,
            retiring_leases,
        }
    }

    pub(crate) fn windows(&self) -> &[WindowCloseBegin] {
        &self.windows
    }

    pub(crate) fn retiring_leases(&self) -> &[SurfaceLease] {
        &self.retiring_leases
    }
}
