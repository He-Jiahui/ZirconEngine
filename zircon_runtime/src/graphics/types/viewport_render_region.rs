//! 渲染区域同时保存物理目标与本地渲染坐标，供分屏、降采样及历史拷贝稳定映射。
use crate::core::framework::render::{
    CameraRenderDescriptor, RenderViewFamilyTarget, RenderViewportRect,
};
use crate::core::math::{Real, UVec2};

/// 同时表达目标物理区域与相机局部区域，供呈现和历史读写共享。
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

    pub(crate) fn from_view_family_target(target: RenderViewFamilyTarget) -> Self {
        let viewport = target.viewport();
        Self {
            physical_position: viewport.physical_position,
            physical_size: viewport.physical_size,
            local_size: viewport.physical_size,
            depth_min: viewport.depth_min,
            depth_max: viewport.depth_max,
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
#[path = "tests/viewport_render_region.rs"]
mod tests;
