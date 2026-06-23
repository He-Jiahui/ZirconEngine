use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, RenderCameraClear, RenderCameraTarget,
    RenderLayerSet, RenderQualityProfile, RenderViewportRect, ViewportCameraSnapshot,
};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

pub(super) fn texture_camera_descriptor(
    entity: u64,
    render_order: i32,
    texture_id: ResourceId,
    render_type: CameraRenderType,
    clear: RenderCameraClear,
    clear_depth: bool,
    layers: RenderLayerSet,
    camera: ViewportCameraSnapshot,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order,
        render_type,
        target: RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(texture_id)),
        clear,
        clear_depth,
        culling_mask: layers.clone(),
        volume_mask: layers,
        ..CameraRenderDescriptor::from_camera_payload(Some(entity), camera)
    }
}

pub(super) fn primary_surface_camera_descriptor(
    entity: u64,
    render_order: i32,
    clear: RenderCameraClear,
    layers: RenderLayerSet,
    camera: ViewportCameraSnapshot,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order,
        render_type: CameraRenderType::Base,
        target: RenderCameraTarget::default(),
        clear,
        culling_mask: layers.clone(),
        volume_mask: layers,
        ..CameraRenderDescriptor::from_camera_payload(Some(entity), camera)
    }
}

pub(super) fn primary_surface_stack_camera_descriptor(
    entity: u64,
    render_type: CameraRenderType,
    clear: RenderCameraClear,
    clear_depth: bool,
    layers: RenderLayerSet,
    camera: ViewportCameraSnapshot,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_type,
        target: RenderCameraTarget::default(),
        clear,
        clear_depth,
        culling_mask: layers.clone(),
        volume_mask: layers,
        ..CameraRenderDescriptor::from_camera_payload(Some(entity), camera)
    }
}

pub(super) fn camera_target_product_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("multi-custom-target-stacks-feed-primary")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

pub(super) trait CameraDescriptorTestExt {
    fn with_stack(self, stack: impl IntoIterator<Item = u64>) -> Self;
    fn with_viewport(self, viewport: RenderViewportRect) -> Self;
}

impl CameraDescriptorTestExt for CameraRenderDescriptor {
    fn with_stack(mut self, stack: impl IntoIterator<Item = u64>) -> Self {
        self.stack = stack.into_iter().collect();
        self
    }

    fn with_viewport(mut self, viewport: RenderViewportRect) -> Self {
        self.viewport_rect = Some(viewport);
        self
    }
}
