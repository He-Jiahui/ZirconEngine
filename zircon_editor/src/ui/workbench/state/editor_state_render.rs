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
    pub ui: Option<UiRenderExtract>,
}

impl EditorState {
    pub fn render_snapshot(&self) -> Option<RenderSceneSnapshot> {
        self.world.try_with_world(|scene| {
            let controller = &self.viewport_controller;
            controller.build_render_snapshot(scene)
        })
    }

    pub(crate) fn render_frame_submission(&self) -> Option<EditorRenderFrameSubmission> {
        if !self.world.is_loaded() {
            return None;
        }
        let highlights = self.viewport_controller.build_runtime_highlight_set();
        if let Err(error) = self.context.gateway().submit_highlight_set(highlights) {
            emit_highlight_delivery_error(&self.context, error.to_string());
            return None;
        }
        self.world.try_with_world(|scene| {
            let controller = &self.viewport_controller;
            let snapshot = controller.build_render_snapshot(scene);
            EditorRenderFrameSubmission {
                extract: RenderFrameExtract::from_snapshot(
                    RenderWorldSnapshotHandle::new(scene.world_generation()),
                    snapshot,
                ),
                ui: controller.build_runtime_overlay_ui(),
            }
        })
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
        if self
            .world
            .try_with_world(|scene| {
                controller.sync_renderer_visible_spatial_snapshot(scene, snapshot)
            })
            .is_none()
        {
            controller.clear_renderer_visible_spatial_snapshot();
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
mod performance_tests {
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

        assert!(implementation.contains("try_with_world(|scene|"));
        assert!(implementation.contains("clear_renderer_visible_spatial_snapshot"));
    }
}
