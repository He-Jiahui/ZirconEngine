use crate::core::framework::render::{CameraRenderDescriptor, RenderViewportRect};
use crate::core::math::{Real, UVec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportRenderRegion {
    physical_position: UVec2,
    physical_size: UVec2,
    local_size: UVec2,
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
            local_size: rect.physical_size,
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

    pub(crate) fn local_position(self) -> UVec2 {
        UVec2::ZERO
    }

    pub(crate) fn local_size(self) -> UVec2 {
        self.local_size
    }

    pub(crate) fn physical_origin(self) -> [u32; 2] {
        [self.physical_position.x, self.physical_position.y]
    }

    pub(crate) fn local_to_physical_coord(self, local_coord: UVec2) -> UVec2 {
        let scale_axis = |coord: u32, local: u32, physical: u32| -> u32 {
            if local <= 1 {
                0
            } else {
                ((u64::from(coord.min(local - 1)) * u64::from(physical.saturating_sub(1)))
                    / u64::from(local - 1)) as u32
            }
        };
        let local_size = UVec2::new(self.local_size.x.max(1), self.local_size.y.max(1));
        UVec2::new(
            self.physical_position.x.saturating_add(scale_axis(
                local_coord.x,
                local_size.x,
                self.physical_size.x,
            )),
            self.physical_position.y.saturating_add(scale_axis(
                local_coord.y,
                local_size.y,
                self.physical_size.y,
            )),
        )
    }

    pub(crate) fn with_local_size(self, local_size: UVec2) -> Self {
        Self {
            local_size: UVec2::new(local_size.x.max(1), local_size.y.max(1)),
            ..self
        }
    }

    pub(crate) fn local_render_region(self) -> Self {
        Self {
            physical_position: UVec2::ZERO,
            physical_size: self.local_size,
            local_size: self.local_size,
            ..self
        }
    }

    pub fn is_empty(self) -> bool {
        self.physical_size.x == 0 || self.physical_size.y == 0
    }

    pub fn apply_to_render_pass(self, pass: &mut wgpu::RenderPass<'_>) -> bool {
        self.apply_physical_to_render_pass(pass)
    }

    pub fn apply_physical_to_render_pass(self, pass: &mut wgpu::RenderPass<'_>) -> bool {
        if self.is_empty() {
            return false;
        }
        set_render_pass_region(
            pass,
            self.physical_position,
            self.physical_size,
            self.depth_min,
            self.depth_max,
        );
        true
    }

    pub fn apply_local_to_render_pass(self, pass: &mut wgpu::RenderPass<'_>) -> bool {
        if self.is_empty() {
            return false;
        }
        set_render_pass_region(
            pass,
            self.local_position(),
            self.local_size(),
            self.depth_min,
            self.depth_max,
        );
        true
    }
}

fn set_render_pass_region(
    pass: &mut wgpu::RenderPass<'_>,
    position: UVec2,
    size: UVec2,
    depth_min: Real,
    depth_max: Real,
) {
    pass.set_viewport(
        position.x as f32,
        position.y as f32,
        size.x as f32,
        size.y as f32,
        depth_min,
        depth_max,
    );
    pass.set_scissor_rect(position.x, position.y, size.x, size.y);
}

impl Default for ViewportRenderRegion {
    fn default() -> Self {
        Self {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(1, 1),
            local_size: UVec2::new(1, 1),
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

    #[test]
    fn viewport_region_reports_local_rect_for_graph_owned_targets() {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        camera.viewport_rect = Some(RenderViewportRect::new(
            UVec2::new(320, 0),
            UVec2::new(320, 180),
        ));

        let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

        assert_eq!(region.local_position(), UVec2::ZERO);
        assert_eq!(region.local_size(), UVec2::new(320, 180));
        assert_eq!(region.physical_position(), UVec2::new(320, 0));
    }

    #[test]
    fn viewport_region_preserves_output_rect_when_local_render_size_changes() {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        camera.viewport_rect = Some(RenderViewportRect::new(
            UVec2::new(80, 40),
            UVec2::new(160, 120),
        ));

        let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(320, 240))
            .with_local_size(UVec2::new(80, 60));

        assert_eq!(region.physical_position(), UVec2::new(80, 40));
        assert_eq!(region.physical_size(), UVec2::new(160, 120));
        assert_eq!(region.local_position(), UVec2::ZERO);
        assert_eq!(region.local_size(), UVec2::new(80, 60));
        assert_eq!(
            region.local_to_physical_coord(UVec2::new(79, 59)),
            UVec2::new(239, 159)
        );
    }

    #[test]
    fn viewport_region_derives_origin_zero_region_for_graph_owned_targets() {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        camera.viewport_rect = Some(RenderViewportRect::new(
            UVec2::new(80, 40),
            UVec2::new(160, 120),
        ));

        let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(320, 240))
            .with_local_size(UVec2::new(80, 60))
            .local_render_region();

        assert_eq!(region.physical_position(), UVec2::ZERO);
        assert_eq!(region.physical_size(), UVec2::new(80, 60));
        assert_eq!(region.local_size(), UVec2::new(80, 60));
    }
}
