use std::sync::Arc;

use crate::core::logging::{LogEntry, LogSeverity, LogSource};
use crate::scene::viewport::ViewportState;
use crate::scene::viewport::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVisibleSpatialQuerySnapshot,
    RenderWorldSnapshotHandle,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::editor_state::EditorState;

#[derive(Clone, Debug)]
pub(crate) struct EditorRenderFrameSubmission {
    pub extract: RenderFrameExtract,
    pub ui: Option<Arc<UiRenderExtract>>,
}

impl EditorState {
    pub fn render_snapshot(&self) -> Option<RenderSceneSnapshot> {
        match self.world.with_world(|scene| {
            let controller = &self.viewport_controller;
            controller.build_render_snapshot(scene)
        }) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.report_authoring_world_access_failure("render snapshot", &error);
                None
            }
        }
    }

    pub(crate) fn render_frame_submission(&self) -> Option<EditorRenderFrameSubmission> {
        if !self.world.is_loaded() {
            return None;
        }
        let highlights = self.viewport_controller.build_runtime_highlight_set();
        if let Err(error) = self
            .context
            .authoring_gateway()
            .submit_highlight_set(highlights)
        {
            emit_highlight_delivery_error(&self.context, error.to_string());
        }
        match self.world.with_world(|scene| {
            let controller = &self.viewport_controller;
            let snapshot = controller.build_render_snapshot(scene);
            EditorRenderFrameSubmission {
                extract: RenderFrameExtract::from_snapshot(
                    RenderWorldSnapshotHandle::new(scene.world_generation()),
                    snapshot,
                ),
                ui: controller.build_runtime_overlay_ui(),
            }
        }) {
            Ok(submission) => submission,
            Err(error) => {
                self.report_authoring_world_access_failure("render frame extraction", &error);
                None
            }
        }
    }

    pub fn render_frame_extract(&self) -> Option<RenderFrameExtract> {
        self.render_frame_submission()
            .map(|submission| submission.extract)
    }

    pub fn viewport_state(&self) -> ViewportState {
        self.viewport_controller.viewport().clone()
    }

    pub(crate) fn sync_renderer_visible_spatial_snapshot(
        &mut self,
        snapshot: Option<RenderVisibleSpatialQuerySnapshot>,
    ) {
        let controller = &mut self.viewport_controller;
        match self
            .world
            .with_world(|scene| controller.sync_renderer_visible_spatial_snapshot(scene, snapshot))
        {
            Ok(Some(())) => {}
            Ok(None) => controller.clear_renderer_visible_spatial_snapshot(),
            Err(error) => {
                controller.clear_renderer_visible_spatial_snapshot();
                self.report_authoring_world_access_failure(
                    "visible-spatial synchronization",
                    &error,
                );
            }
        }
    }
}

fn emit_highlight_delivery_error(context: &crate::core::context::EditorContext, message: String) {
    let entry = LogEntry::new(
        LogSource::runtime(),
        LogSeverity::Error,
        format!("editor viewport highlight delivery failed: {message}"),
        0,
        None,
    );
    if let Ok(entry) = entry {
        let _ = context.logs().emit(entry);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::gateway::{EditorRuntimeGateway, GatewayError};
    use zircon_runtime::scene::{DefaultLevelManager, LevelSystem, World};
    use zircon_runtime_interface::math::UVec2;
    use zircon_runtime_interface::{
        ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
        ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
    };

    use super::*;

    #[test]
    fn render_submission_borrows_the_viewport_controller() {
        let source = include_str!("editor_state_render.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(!implementation.contains("clone_for_render()"));
        assert!(implementation.contains("let controller = &self.viewport_controller"));
    }

    #[test]
    fn render_submission_binds_the_extract_to_the_scene_generation() {
        let source = include_str!("editor_state_render.rs");
        let implementation = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(production, _)| production);

        assert!(implementation.contains("RenderWorldSnapshotHandle::new(scene.world_generation())"));
        assert!(!implementation.contains("RenderWorldSnapshotHandle::new(0)"));
    }

    #[test]
    fn renderer_visible_snapshot_is_only_adopted_through_the_current_scene() {
        let source = include_str!("editor_state_render.rs");
        let implementation = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(production, _)| production);

        assert!(implementation.contains("with_world(|scene|"));
        assert!(implementation.contains("clear_renderer_visible_spatial_snapshot"));
    }

    #[test]
    fn render_frame_submission_keeps_the_base_scene_when_highlight_delivery_fails() {
        let manager = DefaultLevelManager::default();
        let level = manager.create_default_level();
        let state = EditorState::with_default_selection(level.clone(), UVec2::new(1280, 720));
        state
            .context
            .authoring_gateway()
            .replace(Arc::new(HighlightFailingGateway { level }))
            .expect("install highlight delivery fault gateway");

        let submission = state
            .render_frame_submission()
            .expect("a highlight delivery fault must not discard the base scene frame");

        assert_eq!(
            submission.extract.world.raw(),
            state.world.with_world(|scene| scene.world_generation())
        );
    }

    struct HighlightFailingGateway {
        level: LevelSystem,
    }

    impl EditorRuntimeGateway for HighlightFailingGateway {
        fn session_handle(&self) -> ZrRuntimeSessionHandle {
            ZrRuntimeSessionHandle::invalid()
        }

        fn session_identity(&self) -> crate::core::gateway::GatewaySessionIdentity {
            crate::core::gateway::GatewaySessionIdentity::detached()
        }

        fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
            self.level.with_world(read);
            Ok(())
        }

        fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
            self.level.with_world_mut(write);
            Ok(())
        }

        fn submit_highlight_set(
            &self,
            _set: crate::core::gateway::EditorRuntimeHighlightSet,
        ) -> Result<(), GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.editor_overlay.highlight_set",
            })
        }

        fn submit_operation(
            &self,
            _request: ZrRuntimeOperationSubmitRequestV1,
        ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.submit",
            })
        }

        fn poll_operation(
            &self,
            _handle: ZrRuntimeOperationHandle,
        ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.poll",
            })
        }

        fn harvest_operation(
            &self,
            _handle: ZrRuntimeOperationHandle,
        ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.harvest",
            })
        }
    }
}
