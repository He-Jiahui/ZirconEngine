use super::World;
use crate::scene::components::{
    ActiveSelf, Hierarchy, LocalTransform, Name, NodeRecord, RenderLayerMask,
};
use crate::scene::EntityId;

impl World {
    pub fn node_record(&self, entity: EntityId) -> Option<NodeRecord> {
        Some(NodeRecord {
            id: entity,
            name: self.names.get(&entity)?.0.clone(),
            kind: self.node_kind(entity)?,
            parent: self
                .hierarchy
                .get(&entity)
                .and_then(|hierarchy| hierarchy.parent),
            transform: self
                .local_transforms
                .get(&entity)
                .copied()
                .unwrap_or_default()
                .transform,
            camera: self.cameras.get(&entity).cloned(),
            mesh: self.mesh_renderers.get(&entity).cloned(),
            directional_light: self.directional_lights.get(&entity).cloned(),
            point_light: self.point_lights.get(&entity).cloned(),
            spot_light: self.spot_lights.get(&entity).cloned(),
            active: self.active_self.get(&entity).copied().unwrap_or_default().0,
            render_layer_mask: self
                .render_layer_masks
                .get(&entity)
                .copied()
                .unwrap_or_default()
                .0,
            mobility: self.mobility.get(&entity).copied().unwrap_or_default(),
            rigid_body: self.rigid_bodies.get(&entity).cloned(),
            collider: self.colliders.get(&entity).cloned(),
            joint: self.joints.get(&entity).cloned(),
            animation_skeleton: self.animation_skeletons.get(&entity).cloned(),
            animation_player: self.animation_players.get(&entity).cloned(),
            animation_sequence_player: self.animation_sequence_players.get(&entity).cloned(),
            animation_graph_player: self.animation_graph_players.get(&entity).cloned(),
            animation_state_machine_player: self
                .animation_state_machine_players
                .get(&entity)
                .cloned(),
        })
    }

    pub fn insert_node_record(&mut self, record: NodeRecord) -> Result<(), String> {
        if self.entities.contains(&record.id) {
            return Err(format!("entity {} already exists", record.id));
        }

        self.entities.push(record.id);
        self.kinds.insert(record.id, record.kind);
        self.names.insert(record.id, Name(record.name));
        self.hierarchy.insert(
            record.id,
            Hierarchy {
                parent: record.parent,
            },
        );
        self.local_transforms.insert(
            record.id,
            LocalTransform {
                transform: record.transform,
            },
        );
        self.active_self
            .insert(record.id, ActiveSelf(record.active));
        self.render_layer_masks
            .insert(record.id, RenderLayerMask(record.render_layer_mask));
        self.mobility.insert(record.id, record.mobility);

        if let Some(camera) = record.camera {
            self.cameras.insert(record.id, camera);
            if self.active_camera == 0 || !self.cameras.contains_key(&self.active_camera) {
                self.active_camera = record.id;
            }
        }
        if let Some(mesh) = record.mesh {
            self.mesh_renderers.insert(record.id, mesh);
        }
        if let Some(directional_light) = record.directional_light {
            self.directional_lights.insert(record.id, directional_light);
        }
        if let Some(point_light) = record.point_light {
            self.point_lights.insert(record.id, point_light);
        }
        if let Some(spot_light) = record.spot_light {
            self.spot_lights.insert(record.id, spot_light);
        }
        if let Some(rigid_body) = record.rigid_body {
            self.rigid_bodies.insert(record.id, rigid_body);
        }
        if let Some(collider) = record.collider {
            self.colliders.insert(record.id, collider);
        }
        if let Some(joint) = record.joint {
            self.joints.insert(record.id, joint);
        }
        if let Some(animation_skeleton) = record.animation_skeleton {
            self.animation_skeletons
                .insert(record.id, animation_skeleton);
        }
        if let Some(animation_player) = record.animation_player {
            self.animation_players.insert(record.id, animation_player);
        }
        if let Some(animation_sequence_player) = record.animation_sequence_player {
            self.animation_sequence_players
                .insert(record.id, animation_sequence_player);
        }
        if let Some(animation_graph_player) = record.animation_graph_player {
            self.animation_graph_players
                .insert(record.id, animation_graph_player);
        }
        if let Some(animation_state_machine_player) = record.animation_state_machine_player {
            self.animation_state_machine_players
                .insert(record.id, animation_state_machine_player);
        }

        self.next_id = self.next_id.max(record.id + 1);
        self.validate_mobility_change(record.id, record.mobility)?;
        self.rebuild_derived_state();
        Ok(())
    }

    pub fn insert_node_records(&mut self, records: &[NodeRecord]) -> Result<(), String> {
        for record in records {
            self.insert_node_record(record.clone())?;
        }
        Ok(())
    }

    pub fn rename_node(
        &mut self,
        entity: EntityId,
        name: impl Into<String>,
    ) -> Result<bool, String> {
        let name = name.into();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("node name cannot be empty".to_string());
        }
        let Some(current) = self.names.get_mut(&entity) else {
            return Err(format!("cannot rename missing node {entity}"));
        };
        if current.0 == trimmed {
            return Ok(false);
        }
        current.0 = trimmed.to_string();
        self.refresh_node_cache();
        Ok(true)
    }
}
