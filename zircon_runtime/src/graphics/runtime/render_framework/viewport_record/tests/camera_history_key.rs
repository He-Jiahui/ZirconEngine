use std::sync::Arc;

use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, RenderCameraTarget, RenderLayerSet,
    RenderViewportRect, ViewportCameraSnapshot,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::UVec2;

use super::{ViewportCameraHistoryKey, ViewportCameraHistoryLayerKey};

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

#[test]
fn camera_history_key_distinguishes_culling_layers_without_scene_schema_v1_loss() {
    let mut wide_layer = descriptor(13);
    wide_layer.culling_mask = RenderLayerSet::layer(40);
    let mut no_layers = descriptor(13);
    no_layers.culling_mask = RenderLayerSet::none();

    assert_eq!(wide_layer.culling_mask.to_scene_schema_v1_mask_lossy(), 0);
    assert_ne!(
        ViewportCameraHistoryKey::from_camera(&wide_layer),
        ViewportCameraHistoryKey::from_camera(&no_layers)
    );
}

#[test]
fn camera_history_key_distinguishes_volume_layers_without_scene_schema_v1_loss() {
    let mut wide_layer = descriptor(17);
    wide_layer.volume_mask = RenderLayerSet::layer(41);
    let mut no_layers = descriptor(17);
    no_layers.volume_mask = RenderLayerSet::none();

    assert_eq!(wide_layer.volume_mask.to_scene_schema_v1_mask_lossy(), 0);
    assert_ne!(
        ViewportCameraHistoryKey::from_camera(&wide_layer),
        ViewportCameraHistoryKey::from_camera(&no_layers)
    );
}

#[test]
fn camera_history_key_common_layers_are_inline() {
    let mut camera = descriptor(19);
    camera.culling_mask = RenderLayerSet::from_layers([0, 40, 80]);
    let key = ViewportCameraHistoryKey::from_camera(&camera);

    assert!(matches!(
        key.culling_layers,
        ViewportCameraHistoryLayerKey::Inline {
            layers: [0, 40, 80, 0],
            len: 3,
        }
    ));
}

#[test]
fn camera_history_key_wide_clones_share_layer_storage() {
    let mut camera = descriptor(23);
    camera.culling_mask = RenderLayerSet::from_layers([0, 40, 80, 120, 160]);
    camera.volume_mask = RenderLayerSet::from_layers([1, 41, 81, 121, 161]);
    let key = ViewportCameraHistoryKey::from_camera(&camera);
    let cloned = key.clone();

    for (source, copy) in [
        (&key.culling_layers, &cloned.culling_layers),
        (&key.volume_layers, &cloned.volume_layers),
    ] {
        let (
            ViewportCameraHistoryLayerKey::Shared(source),
            ViewportCameraHistoryLayerKey::Shared(copy),
        ) = (source, copy)
        else {
            panic!("more than four layers must use shared fallback storage");
        };
        assert!(Arc::ptr_eq(source, copy));
    }
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
