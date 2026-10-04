use std::sync::Arc;

use crate::core::play::{PlayPreviewFrame, PlayPreviewFrameIdentity};
use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};

mod overlay;

pub(crate) use overlay::HostViewportOverlayImageData;

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageSet {
    scene: Option<Arc<HostViewportImageData>>,
    scene_by_surface: std::collections::BTreeMap<String, Arc<HostViewportImageData>>,
    simulate: Option<Arc<HostViewportImageData>>,
    game: Option<Arc<HostViewportImageData>>,
}

impl HostViewportImageSet {
    pub(crate) fn scene(&self) -> Option<&HostViewportImageData> {
        self.scene.as_deref()
    }

    pub(crate) fn game(&self) -> Option<&HostViewportImageData> {
        self.game.as_deref()
    }

    pub(crate) fn simulate(&self) -> Option<&HostViewportImageData> {
        self.simulate.as_deref()
    }

    pub(crate) fn for_pane(&self, pane_kind: &str) -> Option<&HostViewportImageData> {
        match pane_kind {
            "Scene" => self.simulate().or_else(|| self.scene()),
            "Game" => self.game(),
            _ => None,
        }
    }

    pub(crate) fn replace_scene(&mut self, image: HostViewportImageData) -> bool {
        Self::replace(&mut self.scene, image)
    }

    pub(crate) fn replace_scene_for_surface(
        &mut self,
        surface_key: impl Into<String>,
        image: HostViewportImageData,
    ) -> bool {
        let surface_key = surface_key.into();
        if surface_key.is_empty() {
            return self.replace_scene(image);
        }
        if self
            .scene_by_surface
            .get(&surface_key)
            .is_some_and(|current| {
                current.composite_resource_key() == image.composite_resource_key()
            })
        {
            return false;
        }
        self.scene_by_surface.insert(surface_key, Arc::new(image));
        true
    }

    pub(crate) fn for_surface(
        &self,
        surface_key: &str,
        pane_kind: &str,
    ) -> Option<&HostViewportImageData> {
        if pane_kind != "Scene" || surface_key.is_empty() {
            self.for_pane(pane_kind)
        } else if let Some(simulate) = self.simulate() {
            Some(simulate)
        } else {
            self.scene_by_surface
                .get(surface_key)
                .map(Arc::as_ref)
                .or_else(|| self.scene())
        }
    }

    pub(crate) fn replace_game(&mut self, image: HostViewportImageData) -> bool {
        Self::replace(&mut self.game, image)
    }

    pub(crate) fn replace_simulate(&mut self, image: HostViewportImageData) -> bool {
        Self::replace(&mut self.simulate, image)
    }

    pub(crate) fn clear_game(&mut self) -> bool {
        self.game.take().is_some()
    }

    pub(crate) fn clear_simulate(&mut self) -> bool {
        self.simulate.take().is_some()
    }

    fn replace(
        slot: &mut Option<Arc<HostViewportImageData>>,
        image: HostViewportImageData,
    ) -> bool {
        if slot.as_ref().is_some_and(|current| {
            current.composite_resource_key() == image.composite_resource_key()
        }) {
            return false;
        }
        *slot = Some(Arc::new(image));
        true
    }
}

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageData {
    pub(crate) resource_key: String,
    pub(crate) resource_generation: u64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Option<Arc<[u8]>>,
    pub(crate) play_frame_identity: Option<PlayPreviewFrameIdentity>,
    pub(crate) overlay: Option<Arc<HostViewportOverlayImageData>>,
}

impl HostViewportImageData {
    pub(crate) fn from_captured_frame(
        viewport: RenderViewportHandle,
        frame: CapturedFrame,
    ) -> Option<Self> {
        let width = frame.width;
        let height = frame.height;
        let generation = frame.generation;
        let image = Self {
            resource_key: viewport_image_resource_key(viewport, generation),
            resource_generation: 0,
            width,
            height,
            rgba: Some(frame.rgba.into()),
            play_frame_identity: None,
            overlay: None,
        };
        image.is_valid().then_some(image)
    }

    pub(crate) fn from_viewport_product(product: RenderViewportProduct) -> Option<Self> {
        let image = Self {
            resource_key: product.resource_key().to_string(),
            resource_generation: product.generation(),
            width: product.width(),
            height: product.height(),
            rgba: None,
            play_frame_identity: None,
            overlay: None,
        };
        (product.is_valid() && image.is_valid()).then_some(image)
    }

    pub(crate) fn from_play_preview_frame(frame: PlayPreviewFrame) -> Option<Self> {
        let identity = frame.identity().clone();
        let image = Self {
            resource_key: identity.resource_scope("play-viewport"),
            resource_generation: 0,
            width: frame.width(),
            height: frame.height(),
            rgba: Some(Arc::clone(frame.rgba())),
            play_frame_identity: Some(identity),
            overlay: None,
        };
        image.is_valid().then_some(image)
    }

    pub(crate) fn rgba(&self) -> Option<&Arc<[u8]>> {
        self.rgba.as_ref()
    }

    pub(crate) fn play_frame_identity(&self) -> Option<&PlayPreviewFrameIdentity> {
        self.play_frame_identity.as_ref()
    }

    pub(crate) fn overlay(&self) -> Option<&HostViewportOverlayImageData> {
        self.overlay.as_deref()
    }

    pub(crate) fn with_overlay(
        mut self,
        overlay: Option<HostViewportOverlayImageData>,
    ) -> Option<Self> {
        if overlay
            .as_ref()
            .is_some_and(|overlay| !overlay.is_valid_for(self.width, self.height))
        {
            return None;
        }
        self.overlay = overlay.map(Arc::new);
        Some(self)
    }

    pub(crate) fn is_valid(&self) -> bool {
        // GPU texture cache entries are keyed by resource_key, so a drawable
        // viewport image must never use the empty default key.
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self.rgba.as_ref().is_none_or(|rgba| {
                self.width
                    .checked_mul(self.height)
                    .and_then(|pixels| pixels.checked_mul(4))
                    .is_some_and(|bytes| bytes as usize == rgba.len())
            })
            && self
                .overlay
                .as_ref()
                .is_none_or(|overlay| overlay.is_valid_for(self.width, self.height))
    }

    fn composite_resource_key(&self) -> (&str, Option<&str>) {
        (
            self.resource_key.as_str(),
            self.overlay
                .as_ref()
                .map(|overlay| overlay.resource_key.as_str()),
        )
    }
}

fn viewport_image_resource_key(viewport: RenderViewportHandle, generation: u64) -> String {
    format!("viewport:{}:{generation}", viewport.raw())
}

#[cfg(test)]
#[path = "tests/viewport_image.rs"]
mod tests;
