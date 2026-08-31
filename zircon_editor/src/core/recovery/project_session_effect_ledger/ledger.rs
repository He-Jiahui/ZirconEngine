use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::ProjectActivationOperationId;

use super::{
    ProjectSessionEffect, ProjectSessionEffectDisposition, ProjectSessionEffectLedgerError,
    ProjectSessionEffectLedgerPhase, ProjectSessionEffectMutation,
    ProjectSessionEffectRecoveryEntry, ProjectSessionRecoveryStatus,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectSessionEffectLedger {
    schema_version: u32,
    operation_id: ProjectActivationOperationId,
    phase: ProjectSessionEffectLedgerPhase,
    effects: BTreeMap<ProjectSessionEffect, ProjectSessionEffectDisposition>,
}

impl ProjectSessionEffectLedger {
    pub(super) const SCHEMA_VERSION: u32 = 1;

    pub(crate) fn for_operation(operation_id: ProjectActivationOperationId) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            operation_id,
            phase: ProjectSessionEffectLedgerPhase::Activating,
            effects: BTreeMap::new(),
        }
    }

    pub(crate) const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    pub(crate) const fn phase(&self) -> ProjectSessionEffectLedgerPhase {
        self.phase
    }

    pub(crate) fn effects(
        &self,
    ) -> &BTreeMap<ProjectSessionEffect, ProjectSessionEffectDisposition> {
        &self.effects
    }

    pub(crate) fn disposition(
        &self,
        effect: ProjectSessionEffect,
    ) -> Option<ProjectSessionEffectDisposition> {
        self.effects.get(&effect).copied()
    }

    pub(crate) fn prepare(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.apply(effect, ProjectSessionEffectMutation::Prepare)
    }

    pub(crate) fn commit(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.apply(effect, ProjectSessionEffectMutation::Commit)
    }

    pub(crate) fn roll_back(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.apply(effect, ProjectSessionEffectMutation::RollBack)
    }

    pub(crate) fn mark_recovery_required(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.phase == ProjectSessionEffectLedgerPhase::Closed
            && self.disposition(effect) == Some(ProjectSessionEffectDisposition::Committed)
        {
            self.effects
                .insert(effect, ProjectSessionEffectDisposition::RecoveryRequired);
            self.phase = ProjectSessionEffectLedgerPhase::RecoveryRequired;
            return Ok(());
        }
        self.apply(effect, ProjectSessionEffectMutation::RequireRecovery)?;
        self.phase = ProjectSessionEffectLedgerPhase::RecoveryRequired;
        Ok(())
    }

    pub(crate) fn require_recovery_for_active_effects(
        &mut self,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        for disposition in self.effects.values_mut() {
            if matches!(
                *disposition,
                ProjectSessionEffectDisposition::Prepared
                    | ProjectSessionEffectDisposition::Committed
            ) {
                *disposition = ProjectSessionEffectDisposition::RecoveryRequired;
            }
        }
        self.phase = ProjectSessionEffectLedgerPhase::RecoveryRequired;
        Ok(())
    }

    pub(crate) fn roll_back_active_effects(
        &mut self,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        for disposition in self.effects.values_mut() {
            if matches!(
                *disposition,
                ProjectSessionEffectDisposition::Prepared
                    | ProjectSessionEffectDisposition::Committed
            ) {
                *disposition = ProjectSessionEffectDisposition::RolledBack;
            }
        }
        Ok(())
    }

    pub(crate) fn begin_ready(&mut self) -> Result<(), ProjectSessionEffectLedgerError> {
        self.require_phase(ProjectSessionEffectLedgerPhase::Activating)?;
        self.require_committed(&ProjectSessionEffect::ACTIVATION_EFFECTS)?;
        self.phase = ProjectSessionEffectLedgerPhase::Ready;
        Ok(())
    }

    pub(crate) fn finish_aborted_activation(
        &mut self,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.require_phase(ProjectSessionEffectLedgerPhase::Activating)?;
        let unsettled = self
            .effects
            .iter()
            .filter_map(|(effect, disposition)| {
                matches!(
                    disposition,
                    ProjectSessionEffectDisposition::Prepared
                        | ProjectSessionEffectDisposition::Committed
                        | ProjectSessionEffectDisposition::RecoveryRequired
                )
                .then_some(*effect)
            })
            .collect::<Vec<_>>();
        if !unsettled.is_empty() {
            return Err(ProjectSessionEffectLedgerError::UnsettledEffects {
                phase: self.phase,
                effects: unsettled,
            });
        }
        self.phase = ProjectSessionEffectLedgerPhase::Closed;
        Ok(())
    }

    pub(crate) fn begin_closing(&mut self) -> Result<(), ProjectSessionEffectLedgerError> {
        self.require_phase(ProjectSessionEffectLedgerPhase::Ready)?;
        self.effects.clear();
        self.effects.extend(
            ProjectSessionEffect::CLOSE_EFFECTS
                .into_iter()
                .map(|effect| (effect, ProjectSessionEffectDisposition::Prepared)),
        );
        self.phase = ProjectSessionEffectLedgerPhase::Closing;
        Ok(())
    }

    pub(crate) fn finish_closed(&mut self) -> Result<(), ProjectSessionEffectLedgerError> {
        self.require_phase(ProjectSessionEffectLedgerPhase::Closing)?;
        self.require_committed(&ProjectSessionEffect::CLOSE_EFFECTS)?;
        self.phase = ProjectSessionEffectLedgerPhase::Closed;
        Ok(())
    }

    pub(super) const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub(super) fn recovery_status(&self) -> ProjectSessionRecoveryStatus {
        match self.phase {
            ProjectSessionEffectLedgerPhase::Closed => ProjectSessionRecoveryStatus::Terminal,
            ProjectSessionEffectLedgerPhase::RecoveryRequired => {
                ProjectSessionRecoveryStatus::RecoveryRequired {
                    phase: self.phase,
                    effects: self
                        .effects
                        .iter()
                        .map(|(effect, disposition)| {
                            ProjectSessionEffectRecoveryEntry::new(*effect, *disposition)
                        })
                        .collect(),
                }
            }
            phase => ProjectSessionRecoveryStatus::Incomplete {
                phase,
                effects: self
                    .effects
                    .iter()
                    .map(|(effect, disposition)| {
                        ProjectSessionEffectRecoveryEntry::new(*effect, *disposition)
                    })
                    .collect(),
            },
        }
    }

    pub(super) fn validate_persisted_state(&self) -> Result<(), String> {
        let valid = match self.phase {
            ProjectSessionEffectLedgerPhase::Activating => {
                self.has_only_effects(&ProjectSessionEffect::ACTIVATION_EFFECTS)
                    && self.has_only_dispositions(&[
                        ProjectSessionEffectDisposition::Prepared,
                        ProjectSessionEffectDisposition::Committed,
                        ProjectSessionEffectDisposition::RolledBack,
                    ])
            }
            ProjectSessionEffectLedgerPhase::Ready => {
                self.has_all_committed(&ProjectSessionEffect::ACTIVATION_EFFECTS)
                    && self.has_only_effects(
                        &[
                            &ProjectSessionEffect::ACTIVATION_EFFECTS[..],
                            &ProjectSessionEffect::READY_EFFECTS[..],
                        ]
                        .concat(),
                    )
                    && self.has_only_dispositions(&[
                        ProjectSessionEffectDisposition::Prepared,
                        ProjectSessionEffectDisposition::Committed,
                        ProjectSessionEffectDisposition::RolledBack,
                    ])
            }
            ProjectSessionEffectLedgerPhase::Closing => {
                self.has_exact_effects(&ProjectSessionEffect::CLOSE_EFFECTS)
                    && self.has_only_dispositions(&[
                        ProjectSessionEffectDisposition::Prepared,
                        ProjectSessionEffectDisposition::Committed,
                    ])
            }
            ProjectSessionEffectLedgerPhase::Closed => {
                (self.has_exact_effects(&ProjectSessionEffect::CLOSE_EFFECTS)
                    && self.has_all_committed(&ProjectSessionEffect::CLOSE_EFFECTS))
                    || (self.has_only_effects(&ProjectSessionEffect::ACTIVATION_EFFECTS)
                        && self
                            .has_only_dispositions(&[ProjectSessionEffectDisposition::RolledBack]))
            }
            ProjectSessionEffectLedgerPhase::RecoveryRequired => {
                let has_recovery_owner = self.effects.values().any(|disposition| {
                    *disposition == ProjectSessionEffectDisposition::RecoveryRequired
                });
                let activation_and_ready = [
                    &ProjectSessionEffect::ACTIVATION_EFFECTS[..],
                    &ProjectSessionEffect::READY_EFFECTS[..],
                ]
                .concat();
                has_recovery_owner
                    && ((self.has_exact_effects(&ProjectSessionEffect::CLOSE_EFFECTS)
                        && self.has_only_dispositions(&[
                            ProjectSessionEffectDisposition::Prepared,
                            ProjectSessionEffectDisposition::Committed,
                            ProjectSessionEffectDisposition::RecoveryRequired,
                        ]))
                        || (self.has_only_effects(&ProjectSessionEffect::ACTIVATION_EFFECTS)
                            && self.has_only_dispositions(&[
                                ProjectSessionEffectDisposition::Prepared,
                                ProjectSessionEffectDisposition::Committed,
                                ProjectSessionEffectDisposition::RolledBack,
                                ProjectSessionEffectDisposition::RecoveryRequired,
                            ]))
                        || (self.has_all_effects(&ProjectSessionEffect::ACTIVATION_EFFECTS)
                            && self.has_only_effects(&activation_and_ready)
                            && self.effects_have_only_dispositions(
                                &ProjectSessionEffect::ACTIVATION_EFFECTS,
                                &[
                                    ProjectSessionEffectDisposition::Committed,
                                    ProjectSessionEffectDisposition::RecoveryRequired,
                                ],
                            )
                            && self.effects_have_only_dispositions(
                                &ProjectSessionEffect::READY_EFFECTS,
                                &[
                                    ProjectSessionEffectDisposition::Prepared,
                                    ProjectSessionEffectDisposition::Committed,
                                    ProjectSessionEffectDisposition::RolledBack,
                                    ProjectSessionEffectDisposition::RecoveryRequired,
                                ],
                            )))
            }
        };
        if valid {
            return Ok(());
        }
        Err(format!(
            "phase {:?} has an unreachable effect inventory {:?}",
            self.phase, self.effects
        ))
    }

    fn apply(
        &mut self,
        effect: ProjectSessionEffect,
        mutation: ProjectSessionEffectMutation,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if !effect.allowed_in(self.phase) {
            return Err(ProjectSessionEffectLedgerError::EffectNotAllowed {
                phase: self.phase,
                effect,
            });
        }
        let current = self.disposition(effect);
        if self.phase == ProjectSessionEffectLedgerPhase::Closing
            && mutation == ProjectSessionEffectMutation::RollBack
        {
            return Err(ProjectSessionEffectLedgerError::InvalidEffectTransition {
                effect,
                current,
                requested: mutation.target(),
            });
        }
        if !mutation.permits(current) {
            return Err(ProjectSessionEffectLedgerError::InvalidEffectTransition {
                effect,
                current,
                requested: mutation.target(),
            });
        }
        self.effects.insert(effect, mutation.target());
        Ok(())
    }

    fn require_phase(
        &self,
        requested_from: ProjectSessionEffectLedgerPhase,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.phase == requested_from {
            return Ok(());
        }
        Err(ProjectSessionEffectLedgerError::InvalidPhaseTransition {
            current: self.phase,
            requested: match requested_from {
                ProjectSessionEffectLedgerPhase::Activating => {
                    ProjectSessionEffectLedgerPhase::Ready
                }
                ProjectSessionEffectLedgerPhase::Ready => ProjectSessionEffectLedgerPhase::Closing,
                ProjectSessionEffectLedgerPhase::Closing => ProjectSessionEffectLedgerPhase::Closed,
                phase => phase,
            },
        })
    }

    fn require_committed(
        &self,
        required: &[ProjectSessionEffect],
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        let missing = required
            .iter()
            .copied()
            .filter(|effect| {
                self.disposition(*effect) != Some(ProjectSessionEffectDisposition::Committed)
            })
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return Ok(());
        }
        Err(ProjectSessionEffectLedgerError::MissingCommittedEffects {
            phase: self.phase,
            effects: missing,
        })
    }

    fn has_only_effects(&self, allowed: &[ProjectSessionEffect]) -> bool {
        self.effects.keys().all(|effect| allowed.contains(effect))
    }

    fn has_exact_effects(&self, required: &[ProjectSessionEffect]) -> bool {
        self.effects.len() == required.len()
            && required
                .iter()
                .all(|effect| self.effects.contains_key(effect))
    }

    fn has_all_effects(&self, required: &[ProjectSessionEffect]) -> bool {
        required
            .iter()
            .all(|effect| self.effects.contains_key(effect))
    }

    fn has_all_committed(&self, required: &[ProjectSessionEffect]) -> bool {
        required.iter().all(|effect| {
            self.disposition(*effect) == Some(ProjectSessionEffectDisposition::Committed)
        })
    }

    fn has_only_dispositions(&self, allowed: &[ProjectSessionEffectDisposition]) -> bool {
        self.effects
            .values()
            .all(|disposition| allowed.contains(disposition))
    }

    fn effects_have_only_dispositions(
        &self,
        effects: &[ProjectSessionEffect],
        allowed: &[ProjectSessionEffectDisposition],
    ) -> bool {
        effects.iter().all(|effect| {
            self.effects
                .get(effect)
                .map_or(true, |disposition| allowed.contains(disposition))
        })
    }
}
