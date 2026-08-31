use zircon_runtime_interface::ui::binding::{UiBindingDirtyDomain, UiBindingMutationReceipt};

use super::{
    UiSurface,
    mutation_snapshot::{UiSurfaceMutationDomains, UiSurfaceMutationSnapshot},
};

pub(crate) struct UiBindingMutationTransaction {
    base_generation: u64,
    target_count: usize,
    snapshot: UiSurfaceMutationSnapshot,
}

impl UiBindingMutationTransaction {
    pub(crate) fn prepare(surface: &UiSurface, target_count: usize) -> Self {
        Self {
            base_generation: surface.invalidation_generations().generation,
            target_count,
            snapshot: UiSurfaceMutationSnapshot::capture(
                surface,
                UiSurfaceMutationDomains::binding_targets(),
            ),
        }
    }

    pub(crate) fn commit(
        self,
        applied_target_count: usize,
        unchanged_target_count: usize,
        impact: Vec<UiBindingDirtyDomain>,
        advances_surface_revision: bool,
    ) -> UiBindingMutationReceipt {
        let revision = self
            .base_generation
            .saturating_add(if advances_surface_revision { 1 } else { 0 });
        UiBindingMutationReceipt::committed(
            self.base_generation,
            revision,
            self.target_count,
            applied_target_count,
            unchanged_target_count,
            impact,
        )
    }

    pub(crate) fn rollback(self, surface: &mut UiSurface) -> UiBindingMutationReceipt {
        let receipt =
            UiBindingMutationReceipt::rolled_back(self.base_generation, self.target_count);
        self.snapshot.restore(surface);
        receipt
    }
}
