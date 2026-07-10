use std::collections::{BTreeMap, HashMap};

use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsColliderSyncState, PhysicsSceneStepResult, PhysicsSettings,
        PhysicsWorldStepPlan, PhysicsWorldSyncState,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::Real;

use crate::backend::{
    BodyCommand, BodyDesc, BodyHandle, JoltPhysicsBackend, PhysicsBackend, PhysicsBackendError,
    PhysicsBackendObjectKind, ShapeHandle,
};

use super::change_detection::{collider_requires_recreation, detect_body_change};
use super::command_buffer::{apply_commands_to_sync, PhysicsBodyCommand};
use super::poison_recovery::recover_lock;
use super::world_sync::sanitize_world_sync_state;
use super::DefaultPhysicsManager;

#[derive(Debug)]
pub(super) struct JoltManagedWorld {
    backend: JoltPhysicsBackend,
    entities: HashMap<EntityId, JoltEntity>,
}

#[derive(Clone, Debug)]
struct JoltEntity {
    shape: ShapeHandle,
    body: BodyHandle,
    last_body: PhysicsBodySyncState,
    last_collider: PhysicsColliderSyncState,
    material: PhysicsMaterialMetadata,
}

impl JoltManagedWorld {
    fn new(settings: PhysicsSettings) -> Result<Self, PhysicsBackendError> {
        Ok(Self {
            backend: JoltPhysicsBackend::new(settings)?,
            entities: HashMap::new(),
        })
    }

    fn synchronize(
        &mut self,
        mut sync: PhysicsWorldSyncState,
        default_material: &PhysicsMaterialMetadata,
        step_seconds: Option<Real>,
        queued_commands: &[PhysicsBodyCommand],
    ) -> Result<PhysicsWorldSyncState, PhysicsBackendError> {
        apply_commands_to_sync(&mut sync, queued_commands);
        let colliders = sync
            .colliders
            .iter()
            .cloned()
            .map(|collider| (collider.entity, collider))
            .collect::<HashMap<_, _>>();
        let desired = sync
            .bodies
            .iter()
            .filter_map(|body| {
                let collider = colliders.get(&body.entity)?.clone();
                let material = collider
                    .material_override
                    .clone()
                    .unwrap_or_else(|| default_material.clone());
                Some((body.entity, (body.clone(), collider, material)))
            })
            .collect::<BTreeMap<_, _>>();

        self.remove_stale_entities(&desired)?;
        for (entity, (body, collider, material)) in &desired {
            self.synchronize_entity(sync.world, *entity, body, collider, material)?;
        }
        let backend_commands = self.translate_commands(queued_commands)?;
        if !backend_commands.is_empty() {
            self.backend.apply_commands(&backend_commands)?;
        }
        if let Some(step_seconds) = step_seconds {
            self.backend.step(step_seconds)?;
        }
        self.read_active_states(&mut sync);
        Ok(sync)
    }

    fn remove_stale_entities(
        &mut self,
        desired: &BTreeMap<
            EntityId,
            (
                PhysicsBodySyncState,
                PhysicsColliderSyncState,
                PhysicsMaterialMetadata,
            ),
        >,
    ) -> Result<(), PhysicsBackendError> {
        let mut stale = self
            .entities
            .iter()
            .filter_map(|(entity, record)| {
                let Some((body, collider, material)) = desired.get(entity) else {
                    return Some(*entity);
                };
                (detect_body_change(&record.last_body, body).requires_recreation()
                    || collider_requires_recreation(&record.last_collider, collider)
                    || record.material != *material)
                    .then_some(*entity)
            })
            .collect::<Vec<_>>();
        stale.sort_unstable();
        for entity in stale {
            self.remove_entity(entity)?;
        }
        Ok(())
    }

    fn synchronize_entity(
        &mut self,
        world: WorldHandle,
        entity: EntityId,
        body: &PhysicsBodySyncState,
        collider: &PhysicsColliderSyncState,
        material: &PhysicsMaterialMetadata,
    ) -> Result<(), PhysicsBackendError> {
        if let Some(record) = self.entities.get_mut(&entity) {
            let change = detect_body_change(&record.last_body, body);
            let commands = change.commands(record.body, body);
            if !commands.is_empty() {
                self.backend.apply_commands(&commands)?;
            }
            record.last_body = body.clone();
            record.last_collider = collider.clone();
            record.material = material.clone();
            return Ok(());
        }

        let shape = self.backend.create_shape(&collider.shape, material)?;
        let desc = BodyDesc::from_sync(world, shape, body, collider)?;
        let native_body = match self.backend.create_body(&desc) {
            Ok(native_body) => native_body,
            Err(error) => {
                self.backend.destroy_shape(shape)?;
                return Err(error);
            }
        };
        self.entities.insert(
            entity,
            JoltEntity {
                shape,
                body: native_body,
                last_body: body.clone(),
                last_collider: collider.clone(),
                material: material.clone(),
            },
        );
        Ok(())
    }

    fn remove_entity(&mut self, entity: EntityId) -> Result<(), PhysicsBackendError> {
        let record = self.entities.remove(&entity).ok_or_else(|| {
            PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Body,
                detail: format!("Jolt entity {entity} is not synchronized"),
            }
        })?;
        self.backend.destroy_body(record.body)?;
        self.backend.destroy_shape(record.shape)
    }

    fn read_active_states(&mut self, sync: &mut PhysicsWorldSyncState) {
        let handles = self
            .entities
            .iter()
            .map(|(entity, record)| (record.body, *entity))
            .collect::<HashMap<_, _>>();
        let mut active = Vec::new();
        self.backend.read_active_states(&mut active);
        let active = active
            .into_iter()
            .filter_map(|(handle, body)| Some((*handles.get(&handle)?, body)))
            .collect::<HashMap<_, _>>();
        for body in &mut sync.bodies {
            if let Some(active) = active.get(&body.entity) {
                *body = active.clone();
                if let Some(record) = self.entities.get_mut(&body.entity) {
                    record.last_body = active.clone();
                }
            }
        }
        for collider in &mut sync.colliders {
            if let Some(active) = active.get(&collider.entity) {
                collider.transform = active.transform;
                if let Some(record) = self.entities.get_mut(&collider.entity) {
                    record.last_collider.transform = active.transform;
                }
            }
        }
    }

    fn translate_commands(
        &self,
        commands: &[PhysicsBodyCommand],
    ) -> Result<Vec<BodyCommand>, PhysicsBackendError> {
        commands
            .iter()
            .map(|command| {
                let entity = command.entity();
                let record = self.entities.get(&entity).ok_or_else(|| {
                    PhysicsBackendError::InvalidDescriptor {
                        kind: PhysicsBackendObjectKind::Body,
                        detail: format!(
                            "queued command targets unsynchronized Jolt entity {entity}"
                        ),
                    }
                })?;
                Ok(command.to_backend(record.body))
            })
            .collect()
    }
}

impl DefaultPhysicsManager {
    pub(super) fn sync_jolt_world(&self, sync: PhysicsWorldSyncState, settings: &PhysicsSettings) {
        let world = sync.world;
        match self.synchronize_jolt_world(sync, settings, None, &[]) {
            Ok(sync) => self.store_jolt_sync(sync),
            Err(error) => self.record_jolt_error(world, error),
        }
    }

    pub(super) fn tick_jolt_scene_world(
        &self,
        sync: PhysicsWorldSyncState,
        settings: &PhysicsSettings,
        step_plan: PhysicsWorldStepPlan,
        queued_commands: Vec<PhysicsBodyCommand>,
    ) -> PhysicsSceneStepResult {
        let world = sync.world;
        let step_seconds = (step_plan.steps > 0).then_some(step_plan.step_seconds);
        match self.synchronize_jolt_world(sync, settings, step_seconds, &queued_commands) {
            Ok(sync) => {
                self.store_jolt_sync(sync);
                PhysicsSceneStepResult {
                    step_plan,
                    contacts: Vec::new(),
                    triggers: Vec::new(),
                }
            }
            Err(error) => {
                self.record_jolt_error(world, error);
                PhysicsSceneStepResult {
                    step_plan: PhysicsWorldStepPlan {
                        steps: 0,
                        ..step_plan
                    },
                    contacts: Vec::new(),
                    triggers: Vec::new(),
                }
            }
        }
    }

    pub(super) fn remove_jolt_world(&self, world: WorldHandle) {
        recover_lock(&self.jolt_worlds).remove(&world);
    }

    fn synchronize_jolt_world(
        &self,
        sync: PhysicsWorldSyncState,
        settings: &PhysicsSettings,
        step_seconds: Option<Real>,
        queued_commands: &[PhysicsBodyCommand],
    ) -> Result<PhysicsWorldSyncState, PhysicsBackendError> {
        let sync = sanitize_world_sync_state(sync);
        let world = sync.world;
        let mut worlds = recover_lock(&self.jolt_worlds);
        if !worlds.contains_key(&world) {
            worlds.insert(world, JoltManagedWorld::new(settings.clone())?);
        }
        let Some(runtime) = worlds.get_mut(&world) else {
            return Err(PhysicsBackendError::Initialization {
                backend: "jolt",
                detail: "managed world disappeared during initialization".to_string(),
            });
        };
        runtime.synchronize(sync, &self.default_material, step_seconds, queued_commands)
    }

    fn store_jolt_sync(&self, sync: PhysicsWorldSyncState) {
        recover_lock(&self.contacts).remove(&sync.world);
        recover_lock(&self.trigger_pairs).remove(&sync.world);
        recover_lock(&self.triggers).remove(&sync.world);
        recover_lock(&self.synced_worlds).insert(sync.world, sync);
        *recover_lock(&self.last_backend_error) = None;
    }

    fn record_jolt_error(&self, world: WorldHandle, error: PhysicsBackendError) {
        recover_lock(&self.jolt_worlds).remove(&world);
        recover_lock(&self.synced_worlds).remove(&world);
        recover_lock(&self.contacts).remove(&world);
        recover_lock(&self.trigger_pairs).remove(&world);
        recover_lock(&self.triggers).remove(&world);
        *recover_lock(&self.last_backend_error) = Some(error.to_string());
    }
}
