use crate::core::diagnostics::DiagnosticStore;
use crate::core::CoreHandle;

pub const ECS_BUNDLE_TRANSACTION_COUNT_DIAGNOSTIC: &str = "scene.ecs.bundle_transactions.committed";
pub const ECS_BUNDLE_FINAL_ARCHETYPE_TRANSITIONS_DIAGNOSTIC: &str =
    "scene.ecs.bundle_transactions.final_archetype_transitions";
pub const ECS_BUNDLE_INTERMEDIATE_SIGNATURES_DIAGNOSTIC: &str =
    "scene.ecs.bundle_transactions.intermediate_signatures";
pub const ECS_BUNDLE_STORAGE_MOVES_DIAGNOSTIC: &str =
    "scene.ecs.bundle_transactions.component_storage_moves";
pub const ECS_BUNDLE_LIFECYCLE_EVENTS_DIAGNOSTIC: &str =
    "scene.ecs.bundle_transactions.lifecycle_events";
pub const ECS_BUNDLE_STAGING_ALLOCATIONS_DIAGNOSTIC: &str =
    "scene.ecs.bundle_transactions.staged_value_allocations";

/// Per-frame counters for staged Bundle publication. These counters describe
/// only the transaction path, so performance tests can detect an accidental
/// return to one-archetype-transition-per-component behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BundleTransactionDiagnostics {
    pub committed_transactions: u64,
    pub final_archetype_transitions: u64,
    pub intermediate_signatures: u64,
    pub component_storage_moves: u64,
    pub lifecycle_events: u64,
    pub staged_value_allocations: u64,
}

impl BundleTransactionDiagnostics {
    pub(crate) fn record_commit(
        &mut self,
        final_archetype_transition: bool,
        archetype_assignments: u64,
        component_storage_moves: usize,
        lifecycle_events: usize,
        staged_value_allocations: usize,
    ) {
        // `archetype_assignments` is the World assignment-counter delta for
        // this transaction. Anything after the one final location transition
        // is an observable intermediate signature regression.
        self.committed_transactions = self.committed_transactions.saturating_add(1);
        self.final_archetype_transitions = self
            .final_archetype_transitions
            .saturating_add(u64::from(final_archetype_transition));
        self.intermediate_signatures = self.intermediate_signatures.saturating_add(
            archetype_assignments.saturating_sub(u64::from(final_archetype_transition)),
        );
        self.component_storage_moves = self
            .component_storage_moves
            .saturating_add(component_storage_moves as u64);
        self.lifecycle_events = self
            .lifecycle_events
            .saturating_add(lifecycle_events as u64);
        self.staged_value_allocations = self
            .staged_value_allocations
            .saturating_add(staged_value_allocations as u64);
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        for (path, value) in self.diagnostic_values() {
            store.record(path, frame_index, value, Some("count"), ["ecs", "bundle"]);
        }
    }

    pub fn publish(&self, core: &CoreHandle, frame_index: u64) {
        for (path, value) in self.diagnostic_values() {
            core.record_diagnostic(path, frame_index, value, Some("count"), ["ecs", "bundle"]);
        }
    }

    fn diagnostic_values(&self) -> [(&'static str, f64); 6] {
        [
            (
                ECS_BUNDLE_TRANSACTION_COUNT_DIAGNOSTIC,
                self.committed_transactions as f64,
            ),
            (
                ECS_BUNDLE_FINAL_ARCHETYPE_TRANSITIONS_DIAGNOSTIC,
                self.final_archetype_transitions as f64,
            ),
            (
                ECS_BUNDLE_INTERMEDIATE_SIGNATURES_DIAGNOSTIC,
                self.intermediate_signatures as f64,
            ),
            (
                ECS_BUNDLE_STORAGE_MOVES_DIAGNOSTIC,
                self.component_storage_moves as f64,
            ),
            (
                ECS_BUNDLE_LIFECYCLE_EVENTS_DIAGNOSTIC,
                self.lifecycle_events as f64,
            ),
            (
                ECS_BUNDLE_STAGING_ALLOCATIONS_DIAGNOSTIC,
                self.staged_value_allocations as f64,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::BundleTransactionDiagnostics;

    #[test]
    fn transaction_diagnostics_derive_intermediate_signatures_from_actual_assignments() {
        let mut diagnostics = BundleTransactionDiagnostics::default();

        diagnostics.record_commit(true, 3, 0, 0, 0);

        assert_eq!(diagnostics.committed_transactions, 1);
        assert_eq!(diagnostics.final_archetype_transitions, 1);
        assert_eq!(diagnostics.intermediate_signatures, 2);
    }
}
