use std::sync::Arc;

use crate::core::editor_event::ViewInstanceId;
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

    pub(crate) fn render_frame_submission_for_view(
        &mut self,
        view_id: &ViewInstanceId,
        size: zircon_runtime_interface::math::UVec2,
        runtime_viewport: zircon_runtime_interface::ZrRuntimeViewportHandle,
    ) -> Option<EditorRenderFrameSubmission> {
        if !self.world.is_loaded() {
            return None;
        }
        let controller = self.viewport_controller.session(view_id);
        controller.resize(size);
        controller.set_runtime_viewport(runtime_viewport);
        let highlights = controller.build_runtime_highlight_set();
        if let Err(error) = self
            .context
            .authoring_gateway()
            .submit_highlight_set(highlights)
        {
            emit_highlight_delivery_error(&self.context, error.to_string());
        }
        let world = &self.world;
        let viewport_controller = &mut self.viewport_controller;
        match world.with_world(|scene| {
            let controller = viewport_controller.session(view_id);
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
            Ok(_) => {}
            Err(error) => {
                controller.clear_renderer_visible_spatial_snapshot();
                self.report_authoring_world_access_failure(
                    "visible-spatial synchronization",
                    &error,
                );
            }
        }
    }

    pub(crate) fn sync_renderer_visible_spatial_snapshot_for_view(
        &mut self,
        view_id: &ViewInstanceId,
        snapshot: Option<RenderVisibleSpatialQuerySnapshot>,
    ) {
        let world = &self.world;
        let controller = self.viewport_controller.session(view_id);
        match world
            .with_world(|scene| controller.sync_renderer_visible_spatial_snapshot(scene, snapshot))
        {
            Ok(_) => {}
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
#[path = "tests/editor_state_render.rs"]
mod tests;
