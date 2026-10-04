use crate::core::play::{PlayPreviewFrame, PlayPreviewFrameIdentity};
use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};

use super::super::super::super::data::{HostViewportImageData, HostViewportOverlayImageData};
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_scene_viewport_surface_key(&self, surface_key: &str) {
        self.state.borrow_mut().set_scene_surface_key(surface_key);
    }

    pub(crate) fn scene_viewport_surface_key(&self) -> Option<String> {
        self.state.borrow().scene_surface_key.clone()
    }

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
            || scene.document_leaves.iter().any(|leaf| {
                leaf.pane.kind.as_str() == pane_kind
                    && leaf.content_frame.width > 0.0
                    && leaf.content_frame.height > 0.0
            })
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

    pub(crate) fn set_scene_viewport_capture_for_surface(
        &self,
        surface_key: &str,
        viewport: RenderViewportHandle,
        frame: CapturedFrame,
    ) -> bool {
        let Some(image) = HostViewportImageData::from_captured_frame(viewport, frame) else {
            return false;
        };
        self.state
            .borrow_mut()
            .replace_scene_viewport_image_for_surface(surface_key, image)
    }

    pub(crate) fn set_scene_viewport_product(&self, product: RenderViewportProduct) -> bool {
        let Some(image) = HostViewportImageData::from_viewport_product(product) else {
            return false;
        };
        self.state.borrow_mut().replace_scene_viewport_image(image)
    }

    pub(crate) fn set_scene_viewport_product_for_surface(
        &self,
        surface_key: &str,
        product: RenderViewportProduct,
    ) -> bool {
        let Some(image) = HostViewportImageData::from_viewport_product(product) else {
            return false;
        };
        self.state
            .borrow_mut()
            .replace_scene_viewport_image_for_surface(surface_key, image)
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
#[path = "tests/viewport.rs"]
mod tests;
