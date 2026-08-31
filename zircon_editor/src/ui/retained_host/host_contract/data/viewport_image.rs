use std::sync::Arc;

use crate::core::play::{PlayPreviewFrame, PlayPreviewFrameIdentity};
use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};

mod overlay;

pub(crate) use overlay::HostViewportOverlayImageData;

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageSet {
    scene: Option<Arc<HostViewportImageData>>,
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
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::math::{UVec2, Vec2, Vec4};

    use crate::scene::viewport::HandleScreenLine;

    use super::*;

    #[test]
    fn viewport_image_resource_key_tracks_viewport_generation() {
        let red = viewport_image(3, 7, &[255, 0, 0, 255]);
        let blue = viewport_image(3, 8, &[0, 0, 255, 255]);

        assert_ne!(red.resource_key, blue.resource_key);
        assert_eq!(red.resource_key, "viewport:3:7");
        assert_eq!(blue.resource_key, "viewport:3:8");
    }

    #[test]
    fn viewport_image_requires_resource_key_to_be_valid() {
        let image = HostViewportImageData {
            resource_key: String::new(),
            width: 1,
            height: 1,
            rgba: Some(vec![255, 255, 255, 255].into()),
            play_frame_identity: None,
            overlay: None,
        };

        assert!(!image.is_valid());
    }

    #[test]
    fn viewport_image_clones_share_the_captured_rgba_payload() {
        let image = viewport_image(3, 7, &[255, 0, 0, 255]);
        let cloned = image.clone();

        assert!(Arc::ptr_eq(
            image.rgba.as_ref().expect("capture payload"),
            cloned.rgba.as_ref().expect("capture payload"),
        ));
    }

    #[test]
    fn viewport_product_keeps_the_gpu_resource_key_without_cpu_pixels() {
        let product = RenderViewportProduct::new(RenderViewportHandle::new(3), 640, 360, 11);

        let image = HostViewportImageData::from_viewport_product(product)
            .expect("valid GPU product should transfer into host data");

        assert_eq!(image.resource_key, "viewport:3:11");
        assert!(image.rgba.is_none());
        assert!(image.is_valid());
    }

    #[test]
    fn play_preview_image_retains_structured_frame_and_gateway_identity() {
        let frame = PlayPreviewFrame::for_test(
            crate::core::play::PlayInstanceId::for_test(7),
            zircon_runtime_interface::GatewaySessionIdentity::new(
                11,
                zircon_runtime_interface::ZrRuntimeSessionHandle::new(13),
                17,
                None,
            )
            .with_gateway_generation(19)
            .with_play_instance(Some(7)),
            2,
            1,
            23,
            vec![0; 8],
        );

        let image = HostViewportImageData::from_play_preview_frame(frame)
            .expect("valid play preview image");
        let identity = image
            .play_frame_identity()
            .expect("play image must retain its structured identity");

        assert_eq!(identity.instance().raw(), 7);
        assert_eq!(identity.gateway().gateway_generation(), 19);
        assert_eq!(identity.generation(), 23);
        assert_eq!(identity.size(), (2, 1));
    }

    #[test]
    fn play_preview_resource_key_separates_replaced_gateway_generations() {
        let gateway = zircon_runtime_interface::GatewaySessionIdentity::new(
            11,
            zircon_runtime_interface::ZrRuntimeSessionHandle::new(13),
            17,
            None,
        )
        .with_play_instance(Some(7));
        let first = PlayPreviewFrame::for_test(
            crate::core::play::PlayInstanceId::for_test(7),
            gateway.clone().with_gateway_generation(19),
            1,
            1,
            23,
            vec![0; 4],
        );
        let replacement = PlayPreviewFrame::for_test(
            crate::core::play::PlayInstanceId::for_test(7),
            gateway.with_gateway_generation(29),
            1,
            1,
            23,
            vec![0; 4],
        );

        let first = HostViewportImageData::from_play_preview_frame(first).unwrap();
        let replacement = HostViewportImageData::from_play_preview_frame(replacement).unwrap();

        assert_ne!(first.resource_key, replacement.resource_key);
        assert_eq!(
            first.resource_key,
            "play-viewport:7:11:13:17:19:some:7:1:1:23:none"
        );
        assert_eq!(
            replacement.resource_key,
            "play-viewport:7:11:13:17:29:some:7:1:1:23:none"
        );
    }

    #[test]
    fn image_set_routes_scene_and_game_without_cross_pane_fallback() {
        let mut images = HostViewportImageSet::default();
        assert!(images.replace_scene(viewport_image(3, 7, &[255, 0, 0, 255])));

        assert!(images.for_pane("Scene").is_some());
        assert!(images.for_pane("Game").is_none());
        assert!(images.for_pane("Inspector").is_none());
    }

    #[test]
    fn simulate_image_temporarily_overrides_scene_without_destroying_authoring_image() {
        let mut images = HostViewportImageSet::default();
        let authoring = viewport_image(3, 7, &[255, 0, 0, 255]);
        let simulate = viewport_image(9, 11, &[0, 0, 255, 255]);
        assert!(images.replace_scene(authoring.clone()));
        assert!(images.replace_simulate(simulate.clone()));

        assert_eq!(
            images.for_pane("Scene").unwrap().resource_key,
            simulate.resource_key
        );
        assert!(images.clear_simulate());
        assert_eq!(
            images.for_pane("Scene").unwrap().resource_key,
            authoring.resource_key
        );
    }

    #[test]
    fn screen_line_overlay_raster_is_tight_transparent_and_attachable() {
        let overlay = HostViewportOverlayImageData::from_screen_lines(
            "play-gizmo:test",
            UVec2::new(640, 360),
            &[HandleScreenLine::new(
                Vec2::new(100.0, 80.0),
                Vec2::new(120.0, 80.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                2.0,
                Some(crate::scene::viewport::GizmoAxis::X),
            )],
        )
        .expect("a visible line should produce a compact overlay");

        assert!(overlay.width < 640);
        assert!(overlay.height < 360);
        assert!(overlay.rgba.chunks_exact(4).any(|pixel| pixel[3] == 0));
        assert!(overlay.rgba.chunks_exact(4).any(|pixel| pixel[3] > 0));

        let base = HostViewportImageData {
            resource_key: "play:test".to_string(),
            width: 640,
            height: 360,
            rgba: Some(vec![0; 640 * 360 * 4].into()),
            play_frame_identity: None,
            overlay: None,
        };
        let composed = base
            .with_overlay(Some(overlay))
            .expect("an in-bounds overlay should attach atomically");
        assert!(composed.overlay().is_some());
    }

    fn viewport_image(viewport: u64, generation: u64, rgba: &[u8]) -> HostViewportImageData {
        HostViewportImageData::from_captured_frame(
            RenderViewportHandle::new(viewport),
            CapturedFrame::new(1, 1, rgba.to_vec(), generation),
        )
        .expect("valid capture should transfer into host data")
    }
}
