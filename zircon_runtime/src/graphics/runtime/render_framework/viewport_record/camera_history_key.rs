//! 相机历史按目标、层级和局部视图区分，不同 overlay 或纹理目标不能共享时间序列。
use std::sync::Arc;

use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, RenderCameraTargetOrderKey, RenderLayer,
    RenderLayerSet, RenderViewportRect,
};
use crate::core::framework::scene::EntityId;

const INLINE_HISTORY_LAYER_CAPACITY: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics::runtime::render_framework) struct ViewportCameraHistoryKey {
    entity: Option<EntityId>,
    render_order: i32,
    render_type: CameraRenderType,
    target: RenderCameraTargetOrderKey,
    viewport: Option<ViewportCameraHistoryRectKey>,
    culling_layers: ViewportCameraHistoryLayerKey,
    volume_layers: ViewportCameraHistoryLayerKey,
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
            culling_layers: ViewportCameraHistoryLayerKey::from(&camera.culling_mask),
            volume_layers: ViewportCameraHistoryLayerKey::from(&camera.volume_mask),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ViewportCameraHistoryLayerKey {
    Inline {
        layers: [RenderLayer; INLINE_HISTORY_LAYER_CAPACITY],
        len: u8,
    },
    Shared(Arc<[RenderLayer]>),
}

impl From<&RenderLayerSet> for ViewportCameraHistoryLayerKey {
    fn from(value: &RenderLayerSet) -> Self {
        let mut layers = [0; INLINE_HISTORY_LAYER_CAPACITY];
        let mut len = 0;
        for layer in value.iter() {
            if len == INLINE_HISTORY_LAYER_CAPACITY {
                return Self::Shared(value.iter().collect());
            }
            layers[len] = layer;
            len += 1;
        }
        Self::Inline {
            layers,
            len: len as u8,
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
#[path = "tests/camera_history_key.rs"]
mod tests;
