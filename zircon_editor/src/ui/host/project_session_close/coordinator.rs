use super::{ProjectCloseCoordinatorPhase, ProjectCloseOperation, ProjectCloseTransitionError};

#[derive(Debug, Default)]
pub(crate) struct ProjectCloseCoordinator {
    phase: ProjectCloseCoordinatorPhase,
    operation: Option<ProjectCloseOperation>,
}

impl ProjectCloseCoordinator {
    pub(crate) const fn phase(&self) -> ProjectCloseCoordinatorPhase {
        self.phase
    }

    pub(crate) fn operation(&self) -> Option<&ProjectCloseOperation> {
        self.operation.as_ref()
    }

    pub(crate) fn begin_quiescing(
        &mut self,
        operation: ProjectCloseOperation,
    ) -> Result<(), ProjectCloseTransitionError> {
        if self.phase != ProjectCloseCoordinatorPhase::Decision {
            return Err(self.invalid_transition(ProjectCloseCoordinatorPhase::Quiescing));
        }
        self.operation = Some(operation);
        self.phase = ProjectCloseCoordinatorPhase::Quiescing;
        Ok(())
    }

    pub(crate) fn begin_committing(
        &mut self,
        operation: &ProjectCloseOperation,
    ) -> Result<(), ProjectCloseTransitionError> {
        self.require_operation(operation)?;
        if self.phase != ProjectCloseCoordinatorPhase::Quiescing {
            return Err(self.invalid_transition(ProjectCloseCoordinatorPhase::Committing));
        }
        self.phase = ProjectCloseCoordinatorPhase::Committing;
        Ok(())
    }

    pub(crate) fn finish_closed(
        &mut self,
        operation: &ProjectCloseOperation,
    ) -> Result<(), ProjectCloseTransitionError> {
        self.require_operation(operation)?;
        if self.phase != ProjectCloseCoordinatorPhase::Committing {
            return Err(self.invalid_transition(ProjectCloseCoordinatorPhase::Closed));
        }
        self.phase = ProjectCloseCoordinatorPhase::Closed;
        Ok(())
    }

    pub(crate) fn require_recovery(
        &mut self,
        operation: &ProjectCloseOperation,
    ) -> Result<(), ProjectCloseTransitionError> {
        self.require_operation(operation)?;
        if !matches!(
            self.phase,
            ProjectCloseCoordinatorPhase::Quiescing | ProjectCloseCoordinatorPhase::Committing
        ) {
            return Err(self.invalid_transition(ProjectCloseCoordinatorPhase::RecoveryRequired));
        }
        self.phase = ProjectCloseCoordinatorPhase::RecoveryRequired;
        Ok(())
    }

    pub(crate) fn reset_for_new_session(&mut self) -> Result<(), ProjectCloseTransitionError> {
        if self.phase != ProjectCloseCoordinatorPhase::Closed {
            return Err(self.invalid_transition(ProjectCloseCoordinatorPhase::Decision));
        }
        self.phase = ProjectCloseCoordinatorPhase::Decision;
        self.operation = None;
        Ok(())
    }

    fn require_operation(
        &self,
        requested: &ProjectCloseOperation,
    ) -> Result<(), ProjectCloseTransitionError> {
        let Some(active) = self.operation.as_ref() else {
            return Err(ProjectCloseTransitionError::MissingOperation { phase: self.phase });
        };
        if active == requested {
            return Ok(());
        }
        Err(ProjectCloseTransitionError::OperationMismatch {
            active_root: active.project_root().to_path_buf(),
            active_operation_id: active.operation_id(),
            requested_root: requested.project_root().to_path_buf(),
            requested_operation_id: requested.operation_id(),
        })
    }

    fn invalid_transition(
        &self,
        requested: ProjectCloseCoordinatorPhase,
    ) -> ProjectCloseTransitionError {
        ProjectCloseTransitionError::InvalidTransition {
            current: self.phase,
            requested,
        }
    }
}
