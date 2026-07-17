use crate::scene::viewport::ViewportState;
use crate::scene::viewport::{RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle};
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
        self.world.try_with_world(|scene| {
            let controller = &self.viewport_controller;
            let snapshot = controller.build_render_snapshot(scene);
            EditorRenderFrameSubmission {
                extract: RenderFrameExtract::from_snapshot(
                    RenderWorldSnapshotHandle::new(0),
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
}
