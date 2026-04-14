use super::World;
use crate::components::{Active, Hierarchy, LocalTransform, Name, NodeRecord};
use crate::EntityId;

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
            active: self.active.get(&entity).copied().unwrap_or_default().0,
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
        self.active.insert(record.id, Active(record.active));

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

        self.next_id = self.next_id.max(record.id + 1);
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
