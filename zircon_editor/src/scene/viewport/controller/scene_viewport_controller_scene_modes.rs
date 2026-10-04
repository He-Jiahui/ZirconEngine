use crate::core::editor_message::SceneModeId;
use crate::core::extension::ContributionTicket;
use crate::core::tools::{AcquireOutcome, ToolDefinitionId, ToolOwnerGeneration};
use crate::scene::modes::{
    SceneModeActivation, SceneModeCtx, SceneModeRegistration, SceneModeRegistry,
    SceneModeRegistryError,
};

use super::{
    scene_viewport_controller::SceneToolIdentity, SceneViewportController,
    SceneViewportControllerError,
};

pub(crate) struct PreparedSceneModeContributionRetirement {
    pub(crate) ticket: ContributionTicket,
    pub(crate) registry: SceneModeRegistry,
}

impl SceneViewportController {
    fn scene_tool_id(
        &mut self,
    ) -> Result<crate::core::tools::ToolInstanceId, SceneViewportControllerError> {
        match &self.scene_tool_identity {
            SceneToolIdentity::Allocated(tool_id) => return Ok(tool_id.clone()),
            SceneToolIdentity::Failed(error) => return Err(error.clone().into()),
            SceneToolIdentity::Pending => {}
        }
        let definition = ToolDefinitionId::parse("editor.scene.viewport")?;
        match self
            .tool_scheduler
            .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
        {
            Ok(tool_id) => {
                self.scene_tool_identity = SceneToolIdentity::Allocated(tool_id.clone());
                Ok(tool_id)
            }
            Err(error) => {
                self.scene_tool_identity = SceneToolIdentity::Failed(error.clone());
                Err(error.into())
            }
        }
    }

    pub(super) fn ensure_scene_tool_lease(&mut self) -> Result<(), SceneViewportControllerError> {
        if self.scene_tool_lease.is_some() {
            return Ok(());
        }
        let tool = self.scene_tool_id()?;
        let resources = self.scene_tool_resources.clone();
        let report = self.tool_scheduler.acquire(tool, resources)?;
        match report.outcome() {
            AcquireOutcome::Acquired { lease } | AcquireOutcome::AlreadyHeld { lease } => {
                self.scene_tool_lease = Some(lease.clone());
                Ok(())
            }
            AcquireOutcome::Queued { request, position }
            | AcquireOutcome::AlreadyQueued { request, position } => {
                let _ = self.tool_scheduler.withdraw(request.id());
                Err(SceneViewportControllerError::SceneToolQueued {
                    position: *position,
                })
            }
            AcquireOutcome::Denied { reason, .. } => {
                Err(SceneViewportControllerError::SceneToolDenied {
                    reason: reason.clone(),
                })
            }
        }
    }

    pub(super) fn release_scene_tool_lease(&mut self) {
        if let Some(lease) = self.scene_tool_lease.take() {
            let _ = self.tool_scheduler.release(lease.id());
        }
    }

    pub(crate) fn prepare_scene_modes(
        &self,
        registrations: impl IntoIterator<Item = SceneModeRegistration>,
    ) -> Result<SceneModeRegistry, SceneModeRegistryError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        candidate_registry(&self.state.scene_mode_registry, &registrations)
    }

    pub(crate) fn install_prepared_scene_modes(&mut self, registry: SceneModeRegistry) {
        self.state.scene_mode_registry = registry;
        self.interaction_extract.invalidate();
    }

    pub(crate) fn push_scene_mode_overlay(
        &mut self,
        mode_id: &SceneModeId,
    ) -> Result<(), SceneViewportControllerError> {
        let activation = SceneModeActivation::Custom(mode_id.clone());
        activation.validate()?;
        let acquired_now = self.scene_tool_lease.is_none();
        self.ensure_scene_tool_lease()?;
        let (mode, contribution_ticket) = match self
            .state
            .scene_mode_registry
            .create_with_contribution(mode_id)
        {
            Ok(created) => created,
            Err(error) => {
                if acquired_now {
                    self.release_scene_tool_lease();
                }
                return Err(error.into());
            }
        };
        let result = {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state.scene_modes.push_overlay_with_contribution(
                activation,
                mode,
                contribution_ticket,
                &mut ctx,
            )
        };
        if let Err(error) = result {
            if acquired_now {
                self.release_scene_tool_lease();
            }
            return Err(error.into());
        }
        self.interaction_extract.invalidate();
        Ok(())
    }

    pub(crate) fn pop_scene_mode_overlay(&mut self) -> Option<SceneModeId> {
        let popped = {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state.scene_modes.pop(&mut ctx)
        };
        if popped.is_some() {
            if !self.state.scene_modes.requires_exclusive_tool() {
                self.release_scene_tool_lease();
            }
            self.interaction_extract.invalidate();
        }
        popped
    }

    pub(crate) fn update_scene_modes(&mut self) {
        let state = &mut self.state;
        let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state.scene_modes.update(&mut ctx);
        if ctx.take_overlay_invalidation() {
            self.interaction_extract.invalidate();
        }
    }

    pub(crate) fn shutdown_scene_modes(&mut self) {
        {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state.scene_modes.shutdown(&mut ctx);
        }
        self.release_scene_tool_lease();
        self.interaction_extract.invalidate();
    }

    pub(crate) fn prepare_scene_mode_contribution_retirement(
        &self,
        ticket: ContributionTicket,
    ) -> PreparedSceneModeContributionRetirement {
        let (registry_candidate, _) = self.state.scene_mode_registry.without_contribution(ticket);
        PreparedSceneModeContributionRetirement {
            ticket,
            registry: registry_candidate,
        }
    }

    pub(crate) fn install_prepared_scene_mode_contribution_retirement(
        &mut self,
        prepared: PreparedSceneModeContributionRetirement,
    ) -> (
        Vec<Box<dyn crate::scene::modes::EditorSceneMode>>,
        Option<String>,
    ) {
        let retirement = {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            let retirement = state
                .scene_modes
                .retire_contribution_to_builtin_select(prepared.ticket, &mut ctx);
            state.scene_mode_registry = prepared.registry;
            retirement
        };
        if !self.state.scene_modes.requires_exclusive_tool() {
            self.release_scene_tool_lease();
        }
        self.interaction_extract.invalidate();
        retirement
    }
}

fn candidate_registry(
    current: &SceneModeRegistry,
    registrations: &[SceneModeRegistration],
) -> Result<SceneModeRegistry, SceneModeRegistryError> {
    let mut candidate = current.clone();
    for registration in registrations {
        let mode_id = registration.mode_id().clone();
        candidate.register(registration.clone())?;
        candidate.create(&mode_id)?;
    }
    Ok(candidate)
}

#[cfg(test)]
#[path = "tests/scene_viewport_controller_scene_modes.rs"]
mod tests;
