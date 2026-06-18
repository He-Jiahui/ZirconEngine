use crate::core::framework::render::{CameraRenderDescriptor, RenderViewportRect};
use crate::core::math::{Real, UVec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportRenderRegion {
    physical_position: UVec2,
    physical_size: UVec2,
    depth_min: Real,
    depth_max: Real,
}

impl ViewportRenderRegion {
    pub fn full_target(target_size: UVec2) -> Self {
        Self::from_camera(None, target_size)
    }

    pub(crate) fn from_camera(camera: Option<&CameraRenderDescriptor>, target_size: UVec2) -> Self {
        let target_size = UVec2::new(target_size.x.max(1), target_size.y.max(1));
        let rect = camera
            .and_then(|camera| camera.viewport_rect)
            .unwrap_or_else(|| RenderViewportRect::new(UVec2::ZERO, target_size))
            .clamped_to_size(target_size);
        Self {
            physical_position: rect.physical_position,
            physical_size: rect.physical_size,
            depth_min: rect.depth_min.clamp(0.0, 1.0),
            depth_max: rect.depth_max.clamp(0.0, 1.0),
        }
    }

    pub fn physical_position(self) -> UVec2 {
        self.physical_position
    }

    pub fn physical_size(self) -> UVec2 {
        self.physical_size
    }

    pub(crate) fn physical_origin(self) -> [u32; 2] {
        [self.physical_position.x, self.physical_position.y]
    }

    pub(crate) fn local_to_physical_coord(self, local_coord: UVec2) -> UVec2 {
        let max_local = UVec2::new(
            self.physical_size.x.saturating_sub(1),
            self.physical_size.y.saturating_sub(1),
        );
        UVec2::new(
            self.physical_position
                .x
                .saturating_add(local_coord.x.min(max_local.x)),
            self.physical_position
                .y
                .saturating_add(local_coord.y.min(max_local.y)),
        )
    }

    pub fn is_empty(self) -> bool {
        self.physical_size.x == 0 || self.physical_size.y == 0
    }

    pub fn apply_to_render_pass(self, pass: &mut wgpu::RenderPass<'_>) -> bool {
        if self.is_empty() {
            return false;
        }
        pass.set_viewport(
            self.physical_position.x as f32,
            self.physical_position.y as f32,
            self.physical_size.x as f32,
            self.physical_size.y as f32,
            self.depth_min,
            self.depth_max,
        );
        pass.set_scissor_rect(
            self.physical_position.x,
            self.physical_position.y,
            self.physical_size.x,
            self.physical_size.y,
        );
        true
    }
}

impl Default for ViewportRenderRegion {
    fn default() -> Self {
        Self {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(1, 1),
            depth_min: 0.0,
            depth_max: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderViewportRect, ViewportCameraSnapshot,
    };
    use crate::core::math::UVec2;

    use super::ViewportRenderRegion;

    #[test]
    fn viewport_region_defaults_to_full_target_without_camera_rect() {
        let region = ViewportRenderRegion::full_target(UVec2::new(640, 360));

        assert_eq!(region.physical_position(), UVec2::ZERO);
        assert_eq!(region.physical_size(), UVec2::new(640, 360));
        assert!(!region.is_empty());
    }

    #[test]
    fn viewport_region_clamps_camera_rect_to_target() {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        let mut viewport = RenderViewportRect::new(UVec2::new(600, 300), UVec2::new(128, 128));
        viewport.depth_min = -1.0;
        viewport.depth_max = 2.0;
        camera.viewport_rect = Some(viewport);

        let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

        assert_eq!(region.physical_position(), UVec2::new(600, 300));
        assert_eq!(region.physical_size(), UVec2::new(40, 60));
        assert_eq!(region.depth_min, 0.0);
        assert_eq!(region.depth_max, 1.0);
        assert!(!region.is_empty());
    }

    #[test]
    fn viewport_region_clamps_fully_outside_rect_to_last_in_bounds_pixel() {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        camera.viewport_rect = Some(RenderViewportRect::new(
            UVec2::new(1280, 720),
            UVec2::new(320, 180),
        ));

        let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

        assert_eq!(region.physical_position(), UVec2::new(639, 359));
        assert_eq!(region.physical_size(), UVec2::new(1, 1));
        assert!(!region.is_empty());
    }

    #[test]
    fn viewport_region_maps_local_postprocess_coords_to_physical_target_coords() {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        camera.viewport_rect = Some(RenderViewportRect::new(
            UVec2::new(320, 0),
            UVec2::new(320, 180),
        ));

        let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

        assert_eq!(region.physical_origin(), [320, 0]);
        assert_eq!(
            region.local_to_physical_coord(UVec2::new(0, 0)),
            UVec2::new(320, 0)
        );
        assert_eq!(
            region.local_to_physical_coord(UVec2::new(319, 179)),
            UVec2::new(639, 179)
        );
        assert_eq!(
            region.local_to_physical_coord(UVec2::new(999, 999)),
            UVec2::new(639, 179)
        );
    }
}
