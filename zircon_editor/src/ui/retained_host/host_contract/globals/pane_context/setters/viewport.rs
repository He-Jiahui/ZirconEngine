use crate::core::play::{PlayPreviewFrame, PlayPreviewFrameIdentity};
use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};

use super::super::super::super::data::{HostViewportImageData, HostViewportOverlayImageData};
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn simulate_viewport_frame_identity(&self) -> Option<PlayPreviewFrameIdentity> {
        self.state
            .borrow()
            .viewport_images
            .simulate()
            .and_then(HostViewportImageData::play_frame_identity)
            .cloned()
    }

    pub(crate) fn game_viewport_visible(&self) -> bool {
        self.viewport_pane_visible("Game")
    }

    pub(crate) fn scene_viewport_visible(&self) -> bool {
        self.viewport_pane_visible("Scene")
    }

    fn viewport_pane_visible(&self, pane_kind: &str) -> bool {
        let state = self.state.borrow();
        let presentation = state.host_presentation.as_ref();
        let scene = &presentation.host_scene_data;
        let dock_contains_pane = [
            (
                &scene.document_dock.pane,
                &scene.document_dock.content_frame,
            ),
            (&scene.left_dock.pane, &scene.left_dock.content_frame),
            (&scene.right_dock.pane, &scene.right_dock.content_frame),
            (&scene.bottom_dock.pane, &scene.bottom_dock.content_frame),
        ]
        .into_iter()
        .any(|(pane, frame)| {
            pane.kind.as_str() == pane_kind && frame.width > 0.0 && frame.height > 0.0
        });

        dock_contains_pane
            || scene
                .floating_layer
                .floating_windows
                .iter()
                .any(|window| window.active_pane.kind.as_str() == pane_kind)
            || presentation
                .native_floating_surface_data
                .floating_windows
                .iter()
                .any(|window| window.active_pane.kind.as_str() == pane_kind)
    }

    pub(crate) fn set_scene_viewport_capture(
        &self,
        viewport: RenderViewportHandle,
        frame: CapturedFrame,
    ) -> bool {
        let Some(image) = HostViewportImageData::from_captured_frame(viewport, frame) else {
            return false;
        };
        self.state.borrow_mut().replace_scene_viewport_image(image)
    }

    pub(crate) fn set_scene_viewport_product(&self, product: RenderViewportProduct) -> bool {
        let Some(image) = HostViewportImageData::from_viewport_product(product) else {
            return false;
        };
        self.state.borrow_mut().replace_scene_viewport_image(image)
    }

    pub(crate) fn set_game_viewport_frame(&self, frame: PlayPreviewFrame) -> bool {
        let Some(image) = HostViewportImageData::from_play_preview_frame(frame) else {
            return false;
        };
        self.state.borrow_mut().replace_game_viewport_image(image)
    }

    pub(crate) fn set_simulate_viewport_frame(
        &self,
        frame: PlayPreviewFrame,
        overlay: Option<HostViewportOverlayImageData>,
    ) -> bool {
        let Some(image) = HostViewportImageData::from_play_preview_frame(frame) else {
            return false;
        };
        let Some(image) = image.with_overlay(overlay) else {
            return false;
        };
        self.state
            .borrow_mut()
            .replace_simulate_viewport_image(image)
    }

    pub(crate) fn clear_game_viewport_image(&self) -> bool {
        self.state.borrow_mut().clear_game_viewport_image()
    }

    pub(crate) fn clear_simulate_viewport_image(&self) -> bool {
        self.state.borrow_mut().clear_simulate_viewport_image()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
    use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
    use crate::ui::retained_host::primitives::PhysicalSize;

    use super::*;

    #[test]
    fn duplicate_viewport_capture_does_not_report_an_update() {
        let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
            640, 420,
        ))));
        let context = PaneSurfaceHostContext::from_state(state);

        assert!(context.set_scene_viewport_capture(
            RenderViewportHandle::new(3),
            CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 7),
        ));
        assert!(!context.set_scene_viewport_capture(
            RenderViewportHandle::new(3),
            CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 7),
        ));
    }

    #[test]
    fn game_viewport_visibility_uses_the_active_pane_instead_of_tab_existence() {
        let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
            640, 420,
        ))));
        let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));
        assert!(!context.game_viewport_visible());

        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.document_dock.pane.kind = "Game".into();
        presentation
            .host_scene_data
            .document_dock
            .content_frame
            .width = 640.0;
        presentation
            .host_scene_data
            .document_dock
            .content_frame
            .height = 360.0;
        state.borrow_mut().host_presentation = Arc::new(presentation);

        assert!(context.game_viewport_visible());
    }
}
