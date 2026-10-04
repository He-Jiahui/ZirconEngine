use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::context::{ToolSchedulerService, ToolSchedulerServiceError};
use crate::core::editing::interactive_transform::PivotMode;
use crate::core::editor_event::ViewInstanceId;
use crate::core::settings::SettingsMutationCoordinator;
use crate::core::tools::{ToolInstanceId, ToolLeaseHandle, ToolResourceSet};
use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;
use crate::scene::viewport::{
    SceneViewportChromeSettings, SceneViewportSettings, ViewportCameraSnapshot,
    ViewportInteractionExtractCache,
};
use zircon_runtime::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderDynamicResolutionSettings,
};
use zircon_runtime_interface::math::{
    is_finite_mat4, is_finite_quat, is_finite_scalar, is_finite_vec3, Mat4, Real, Transform, Vec3,
};

use super::scene_viewport_controller_build_runtime_overlay_ui::RuntimeOverlayUiExtractCache;
use super::scene_viewport_controller_error::SceneViewportControllerError;
use super::scene_viewport_controller_overlay_providers::ViewportOverlayProviderRegistry;
use super::scene_viewport_controller_scene_modes::PreparedSceneModeContributionRetirement;
use super::scene_viewport_state::SceneViewportState;

pub(crate) struct SceneViewportController {
    pub(in crate::scene::viewport::controller) state: SceneViewportState,
    pub(in crate::scene::viewport::controller) handles: HandleToolRegistry,
    pub(in crate::scene::viewport::controller) interaction_extract: ViewportInteractionExtractCache,
    pub(in crate::scene::viewport::controller) pointer_bridge: ViewportOverlayPointerRouter,
    pub(in crate::scene::viewport::controller) settings_mutations: Arc<SettingsMutationCoordinator>,
    pub(in crate::scene::viewport::controller) overlay_providers: ViewportOverlayProviderRegistry,
    pub(in crate::scene::viewport::controller) runtime_overlay_ui_cache:
        RefCell<RuntimeOverlayUiExtractCache>,
    pub(in crate::scene::viewport::controller) tool_scheduler: ToolSchedulerService,
    pub(in crate::scene::viewport::controller) scene_tool_identity: SceneToolIdentity,
    pub(in crate::scene::viewport::controller) scene_tool_resources: ToolResourceSet,
    pub(in crate::scene::viewport::controller) scene_tool_lease: Option<ToolLeaseHandle>,
}

pub(crate) struct SceneViewportSessionRegistry {
    sessions: BTreeMap<ViewInstanceId, SceneViewportController>,
    active: Option<ViewInstanceId>,
    detached: SceneViewportController,
}

/// Durable, per-Scene-view authoring state. Viewport dimensions, temporal jitter, pointer routing,
/// selection, and tool leases are reconstructed by the live workbench/runtime after reopening.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SceneViewportWorkspaceSessionSnapshot {
    pub settings: SceneViewportSettings,
    pub pivot_mode: PivotMode,
    pub orbit_target: Vec3,
    pub camera: Option<SceneViewportCameraSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SceneViewportCameraSnapshot {
    pub transform: Transform,
    pub core_pipeline: CorePipelineKind,
    pub projection_mode: ProjectionMode,
    pub fov_y_radians: Real,
    pub ortho_size: Real,
    pub z_near: Real,
    pub z_far: Real,
    pub projection_override: Option<Mat4>,
    pub is_active: bool,
    pub hdr: bool,
    pub exposure_ev100: Real,
    pub msaa_samples: u32,
    pub dynamic_resolution: RenderDynamicResolutionSettings,
}

impl SceneViewportWorkspaceSessionSnapshot {
    fn capture(controller: &SceneViewportController) -> Self {
        Self {
            settings: controller.state.settings.clone(),
            pivot_mode: controller.state.pivot_mode,
            orbit_target: controller.state.orbit_target,
            camera: controller
                .state
                .camera
                .as_ref()
                .map(SceneViewportCameraSnapshot::from),
        }
    }

    fn restore_into(&self, controller: &mut SceneViewportController) {
        controller.state.settings = self.settings.clone();
        controller.state.pivot_mode = self.pivot_mode;
        controller.state.orbit_target = if is_finite_vec3(self.orbit_target) {
            self.orbit_target
        } else {
            Vec3::ZERO
        };
        controller
            .state
            .orbit_controller
            .set_target(controller.state.orbit_target);
        controller.state.camera = self
            .camera
            .as_ref()
            .and_then(|camera| camera.to_viewport_camera_snapshot(self.settings.projection_mode));
    }
}

impl From<&ViewportCameraSnapshot> for SceneViewportCameraSnapshot {
    fn from(camera: &ViewportCameraSnapshot) -> Self {
        Self {
            transform: camera.transform,
            core_pipeline: camera.core_pipeline,
            projection_mode: camera.projection_mode,
            fov_y_radians: camera.fov_y_radians,
            ortho_size: camera.ortho_size,
            z_near: camera.z_near,
            z_far: camera.z_far,
            projection_override: camera.projection_override,
            is_active: camera.is_active,
            hdr: camera.hdr,
            exposure_ev100: camera.exposure_ev100,
            msaa_samples: camera.msaa_samples,
            dynamic_resolution: camera.dynamic_resolution,
        }
    }
}

impl SceneViewportCameraSnapshot {
    fn to_viewport_camera_snapshot(
        &self,
        settings_projection_mode: ProjectionMode,
    ) -> Option<ViewportCameraSnapshot> {
        let projection_override_is_finite = self.projection_override.is_none_or(is_finite_mat4);
        let camera_is_finite = is_finite_vec3(self.transform.translation)
            && is_finite_quat(self.transform.rotation)
            && is_finite_vec3(self.transform.scale)
            && self.transform.rotation.length_squared() > Real::EPSILON
            && is_finite_scalar(self.fov_y_radians)
            && is_finite_scalar(self.ortho_size)
            && is_finite_scalar(self.z_near)
            && is_finite_scalar(self.z_far)
            && is_finite_scalar(self.exposure_ev100)
            && is_finite_scalar(self.dynamic_resolution.scale)
            && projection_override_is_finite;
        if !camera_is_finite
            || self.fov_y_radians <= 0.0
            || self.fov_y_radians >= core::f32::consts::PI
            || self.ortho_size <= 0.0
            || self.z_near <= 0.0
            || self.z_far <= self.z_near
        {
            return None;
        }

        Some(ViewportCameraSnapshot {
            transform: self.transform,
            core_pipeline: self.core_pipeline,
            projection_mode: settings_projection_mode,
            fov_y_radians: self.fov_y_radians,
            ortho_size: self.ortho_size,
            z_near: self.z_near,
            z_far: self.z_far,
            // Aspect ratio is derived from the restored leaf's first real viewport resize.
            aspect_ratio: ViewportCameraSnapshot::default().aspect_ratio,
            projection_override: self.projection_override,
            is_active: self.is_active,
            hdr: self.hdr,
            exposure_ev100: self.exposure_ev100,
            msaa_samples: self.msaa_samples,
            dynamic_resolution: self.dynamic_resolution,
            temporal_jitter: Default::default(),
        })
    }
}

impl SceneViewportSessionRegistry {
    /// Returns an already-retained session without forking a retired/stale ID.
    pub(crate) fn session_if_live(
        &self,
        view_id: &ViewInstanceId,
    ) -> Option<&SceneViewportController> {
        self.sessions.get(view_id)
    }

    /// Applies a command to one retained Scene leaf.  Unlike `session`, this
    /// method never creates a session for an unknown ID, so stale toolbar
    /// events cannot mutate the active/default viewport.
    pub(crate) fn apply_command_for_view(
        &mut self,
        view_id: &ViewInstanceId,
        scene: Option<&zircon_runtime::scene::Scene>,
        command: &crate::ui::binding::ViewportCommand,
    ) -> Result<crate::scene::viewport::ViewportFeedback, SceneViewportControllerError> {
        self.sessions
            .get_mut(view_id)
            .ok_or_else(|| SceneViewportControllerError::StaleView {
                view_id: view_id.clone(),
            })?
            .apply_command(scene, command)
    }

    pub(crate) fn chrome_settings_for_view(
        &self,
        view_id: &ViewInstanceId,
    ) -> Option<SceneViewportChromeSettings> {
        self.sessions
            .get(view_id)
            .map(SceneViewportController::chrome_settings)
    }

    #[cfg(test)]
    pub(crate) fn overlay_provider_enabled_for_test(&self, provider_id: &str) -> Option<bool> {
        self.active_controller()
            .overlay_provider_enabled_for_test(provider_id)
    }

    pub(crate) fn new(controller: SceneViewportController, active: ViewInstanceId) -> Self {
        let detached = controller.detached_for_empty_retention();
        let mut sessions = BTreeMap::new();
        sessions.insert(active.clone(), controller);
        Self {
            sessions,
            active: Some(active),
            detached,
        }
    }

    pub(crate) fn focus(&mut self, view_id: ViewInstanceId) -> bool {
        let changed = self.active.as_ref() != Some(&view_id);
        if !self.sessions.contains_key(&view_id) {
            let fork = self.active_controller().fork_for_view(view_id.clone());
            self.sessions.insert(view_id.clone(), fork);
        }
        if changed {
            let selection = self.active_controller().selection().clone();
            *self
                .sessions
                .get_mut(&view_id)
                .expect("the focused viewport session must exist")
                .selection_mut() = selection;
        }
        self.active = Some(view_id);
        changed
    }

    pub(crate) fn session(&mut self, view_id: &ViewInstanceId) -> &mut SceneViewportController {
        if !self.sessions.contains_key(view_id) {
            let fork = self.active_controller().fork_for_view(view_id.clone());
            self.sessions.insert(view_id.clone(), fork);
        }
        if self.active.as_ref() != Some(view_id) {
            let selection = self.active_controller().selection().clone();
            *self
                .sessions
                .get_mut(view_id)
                .expect("the viewport session was inserted above")
                .selection_mut() = selection;
        }
        self.sessions
            .get_mut(view_id)
            .expect("the viewport session was inserted above")
    }

    pub(crate) fn snapshot_workspace_sessions(
        &self,
        live_views: &std::collections::BTreeSet<ViewInstanceId>,
    ) -> BTreeMap<ViewInstanceId, SceneViewportWorkspaceSessionSnapshot> {
        live_views
            .iter()
            .map(|view_id| {
                let controller = self.sessions.get(view_id).unwrap_or(&self.detached);
                (
                    view_id.clone(),
                    SceneViewportWorkspaceSessionSnapshot::capture(controller),
                )
            })
            .collect()
    }

    pub(crate) fn restore_workspace_sessions(
        &mut self,
        live_views: &std::collections::BTreeSet<ViewInstanceId>,
        snapshots: &BTreeMap<ViewInstanceId, SceneViewportWorkspaceSessionSnapshot>,
        focused_view: Option<&ViewInstanceId>,
    ) {
        self.retain(&std::collections::BTreeSet::new());
        for view_id in live_views {
            let session = self.session(view_id);
            if let Some(snapshot) = snapshots.get(view_id) {
                snapshot.restore_into(session);
            }
        }
        self.active = focused_view
            .filter(|view_id| live_views.contains(*view_id))
            .cloned()
            .or_else(|| live_views.iter().next().cloned());
    }

    #[cfg(test)]
    pub(crate) fn session_count_for_test(&self) -> usize {
        self.sessions.len()
    }

    pub(crate) fn install_prepared_scene_modes(
        &mut self,
        registry: crate::scene::modes::SceneModeRegistry,
    ) {
        self.detached.install_prepared_scene_modes(registry.clone());
        for controller in self.sessions.values_mut() {
            controller.install_prepared_scene_modes(registry.clone());
        }
    }

    pub(crate) fn install_prepared_scene_mode_contribution_retirement(
        &mut self,
        prepared: PreparedSceneModeContributionRetirement,
    ) -> (
        Vec<Box<dyn crate::scene::modes::EditorSceneMode>>,
        Option<String>,
    ) {
        let ticket = prepared.ticket;
        let mut retired_modes = Vec::new();
        let (retired, mut first_error) = self
            .detached
            .install_prepared_scene_mode_contribution_retirement(
                PreparedSceneModeContributionRetirement {
                    ticket,
                    registry: prepared.registry.clone(),
                },
            );
        retired_modes.extend(retired);
        for controller in self.sessions.values_mut() {
            let registry = prepared.registry.clone();
            let (retired, error) = controller.install_prepared_scene_mode_contribution_retirement(
                PreparedSceneModeContributionRetirement { ticket, registry },
            );
            retired_modes.extend(retired);
            if first_error.is_none() {
                first_error = error;
            }
        }
        (retired_modes, first_error)
    }

    pub(crate) fn install_prepared_viewport_overlay_providers(
        &mut self,
        registry: ViewportOverlayProviderRegistry,
    ) {
        self.detached
            .install_prepared_viewport_overlay_providers(registry.clone());
        for controller in self.sessions.values_mut() {
            controller.install_prepared_viewport_overlay_providers(registry.clone());
        }
    }

    pub(crate) fn set_viewport_overlay_capabilities<I, S>(&mut self, capabilities: I)
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<str> + Clone,
    {
        self.detached
            .set_viewport_overlay_capabilities(capabilities.clone());
        for controller in self.sessions.values_mut() {
            controller.set_viewport_overlay_capabilities(capabilities.clone());
        }
    }

    pub(crate) fn retain(&mut self, retained: &std::collections::BTreeSet<ViewInstanceId>) {
        if retained.is_empty() {
            let detached = self.active_controller().detached_for_empty_retention();
            let retired = self.sessions.keys().cloned().collect::<Vec<_>>();
            for view_id in retired {
                if let Some(mut controller) = self.sessions.remove(&view_id) {
                    controller.shutdown_scene_modes();
                }
            }
            self.detached.shutdown_scene_modes();
            self.detached = detached;
            self.active = None;
            return;
        }
        for view_id in retained {
            self.session(view_id);
        }
        if self
            .active
            .as_ref()
            .map_or(true, |active| !retained.contains(active))
        {
            self.active = Some(
                retained
                    .iter()
                    .next()
                    .expect("non-empty retained viewport set")
                    .clone(),
            );
        }
        let retired = self
            .sessions
            .keys()
            .filter(|view_id| !retained.contains(*view_id))
            .cloned()
            .collect::<Vec<_>>();
        for view_id in retired {
            if let Some(mut controller) = self.sessions.remove(&view_id) {
                controller.shutdown_scene_modes();
            }
        }
    }

    pub(crate) fn shutdown_scene_modes(&mut self) {
        self.detached.shutdown_scene_modes();
        for controller in self.sessions.values_mut() {
            controller.shutdown_scene_modes();
        }
    }

    fn active_controller(&self) -> &SceneViewportController {
        self.active
            .as_ref()
            .and_then(|view_id| self.sessions.get(view_id))
            .unwrap_or(&self.detached)
    }
}

impl Deref for SceneViewportSessionRegistry {
    type Target = SceneViewportController;

    fn deref(&self) -> &Self::Target {
        self.active_controller()
    }
}

impl DerefMut for SceneViewportSessionRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let active = self.active.clone();
        if let Some(controller) = active.and_then(|view_id| self.sessions.get_mut(&view_id)) {
            controller
        } else {
            &mut self.detached
        }
    }
}

#[derive(Debug)]
pub(in crate::scene::viewport::controller) enum SceneToolIdentity {
    Pending,
    Allocated(ToolInstanceId),
    Failed(ToolSchedulerServiceError),
}

impl SceneToolIdentity {
    pub(super) fn allocated(&self) -> Option<&ToolInstanceId> {
        match self {
            Self::Allocated(tool_id) => Some(tool_id),
            Self::Pending | Self::Failed(_) => None,
        }
    }
}

impl Drop for SceneViewportController {
    fn drop(&mut self) {
        if let Some(lease) = self.scene_tool_lease.take() {
            let _ = self.tool_scheduler.release(lease.id());
        }
    }
}

#[cfg(test)]
#[path = "tests/scene_viewport_controller.rs"]
mod tests;
