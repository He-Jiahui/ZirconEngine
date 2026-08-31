use crate::core::framework::platform::ApplicationLifecycleOperation;
use crate::core::framework::window::SurfaceLease;

/// Driver-owned proof that the suspend operation entered `WillSuspend` only
/// after every currently active surface lease became non-routable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlatformApplicationSuspendTransaction {
    operation: ApplicationLifecycleOperation,
    retiring_leases: Vec<SurfaceLease>,
}

impl PlatformApplicationSuspendTransaction {
    pub(super) const fn new(
        operation: ApplicationLifecycleOperation,
        retiring_leases: Vec<SurfaceLease>,
    ) -> Self {
        Self {
            operation,
            retiring_leases,
        }
    }

    pub(crate) const fn operation(&self) -> ApplicationLifecycleOperation {
        self.operation
    }

    pub(crate) fn retiring_leases(&self) -> &[SurfaceLease] {
        &self.retiring_leases
    }
}
