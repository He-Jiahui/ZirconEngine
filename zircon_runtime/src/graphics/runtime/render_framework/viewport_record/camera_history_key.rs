use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, RenderCameraTargetOrderKey, RenderViewportRect,
};
use crate::core::framework::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics::runtime::render_framework) struct ViewportCameraHistoryKey {
    entity: Option<EntityId>,
    render_order: i32,
    render_type: CameraRenderType,
    target: RenderCameraTargetOrderKey,
    viewport: Option<ViewportCameraHistoryRectKey>,
}

impl ViewportCameraHistoryKey {
    pub(in crate::graphics::runtime::render_framework) fn from_camera(
        camera: &CameraRenderDescriptor,
    ) -> Self {
        Self {
            entity: camera.entity,
            render_order: camera.render_order,
            render_type: camera.render_type,
            target: camera.target_key(),
            viewport: camera.viewport_rect.map(ViewportCameraHistoryRectKey::from),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ViewportCameraHistoryRectKey {
    position_x: u32,
    position_y: u32,
    width: u32,
    height: u32,
    depth_min_bits: u32,
    depth_max_bits: u32,
}

impl From<RenderViewportRect> for ViewportCameraHistoryRectKey {
    fn from(value: RenderViewportRect) -> Self {
        Self {
            position_x: value.physical_position.x,
            position_y: value.physical_position.y,
            width: value.physical_size.x,
            height: value.physical_size.y,
            depth_min_bits: value.depth_min.to_bits(),
            depth_max_bits: value.depth_max.to_bits(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, CameraRenderType, RenderCameraTarget, RenderViewportRect,
        ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::EntityId;
    use crate::core::math::UVec2;

    use super::ViewportCameraHistoryKey;

    #[test]
    fn camera_history_key_distinguishes_same_entity_viewport_regions() {
        let left =
            descriptor(7).with_viewport(RenderViewportRect::new(UVec2::ZERO, UVec2::new(32, 48)));
        let right = descriptor(7).with_viewport(RenderViewportRect::new(
            UVec2::new(32, 0),
            UVec2::new(32, 48),
        ));

        assert_ne!(
            ViewportCameraHistoryKey::from_camera(&left),
            ViewportCameraHistoryKey::from_camera(&right)
        );
    }

    #[test]
    fn camera_history_key_distinguishes_base_and_overlay_slots() {
        let base = descriptor(11);
        let mut overlay = descriptor(11);
        overlay.render_type = CameraRenderType::Overlay;

        assert_ne!(
            ViewportCameraHistoryKey::from_camera(&base),
            ViewportCameraHistoryKey::from_camera(&overlay)
        );
    }

    fn descriptor(entity: EntityId) -> CameraRenderDescriptor {
        CameraRenderDescriptor {
            entity: Some(entity),
            target: RenderCameraTarget::PrimarySurface,
            camera: ViewportCameraSnapshot::default(),
            ..CameraRenderDescriptor::from_camera_payload(
                Some(entity),
                ViewportCameraSnapshot::default(),
            )
        }
    }

    trait CameraDescriptorTestExt {
        fn with_viewport(self, viewport: RenderViewportRect) -> Self;
    }

    impl CameraDescriptorTestExt for CameraRenderDescriptor {
        fn with_viewport(mut self, viewport: RenderViewportRect) -> Self {
            self.viewport_rect = Some(viewport);
            self
        }
    }
}
