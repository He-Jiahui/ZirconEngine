use super::SurfaceLease;

/// A fully validated set of active leases that may move to retirement without
/// further allocation or admission checks while the registry lock is held.
pub(crate) struct SurfaceLeaseRetirementPlan {
    retiring_leases: Vec<SurfaceLease>,
    registry_leases: Vec<SurfaceLease>,
}

impl SurfaceLeaseRetirementPlan {
    pub(super) const fn new(
        retiring_leases: Vec<SurfaceLease>,
        registry_leases: Vec<SurfaceLease>,
    ) -> Self {
        Self {
            retiring_leases,
            registry_leases,
        }
    }

    pub(crate) fn leases(&self) -> &[SurfaceLease] {
        &self.retiring_leases
    }

    pub(super) fn into_parts(self) -> (Vec<SurfaceLease>, Vec<SurfaceLease>) {
        (self.retiring_leases, self.registry_leases)
    }
}
