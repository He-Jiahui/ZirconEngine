use super::{
    ProjectSessionEffect, ProjectSessionEffectDisposition, ProjectSessionEffectLedgerPhase,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProjectSessionEffectRecoveryEntry {
    effect: ProjectSessionEffect,
    disposition: ProjectSessionEffectDisposition,
}

impl ProjectSessionEffectRecoveryEntry {
    pub(crate) const fn new(
        effect: ProjectSessionEffect,
        disposition: ProjectSessionEffectDisposition,
    ) -> Self {
        Self {
            effect,
            disposition,
        }
    }

    pub(crate) const fn effect(self) -> ProjectSessionEffect {
        self.effect
    }

    pub(crate) const fn disposition(self) -> ProjectSessionEffectDisposition {
        self.disposition
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectSessionRecoveryStatus {
    Missing,
    Terminal,
    Incomplete {
        phase: ProjectSessionEffectLedgerPhase,
        effects: Vec<ProjectSessionEffectRecoveryEntry>,
    },
    RecoveryRequired {
        phase: ProjectSessionEffectLedgerPhase,
        effects: Vec<ProjectSessionEffectRecoveryEntry>,
    },
}
