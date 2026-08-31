use crate::core::recovery::{ProjectSessionEffectLedger, ProjectSessionEffectLedgerPhase};

use super::{ProjectCloseEffectReceipt, ProjectCloseOperation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectCloseReceipt {
    operation: ProjectCloseOperation,
    phase: ProjectSessionEffectLedgerPhase,
    effects: Vec<ProjectCloseEffectReceipt>,
}

impl ProjectCloseReceipt {
    pub(crate) fn from_ledger(
        operation: ProjectCloseOperation,
        ledger: &ProjectSessionEffectLedger,
    ) -> Self {
        let effects = ledger
            .effects()
            .iter()
            .map(|(effect, disposition)| ProjectCloseEffectReceipt {
                effect: *effect,
                disposition: *disposition,
            })
            .collect();
        Self {
            operation,
            phase: ledger.phase(),
            effects,
        }
    }

    pub(crate) const fn phase(&self) -> ProjectSessionEffectLedgerPhase {
        self.phase
    }

    pub(crate) fn operation(&self) -> &ProjectCloseOperation {
        &self.operation
    }

    pub(crate) fn effects(&self) -> &[ProjectCloseEffectReceipt] {
        &self.effects
    }
}
