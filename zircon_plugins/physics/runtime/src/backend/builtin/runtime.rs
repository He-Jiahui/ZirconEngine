use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsColliderShape, PhysicsQueryFilter, PhysicsRayCastHit,
        PhysicsRayCastQuery, PhysicsSettings, PhysicsShapeCastHit, PhysicsShapeCastQuery,
        PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::{Real, Vec3};

use super::query_contact::{compute_contact_events, ray_cast_collider, shape_overlap_query};
use super::step::integrate_body_sync_state;
use super::trigger::{compute_trigger_events, PhysicsTriggerPairMap};
use crate::backend::handle_pool::HandlePool;
use crate::backend::validation::{body_desc_is_valid, material_is_valid, shape_is_valid};
use crate::backend::{
    BodyCommand, BodyDesc, BodyHandle, ConstraintDesc, ConstraintHandle, PhysicsBackend,
    PhysicsBackendError, PhysicsBackendObjectKind, PhysicsEventBuffer, ShapeHandle,
};

const BACKEND_NAME: &str = "builtin";

pub struct BuiltinPhysicsBackend {
    settings: PhysicsSettings,
    shapes: HandlePool<ShapeRecord, ShapeHandle>,
    bodies: HandlePool<BodyRecord, BodyHandle>,
    constraints: HandlePool<ConstraintDesc, ConstraintHandle>,
    trigger_pairs: HashMap<WorldHandle, PhysicsTriggerPairMap>,
    events: PhysicsEventBuffer,
}

#[derive(Clone)]
struct ShapeRecord {
    shape: PhysicsColliderShape,
    material: PhysicsMaterialMetadata,
}

#[derive(Clone)]
struct BodyRecord {
    desc: BodyDesc,
    accumulated_force: Vec3,
    active: bool,
}

impl BuiltinPhysicsBackend {
    pub fn new(settings: PhysicsSettings) -> Self {
        Self {
            settings,
            shapes: HandlePool::default(),
            bodies: HandlePool::default(),
            constraints: HandlePool::default(),
            trigger_pairs: HashMap::new(),
            events: PhysicsEventBuffer::default(),
        }
    }

    fn world_sync(&self, world: WorldHandle) -> PhysicsWorldSyncState {
        let records = self
            .bodies
            .iter()
            .filter(|(_, record)| record.desc.world == world)
            .map(|(_, record)| record);
        let mut bodies = Vec::new();
        let mut colliders = Vec::new();
        for record in records {
            bodies.push(record.desc.body.clone());
            colliders.push(record.desc.collider.clone());
        }
        PhysicsWorldSyncState {
            world,
            bodies,
            colliders,
            joints: Vec::new(),
            materials: Vec::new(),
        }
    }

    fn refresh_events(&mut self) {
        let worlds = self
            .bodies
            .iter()
            .map(|(_, record)| record.desc.world)
            .collect::<HashSet<_>>();
        for world in worlds {
            let sync = self.world_sync(world);
            self.events
                .contacts
                .extend(compute_contact_events(&sync, &self.settings));
            let previous = self.trigger_pairs.get(&world).cloned().unwrap_or_default();
            let (current, events) = compute_trigger_events(&sync, &self.settings, &previous);
            self.trigger_pairs.insert(world, current);
            self.events.triggers.extend(events);
        }
    }

    fn invalid_handle(kind: PhysicsBackendObjectKind, raw: u64) -> PhysicsBackendError {
        PhysicsBackendError::InvalidHandle { kind, raw }
    }
}

impl PhysicsBackend for BuiltinPhysicsBackend {
    fn name(&self) -> &'static str {
        BACKEND_NAME
    }

    fn create_shape(
        &mut self,
        shape: &PhysicsColliderShape,
        material: &PhysicsMaterialMetadata,
    ) -> Result<ShapeHandle, PhysicsBackendError> {
        if !shape_is_valid(shape) || !material_is_valid(material) {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Shape,
                detail: "shape dimensions and material values must be finite and valid".to_string(),
            });
        }
        self.shapes
            .insert(ShapeRecord {
                shape: shape.clone(),
                material: material.clone(),
            })
            .ok_or(PhysicsBackendError::CapacityExhausted {
                kind: PhysicsBackendObjectKind::Shape,
            })
    }

    fn create_body(&mut self, desc: &BodyDesc) -> Result<BodyHandle, PhysicsBackendError> {
        let shape = self.shapes.get(desc.shape).ok_or_else(|| {
            Self::invalid_handle(PhysicsBackendObjectKind::Shape, desc.shape.raw())
        })?;
        if shape.shape != desc.collider.shape || !body_desc_is_valid(desc) {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Body,
                detail: "body state must be finite and reference the created collider shape"
                    .to_string(),
            });
        }
        let mut desc = desc.clone();
        desc.collider.material_override = Some(shape.material.clone());
        self.bodies
            .insert(BodyRecord {
                desc,
                accumulated_force: Vec3::ZERO,
                active: true,
            })
            .ok_or(PhysicsBackendError::CapacityExhausted {
                kind: PhysicsBackendObjectKind::Body,
            })
    }

    fn create_constraint(
        &mut self,
        desc: &ConstraintDesc,
    ) -> Result<ConstraintHandle, PhysicsBackendError> {
        for body in desc.handles() {
            if self.bodies.get(body).is_none() {
                return Err(Self::invalid_handle(
                    PhysicsBackendObjectKind::Body,
                    body.raw(),
                ));
            }
        }
        Err(PhysicsBackendError::Unsupported {
            backend: BACKEND_NAME,
            operation: "create_constraint",
            detail: "constraint solving starts in Plugins 03 M3",
        })
    }

    fn destroy_shape(&mut self, shape: ShapeHandle) -> Result<(), PhysicsBackendError> {
        if self.bodies.iter().any(|(_, body)| body.desc.shape == shape) {
            return Err(PhysicsBackendError::ObjectInUse {
                kind: PhysicsBackendObjectKind::Shape,
                raw: shape.raw(),
            });
        }
        self.shapes
            .remove(shape)
            .map(drop)
            .ok_or_else(|| Self::invalid_handle(PhysicsBackendObjectKind::Shape, shape.raw()))
    }

    fn destroy_body(&mut self, body: BodyHandle) -> Result<(), PhysicsBackendError> {
        self.bodies
            .remove(body)
            .map(drop)
            .ok_or_else(|| Self::invalid_handle(PhysicsBackendObjectKind::Body, body.raw()))
    }

    fn destroy_constraint(
        &mut self,
        constraint: ConstraintHandle,
    ) -> Result<(), PhysicsBackendError> {
        self.constraints
            .remove(constraint)
            .map(drop)
            .ok_or_else(|| {
                Self::invalid_handle(PhysicsBackendObjectKind::Constraint, constraint.raw())
            })
    }

    fn apply_commands(&mut self, commands: &[BodyCommand]) -> Result<(), PhysicsBackendError> {
        for command in commands {
            let body = command.body();
            if self.bodies.get(body).is_none() {
                return Err(Self::invalid_handle(
                    PhysicsBackendObjectKind::Body,
                    body.raw(),
                ));
            }
        }
        for command in commands {
            let body = command.body();
            let Some(record) = self.bodies.get_mut(body) else {
                return Err(Self::invalid_handle(
                    PhysicsBackendObjectKind::Body,
                    body.raw(),
                ));
            };
            match *command {
                BodyCommand::SetLinearVelocity { velocity, .. } => {
                    record.desc.body.linear_velocity = velocity;
                }
                BodyCommand::SetAngularVelocity { velocity, .. } => {
                    record.desc.body.angular_velocity = velocity;
                }
                BodyCommand::ApplyForce { force, .. } => {
                    record.accumulated_force += Vec3::from_array(force);
                }
                BodyCommand::ApplyImpulse { impulse, .. } => {
                    record.desc.body.linear_velocity =
                        (Vec3::from_array(record.desc.body.linear_velocity)
                            + Vec3::from_array(impulse) / record.desc.body.mass)
                            .to_array();
                }
                BodyCommand::Teleport { transform, .. } => {
                    record.desc.body.transform = transform;
                    record.desc.collider.transform = transform;
                }
                BodyCommand::SetBodyType { body_type, .. } => {
                    record.desc.body.body_type = body_type;
                }
            }
            record.active = true;
        }
        Ok(())
    }

    fn step(&mut self, dt: Real) -> Result<(), PhysicsBackendError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(PhysicsBackendError::InvalidStepSeconds { value: dt });
        }
        for (_, record) in self.bodies.iter_mut() {
            record.active =
                integrate_body_sync_state(&mut record.desc.body, record.accumulated_force, dt);
            record.accumulated_force = Vec3::ZERO;
            record.desc.collider.transform = record.desc.body.transform;
        }
        self.refresh_events();
        Ok(())
    }

    fn read_active_states(&mut self, out: &mut Vec<(BodyHandle, PhysicsBodySyncState)>) {
        for (handle, record) in self.bodies.iter_mut() {
            if record.active {
                out.push((handle, record.desc.body.clone()));
                record.active = false;
            }
        }
    }

    fn ray_cast(
        &self,
        query: &PhysicsRayCastQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsRayCastHit>,
    ) {
        let direction = Vec3::from_array(query.direction).normalize_or_zero();
        if !direction.is_finite()
            || direction.length_squared() <= Real::EPSILON
            || !query.max_distance.is_finite()
            || query.max_distance <= 0.0
        {
            return;
        }
        let filtered_query = PhysicsRayCastQuery {
            filter: filter.clone(),
            ..query.clone()
        };
        let sync = self.world_sync(query.world);
        out.extend(sync.colliders.iter().filter_map(|collider| {
            super::query_contact::collider_matches_query(&filtered_query, collider)
                .then(|| {
                    ray_cast_collider(
                        Vec3::from_array(query.origin),
                        direction,
                        query.max_distance,
                        collider,
                    )
                })
                .flatten()
        }));
        out.sort_by(|left, right| left.distance.total_cmp(&right.distance));
    }

    fn shape_cast(
        &self,
        query: &PhysicsShapeCastQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsShapeCastHit>,
    ) {
        let overlaps = shape_overlap_query(
            &self.world_sync(query.world),
            &PhysicsShapeOverlapQuery {
                world: query.world,
                shape: query.shape.clone(),
                transform: query.origin_transform,
                filter: filter.clone(),
            },
        );
        if let Some(hit) = overlaps.into_iter().next() {
            out.push(PhysicsShapeCastHit {
                entity: hit.entity,
                distance: 0.0,
                position: query.origin_transform.translation.to_array(),
                normal: [0.0; 3],
            });
        }
    }

    fn shape_overlap(
        &self,
        query: &PhysicsShapeOverlapQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsShapeOverlapHit>,
    ) {
        out.extend(shape_overlap_query(
            &self.world_sync(query.world),
            &PhysicsShapeOverlapQuery {
                filter: filter.clone(),
                ..query.clone()
            },
        ));
    }

    fn drain_events(&mut self, out: &mut PhysicsEventBuffer) {
        out.contacts.append(&mut self.events.contacts);
        out.triggers.append(&mut self.events.triggers);
    }
}
