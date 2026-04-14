use super::World;
use crate::components::{
    RenderCameraSnapshot, RenderDirectionalLightSnapshot, RenderExtractPacket,
    RenderGizmoSnapshot, RenderMeshSnapshot, RenderSceneSnapshot,
};

impl World {
    pub fn to_render_snapshot(&self) -> RenderSceneSnapshot {
        self.to_render_extract()
    }

    pub fn to_render_extract(&self) -> RenderExtractPacket {
        let camera_entity = self
            .cameras
            .get(&self.active_camera)
            .map(|camera| (self.active_camera, camera))
            .or_else(|| self.cameras.iter().next().map(|(id, camera)| (*id, camera)))
            .expect("world always contains a camera");
        let camera_transform = self.world_transform(camera_entity.0).unwrap_or_else(|| {
            self.find_node(camera_entity.0)
                .map(|node| node.transform)
                .unwrap_or_default()
        });
        let light = self
            .directional_lights
            .values()
            .next()
            .cloned()
            .unwrap_or_default();

        let meshes = self
            .mesh_renderers
            .iter()
            .filter(|(entity, _)| self.active.get(entity).copied().unwrap_or_default().0)
            .map(|(entity, mesh)| RenderMeshSnapshot {
                node_id: *entity,
                transform: self.world_transform(*entity).unwrap_or_default(),
                model: mesh.model,
                material: mesh.material,
                tint: mesh.tint,
                selected: self.selected_entity == Some(*entity),
            })
            .collect();

        let gizmo = self.selected_entity.and_then(|entity| {
            if self.directional_lights.contains_key(&entity) {
                None
            } else {
                Some(RenderGizmoSnapshot {
                    target_node: entity,
                    origin: self.world_transform(entity).unwrap_or_default().translation,
                })
            }
        });

        RenderExtractPacket {
            camera: RenderCameraSnapshot {
                node_id: camera_entity.0,
                transform: camera_transform,
                fov_y_radians: camera_entity.1.fov_y_radians,
                z_near: camera_entity.1.z_near,
                z_far: camera_entity.1.z_far,
            },
            meshes,
            light: RenderDirectionalLightSnapshot {
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
            },
            selected_node: self.selected_entity,
            gizmo,
            show_grid: true,
        }
    }
}
