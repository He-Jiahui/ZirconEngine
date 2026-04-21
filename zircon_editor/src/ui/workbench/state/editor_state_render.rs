use crate::scene::viewport::ViewportState;
use crate::scene::viewport::{RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle};
use zircon_runtime::ui::surface::UiRenderExtract;

use super::editor_state::EditorState;

#[derive(Clone, Debug)]
pub(crate) struct EditorRenderFrameSubmission {
    pub extract: RenderFrameExtract,
    pub ui: Option<UiRenderExtract>,
}

impl EditorState {
    pub fn render_snapshot(&self) -> Option<RenderSceneSnapshot> {
        self.world.try_with_world(|scene| {
            let controller = self.viewport_controller.clone_for_render();
            controller.build_render_snapshot(scene)
        })
    }

    pub(crate) fn render_frame_submission(&self) -> Option<EditorRenderFrameSubmission> {
        self.world.try_with_world(|scene| {
            let controller = self.viewport_controller.clone_for_render();
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
