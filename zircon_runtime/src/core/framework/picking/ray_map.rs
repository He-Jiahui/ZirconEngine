use std::collections::HashMap;

use crate::core::framework::render::{RenderViewportHandle, ViewportCameraSnapshot};
use crate::core::framework::scene::EntityId;
use crate::core::math::UVec2;

use super::{ray_from_viewport_point, PointerId, PointerLocation, PointerRay};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RayId {
    pub camera: EntityId,
    pub pointer: PointerId,
    pub viewport: RenderViewportHandle,
}

impl RayId {
    pub const fn new(camera: EntityId, pointer: PointerId, viewport: RenderViewportHandle) -> Self {
        Self {
            camera,
            pointer,
            viewport,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraRaySource {
    pub camera: EntityId,
    pub viewport: RenderViewportHandle,
    pub viewport_size: UVec2,
    pub snapshot: ViewportCameraSnapshot,
    pub active: bool,
}

impl CameraRaySource {
    pub fn new(
        camera: EntityId,
        viewport: RenderViewportHandle,
        viewport_size: UVec2,
        snapshot: ViewportCameraSnapshot,
    ) -> Self {
        Self {
            camera,
            viewport,
            viewport_size,
            snapshot,
            active: true,
        }
    }

    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RayMap {
    map: HashMap<RayId, PointerRay>,
}

impl RayMap {
    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn insert(&mut self, id: RayId, ray: PointerRay) -> Option<PointerRay> {
        self.map.insert(id, ray)
    }

    pub fn get(&self, id: &RayId) -> Option<&PointerRay> {
        self.map.get(id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RayId, &PointerRay)> {
        self.map.iter()
    }

    pub fn rebuild(&mut self, pointers: &[PointerLocation], cameras: &[CameraRaySource]) {
        self.map.clear();
        for camera in cameras {
            if !camera.active {
                continue;
            }
            for pointer in pointers {
                if pointer.viewport != camera.viewport
                    || !pointer.is_inside_viewport(camera.viewport_size)
                {
                    continue;
                }
                if let Some(ray) = ray_from_viewport_point(
                    &camera.snapshot,
                    camera.viewport_size,
                    pointer.position,
                ) {
                    self.map.insert(
                        RayId::new(camera.camera, pointer.pointer, camera.viewport),
                        ray,
                    );
                }
            }
        }
    }
}
