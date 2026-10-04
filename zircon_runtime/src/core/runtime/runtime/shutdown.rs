use std::time::{Duration, Instant};

use super::CoreRuntime;
use crate::core::CoreError;

/// One dependency-safe shutdown attempt, in reverse activation order.
///
/// Failed and blocked modules keep their original lifecycle owner and retry
/// eligibility. Completed modules leave the active ledger; later attempts list
/// only the modules that still need cleanup.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleShutdownReport {
    // Preserve the first traversal error for existing result-based callers.
    first_error: Option<CoreError>,
    pub attempted: Vec<String>,
    pub completed: Vec<String>,
    /// Cleanup and ledger publication finished, but the shared deadline was missed.
    pub completed_after_deadline: Vec<String>,
    pub failed: Vec<(String, CoreError)>,
    pub blocked: Vec<(String, CoreError)>,
    pub not_attempted: Vec<String>,
    pub deadline_exhausted: bool,
}

impl ModuleShutdownReport {
    pub fn is_complete(&self) -> bool {
        self.failed.is_empty()
            && self.blocked.is_empty()
            && self.not_attempted.is_empty()
            && !self.deadline_exhausted
            && self.completed_after_deadline.is_empty()
            && self.completed.len() == self.attempted.len()
    }

    /// Project the full receipt onto the existing result-based host contract.
    pub fn into_result(self) -> Result<(), CoreError> {
        if self.is_complete() {
            return Ok(());
        }
        if let Some(error) = self.first_error {
            return Err(error);
        }
        if let Some((_, error)) = self.failed.into_iter().next() {
            return Err(error);
        }
        if let Some((_, error)) = self.blocked.into_iter().next() {
            return Err(error);
        }
        Err(CoreError::ModuleCleanupTimeout {
            module: self
                .not_attempted
                .first()
                .or_else(|| self.completed_after_deadline.first())
                .cloned()
                .unwrap_or_else(|| "runtime modules".to_owned()),
            operation: "module_deactivation".to_owned(),
            budget: Duration::ZERO,
            incomplete_entries: self.not_attempted.len().max(1),
            failed: 0,
            cancelled: 0,
        })
    }
}

enum ModuleShutdownBudget {
    ImmediateDrainProbe,
    Deadline(Instant),
}

impl CoreRuntime {
    /// Every module shares this timeout; an independent failure does not stop
    /// cleanup of unrelated modules. Use the deadline API for the full report.
    pub fn shutdown_registered_modules_with_drain_timeout(
        &self,
        drain_timeout: Duration,
    ) -> Result<(), CoreError> {
        // Existing zero-duration callers probe service guards immediately and
        // still clean up ready modules. Only the Instant API means an expired deadline.
        if drain_timeout.is_zero() {
            return self
                .shutdown_registered_modules_with_budget(ModuleShutdownBudget::ImmediateDrainProbe)
                .into_result();
        }
        let started_at = Instant::now();
        let deadline = started_at.checked_add(drain_timeout).unwrap_or(started_at);
        self.shutdown_registered_modules_until(deadline)
            .into_result()
    }

    /// All stages receive the caller's absolute deadline. Dependency blockers
    /// are reported separately from cleanup failures and never bypassed.
    pub fn shutdown_registered_modules_until(&self, deadline: Instant) -> ModuleShutdownReport {
        self.shutdown_registered_modules_with_budget(ModuleShutdownBudget::Deadline(deadline))
    }

    fn shutdown_registered_modules_with_budget(
        &self,
        budget: ModuleShutdownBudget,
    ) -> ModuleShutdownReport {
        let shutdown_order = self.handle.active_module_shutdown_order();
        let mut report = ModuleShutdownReport::default();
        for module_name in shutdown_order.iter().rev() {
            if matches!(&budget, ModuleShutdownBudget::Deadline(deadline) if Instant::now() >= *deadline)
            {
                report.deadline_exhausted = true;
                report.not_attempted.push(module_name.clone());
                continue;
            }
            report.attempted.push(module_name.clone());
            let result = match &budget {
                ModuleShutdownBudget::ImmediateDrainProbe => self
                    .handle
                    .deactivate_module_with_drain_timeout(module_name, Duration::ZERO),
                ModuleShutdownBudget::Deadline(deadline) => {
                    self.handle.deactivate_module_until(module_name, *deadline)
                }
            };
            match result {
                Ok(()) => {
                    report.completed.push(module_name.clone());
                    // Destructors, unload notifications and the terminal ledger mutation
                    // happen after the last lower check. Their lateness belongs in the
                    // receipt even when this was the last active module.
                    if matches!(&budget, ModuleShutdownBudget::Deadline(deadline) if Instant::now() >= *deadline)
                    {
                        report.deadline_exhausted = true;
                        report.completed_after_deadline.push(module_name.clone());
                        report
                            .first_error
                            .get_or_insert_with(|| CoreError::ModuleCleanupTimeout {
                                module: module_name.clone(),
                                operation: "module_shutdown_completed_after_deadline".to_owned(),
                                budget: Duration::ZERO,
                                incomplete_entries: 0,
                                failed: 0,
                                cancelled: 0,
                            });
                    }
                }
                Err(
                    error @ (CoreError::ModuleUnloadBlocked { .. } | CoreError::UnloadBlocked(..)),
                ) => {
                    report.first_error.get_or_insert_with(|| error.clone());
                    report.blocked.push((module_name.clone(), error));
                }
                Err(error) => {
                    report.first_error.get_or_insert_with(|| error.clone());
                    if matches!(
                        &error,
                        CoreError::ModuleCleanupTimeout { .. }
                            | CoreError::ServiceCallDrainTimeout { .. }
                    ) {
                        report.deadline_exhausted |= matches!(&budget,
                            ModuleShutdownBudget::Deadline(deadline) if Instant::now() >= *deadline);
                    }
                    report.failed.push((module_name.clone(), error));
                }
            }
        }
        report
    }
}
