use std::collections::{HashMap, HashSet};
use std::fmt;

use joltc_sys::{
    JPC_Body, JPC_BodyCreationSettings, JPC_BodyID, JPC_BodyInterface_AddBody,
    JPC_BodyInterface_CreateBody, JPC_BodyInterface_DestroyBody,
    JPC_BodyInterface_GetAngularVelocity, JPC_BodyInterface_GetLinearVelocity,
    JPC_BodyInterface_GetPositionAndRotation, JPC_BodyInterface_IsActive,
    JPC_BodyInterface_RemoveBody, JPC_Body_GetID, JPC_PhysicsSystem_GetBodyInterface, JPC_Shape,
    JPC_Shape_Release, JPC_ACTIVATION_ACTIVATE, JPC_ACTIVATION_DONT_ACTIVATE, JPC_ALLOWED_DOFS_ALL,
    JPC_MOTION_QUALITY_DISCRETE, JPC_MOTION_QUALITY_LINEAR_CAST, JPC_PHYSICS_UPDATE_ERROR_NONE,
};
#[cfg(test)]
use joltc_sys::{JPC_BodyInterface_GetMotionQuality, JPC_Body_GetAllowSleeping};
#[cfg(test)]
use zircon_runtime::core::framework::scene::physics::PhysicsSleepPolicy;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsMeshAsset,
        PhysicsQueryFilter, PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsSettings,
        PhysicsShapeCastHit, PhysicsShapeCastQuery, PhysicsShapeOverlapHit,
        PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
    },
    scene::physics::{PhysicsCcdMode, PhysicsMaterialMetadata},
};
use zircon_runtime::core::math::Real;
use zircon_runtime::core::resource::AssetReference;

use crate::backend::builtin::{
    compute_contact_events, compute_trigger_events, PhysicsTriggerPairMap,
};
use crate::backend::handle_pool::HandlePool;
use crate::backend::validation::{body_desc_is_valid, material_is_valid, shape_is_valid};
use crate::backend::{
    resolve_body_mass, BodyCommand, BodyDesc, BodyHandle, ConstraintDesc, ConstraintHandle,
    PhysicsBackend, PhysicsBackendError, PhysicsBackendObjectKind, PhysicsEventBuffer, ShapeHandle,
};

use super::command_apply::{apply_body_command, apply_projected_body_state};
use super::conversion::{
    create_shape, motion_type, quat, rvec3, vec3, zircon_quat, zircon_translation, zircon_vec3,
};
use super::layers::{OBJECT_LAYER_MOVING, OBJECT_LAYER_NON_MOVING};
use super::mesh_shape::validate_mesh_asset;
use super::native_world::NativeWorld;

const BACKEND_NAME: &str = "jolt";

pub struct JoltPhysicsBackend {
    native: NativeWorld,
    settings: PhysicsSettings,
    shapes: HandlePool<ShapeRecord, ShapeHandle>,
    bodies: HandlePool<BodyRecord, BodyHandle>,
    constraints: HandlePool<ConstraintDesc, ConstraintHandle>,
    mesh_assets: HashMap<AssetReference, PhysicsMeshAsset>,
    trigger_pairs:
        HashMap<zircon_runtime::core::framework::scene::WorldHandle, PhysicsTriggerPairMap>,
    events: PhysicsEventBuffer,
}

struct ShapeRecord {
    native: *mut JPC_Shape,
    shape: PhysicsColliderShape,
    material: PhysicsMaterialMetadata,
}

#[derive(Clone)]
pub(super) struct BodyRecord {
    pub(super) native: *mut JPC_Body,
    pub(super) native_id: JPC_BodyID,
    pub(super) desc: BodyDesc,
}

unsafe impl Send for JoltPhysicsBackend {}

impl JoltPhysicsBackend {
    pub fn new(settings: PhysicsSettings) -> Result<Self, PhysicsBackendError> {
        Ok(Self {
            native: NativeWorld::new()?,
            settings,
            shapes: HandlePool::default(),
            bodies: HandlePool::default(),
            constraints: HandlePool::default(),
            mesh_assets: HashMap::new(),
            trigger_pairs: HashMap::new(),
            events: PhysicsEventBuffer::default(),
        })
    }

    pub fn register_mesh_asset(
        &mut self,
        reference: AssetReference,
        asset: PhysicsMeshAsset,
    ) -> Result<(), PhysicsBackendError> {
        validate_mesh_asset(&asset).map_err(|detail| PhysicsBackendError::InvalidDescriptor {
            kind: PhysicsBackendObjectKind::Shape,
            detail,
        })?;
        self.mesh_assets.insert(reference, asset);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn debug_body_runtime_policy(
        &self,
        body: BodyHandle,
    ) -> Option<(PhysicsBodyType, PhysicsCcdMode, PhysicsSleepPolicy)> {
        let record = self.bodies.get(body)?;
        let quality =
            unsafe { JPC_BodyInterface_GetMotionQuality(self.body_interface(), record.native_id) };
        let ccd_mode = if quality == JPC_MOTION_QUALITY_LINEAR_CAST {
            PhysicsCcdMode::LinearCast
        } else {
            PhysicsCcdMode::Disabled
        };
        let sleep_policy = if unsafe { JPC_Body_GetAllowSleeping(record.native) } {
            PhysicsSleepPolicy::Allow
        } else {
            PhysicsSleepPolicy::Never
        };
        Some((record.desc.body.body_type, ccd_mode, sleep_policy))
    }

    fn body_interface(&self) -> *mut joltc_sys::JPC_BodyInterface {
        unsafe { JPC_PhysicsSystem_GetBodyInterface(self.native.physics_system) }
    }

    fn invalid_handle(kind: PhysicsBackendObjectKind, raw: u64) -> PhysicsBackendError {
        PhysicsBackendError::InvalidHandle { kind, raw }
    }

    fn destroy_native_body(&self, native_id: JPC_BodyID) {
        unsafe {
            let body_interface = self.body_interface();
            JPC_BodyInterface_RemoveBody(body_interface, native_id);
            JPC_BodyInterface_DestroyBody(body_interface, native_id);
        }
    }

    fn world_sync(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
    ) -> PhysicsWorldSyncState {
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

    fn project_constraints(&mut self, step_seconds: Real) {
        if self.constraints.iter().next().is_none() {
            return;
        }
        let body_interface = self.body_interface();
        let mut touched = HashSet::new();
        for (_, record) in self.bodies.iter_mut() {
            unsafe { read_native_body_state(body_interface, record) };
        }
        let constraints = self
            .constraints
            .iter()
            .map(|(_, constraint)| constraint.clone())
            .collect::<Vec<_>>();
        for constraint in constraints {
            if let Some(body_b) = constraint.body_b {
                let body_b_handle = body_b;
                let Some((body_a, body_b)) =
                    self.bodies.get_pair_mut(constraint.body_a, body_b_handle)
                else {
                    continue;
                };
                crate::constraint::project_constraint(
                    &constraint,
                    crate::constraint::ProjectedBodies {
                        body_a: &mut body_a.desc.body,
                        body_b: Some(&mut body_b.desc.body),
                    },
                    step_seconds,
                );
                body_a.desc.collider.transform = body_a.desc.body.transform;
                body_b.desc.collider.transform = body_b.desc.body.transform;
                touched.insert(constraint.body_a);
                touched.insert(body_b_handle);
            } else if let Some(body_a) = self.bodies.get_mut(constraint.body_a) {
                crate::constraint::project_constraint(
                    &constraint,
                    crate::constraint::ProjectedBodies {
                        body_a: &mut body_a.desc.body,
                        body_b: None,
                    },
                    step_seconds,
                );
                body_a.desc.collider.transform = body_a.desc.body.transform;
                touched.insert(constraint.body_a);
            }
        }
        for handle in touched {
            if let Some(record) = self.bodies.get_mut(handle) {
                unsafe { apply_projected_body_state(body_interface, record) };
            }
        }
    }
}

impl fmt::Debug for JoltPhysicsBackend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JoltPhysicsBackend")
            .field("native", &self.native)
            .field("shape_count", &self.shapes.iter().count())
            .field("body_count", &self.bodies.iter().count())
            .field("mesh_asset_count", &self.mesh_assets.len())
            .finish()
    }
}

impl PhysicsBackend for JoltPhysicsBackend {
    fn name(&self) -> &'static str {
        BACKEND_NAME
    }

    fn create_shape(
        &mut self,
        shape: &PhysicsColliderShape,
        material: &PhysicsMaterialMetadata,
    ) -> Result<ShapeHandle, PhysicsBackendError> {
        if !shape_is_valid(shape)
            || !jolt_shape_dimensions_are_supported(shape)
            || !material_is_valid(material)
            || material.static_friction < 0.0
            || material.dynamic_friction < 0.0
            || !(0.0..=1.0).contains(&material.restitution)
        {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Shape,
                detail: "shape dimensions and material values must be finite and valid".to_string(),
            });
        }
        let native = unsafe { create_shape(shape, &self.mesh_assets, None)? };
        match self.shapes.insert(ShapeRecord {
            native,
            shape: shape.clone(),
            material: material.clone(),
        }) {
            Some(handle) => Ok(handle),
            None => {
                unsafe { JPC_Shape_Release(native) };
                Err(PhysicsBackendError::CapacityExhausted {
                    kind: PhysicsBackendObjectKind::Shape,
                })
            }
        }
    }

    fn create_body(&mut self, desc: &BodyDesc) -> Result<BodyHandle, PhysicsBackendError> {
        let shape = self.shapes.get(desc.shape).ok_or_else(|| {
            Self::invalid_handle(PhysicsBackendObjectKind::Shape, desc.shape.raw())
        })?;
        if desc.body.body_type != PhysicsBodyType::Static
            && shape_requires_static_body(&shape.shape)
        {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Body,
                detail: "triangle-mesh and height-field colliders require a static body"
                    .to_string(),
            });
        }
        if shape.shape != desc.collider.shape || !body_desc_is_valid(desc) {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Body,
                detail: "body state must be finite and reference the created collider shape"
                    .to_string(),
            });
        }
        let resolved_mass = if desc.body.body_type == PhysicsBodyType::Static {
            None
        } else {
            Some(resolve_body_mass(
                &shape.shape,
                desc.body.mass,
                desc.body.mass_properties,
            )?)
        };
        let native_body_shape = match resolved_mass {
            Some(resolved) => unsafe {
                create_shape(&shape.shape, &self.mesh_assets, Some(resolved.density))?
            },
            None => shape.native,
        };
        let native_settings = JPC_BodyCreationSettings {
            Position: rvec3(desc.body.transform.translation),
            Rotation: quat(desc.body.transform.rotation),
            LinearVelocity: vec3(desc.body.linear_velocity),
            AngularVelocity: vec3(desc.body.angular_velocity),
            UserData: desc.body.entity,
            ObjectLayer: if desc.body.body_type == PhysicsBodyType::Static {
                OBJECT_LAYER_NON_MOVING
            } else {
                OBJECT_LAYER_MOVING
            },
            MotionType: motion_type(desc.body.body_type),
            AllowedDOFs: JPC_ALLOWED_DOFS_ALL,
            AllowDynamicOrKinematic: !shape_requires_static_body(&shape.shape),
            IsSensor: desc.collider.sensor,
            MotionQuality: match desc.body.ccd_mode {
                PhysicsCcdMode::Disabled => JPC_MOTION_QUALITY_DISCRETE,
                PhysicsCcdMode::LinearCast => JPC_MOTION_QUALITY_LINEAR_CAST,
            },
            AllowSleeping: desc.body.sleep_policy.allows_sleep(),
            Friction: shape.material.dynamic_friction,
            Restitution: shape.material.restitution,
            LinearDamping: desc.body.linear_damping,
            AngularDamping: desc.body.angular_damping,
            GravityFactor: desc.body.gravity_scale,
            InertiaMultiplier: resolved_mass
                .map(|resolved| resolved.inertia_multiplier)
                .unwrap_or(1.0),
            Shape: native_body_shape,
            ..JPC_BodyCreationSettings::default()
        };
        let body_interface = self.body_interface();
        let native_body = unsafe { JPC_BodyInterface_CreateBody(body_interface, &native_settings) };
        if native_body_shape != shape.native {
            unsafe { JPC_Shape_Release(native_body_shape) };
        }
        if native_body.is_null() {
            return Err(PhysicsBackendError::Initialization {
                backend: BACKEND_NAME,
                detail: "JoltC returned null while creating a body".to_string(),
            });
        }
        let native_id = unsafe { JPC_Body_GetID(native_body) };
        let activation = if desc.body.body_type == PhysicsBodyType::Static {
            JPC_ACTIVATION_DONT_ACTIVATE
        } else {
            JPC_ACTIVATION_ACTIVATE
        };
        unsafe { JPC_BodyInterface_AddBody(body_interface, native_id, activation) };
        let mut stored_desc = desc.clone();
        if let Some(resolved) = resolved_mass {
            stored_desc.body.mass = resolved.mass;
        }
        match self.bodies.insert(BodyRecord {
            native: native_body,
            native_id,
            desc: stored_desc,
        }) {
            Some(handle) => Ok(handle),
            None => {
                self.destroy_native_body(native_id);
                Err(PhysicsBackendError::CapacityExhausted {
                    kind: PhysicsBackendObjectKind::Body,
                })
            }
        }
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
        if desc.body_b == Some(desc.body_a) {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Constraint,
                detail: "constraint cannot connect a body to itself".to_string(),
            });
        }
        if !desc.params.is_valid() {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Constraint,
                detail: "constraint parameters must be finite and ordered".to_string(),
            });
        }
        self.constraints
            .insert(desc.clone())
            .ok_or(PhysicsBackendError::CapacityExhausted {
                kind: PhysicsBackendObjectKind::Constraint,
            })
    }

    fn destroy_shape(&mut self, shape: ShapeHandle) -> Result<(), PhysicsBackendError> {
        if self.bodies.iter().any(|(_, body)| body.desc.shape == shape) {
            return Err(PhysicsBackendError::ObjectInUse {
                kind: PhysicsBackendObjectKind::Shape,
                raw: shape.raw(),
            });
        }
        let record = self
            .shapes
            .remove(shape)
            .ok_or_else(|| Self::invalid_handle(PhysicsBackendObjectKind::Shape, shape.raw()))?;
        unsafe { JPC_Shape_Release(record.native) };
        Ok(())
    }

    fn destroy_body(&mut self, body: BodyHandle) -> Result<(), PhysicsBackendError> {
        if self
            .constraints
            .iter()
            .any(|(_, constraint)| constraint.handles().any(|candidate| candidate == body))
        {
            return Err(PhysicsBackendError::ObjectInUse {
                kind: PhysicsBackendObjectKind::Body,
                raw: body.raw(),
            });
        }
        let record = self
            .bodies
            .remove(body)
            .ok_or_else(|| Self::invalid_handle(PhysicsBackendObjectKind::Body, body.raw()))?;
        self.destroy_native_body(record.native_id);
        Ok(())
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
            let Some(record) = self.bodies.get(body) else {
                return Err(Self::invalid_handle(
                    PhysicsBackendObjectKind::Body,
                    body.raw(),
                ));
            };
            if let BodyCommand::SetBodyType { body_type, .. } = *command {
                if body_type != PhysicsBodyType::Static
                    && shape_requires_static_body(&record.desc.collider.shape)
                {
                    return Err(PhysicsBackendError::InvalidDescriptor {
                        kind: PhysicsBackendObjectKind::Body,
                        detail: "triangle-mesh and height-field colliders require a static body"
                            .to_string(),
                    });
                }
            }
        }
        let body_interface = self.body_interface();
        for command in commands {
            let handle = command.body();
            let record = self.bodies.get_mut(handle).ok_or_else(|| {
                Self::invalid_handle(PhysicsBackendObjectKind::Body, handle.raw())
            })?;
            unsafe { apply_body_command(body_interface, record, *command) };
        }
        Ok(())
    }

    fn step(&mut self, dt: Real) -> Result<(), PhysicsBackendError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(PhysicsBackendError::InvalidStepSeconds { value: dt });
        }
        let result = unsafe { self.native.update(dt) };
        if result != JPC_PHYSICS_UPDATE_ERROR_NONE {
            return Err(PhysicsBackendError::StepFailed {
                backend: BACKEND_NAME,
                code: result,
            });
        }
        self.project_constraints(dt);
        Ok(())
    }

    fn read_active_states(&mut self, out: &mut Vec<(BodyHandle, PhysicsBodySyncState)>) {
        let body_interface = self.body_interface();
        for (handle, record) in self.bodies.iter_mut() {
            let active = unsafe { JPC_BodyInterface_IsActive(body_interface, record.native_id) };
            if !active {
                continue;
            }
            unsafe { read_native_body_state(body_interface, record) };
            out.push((handle, record.desc.body.clone()));
        }
        self.refresh_events();
    }

    fn ray_cast(
        &self,
        _query: &PhysicsRayCastQuery,
        _filter: &PhysicsQueryFilter,
        _out: &mut Vec<PhysicsRayCastHit>,
    ) {
    }

    fn shape_cast(
        &self,
        _query: &PhysicsShapeCastQuery,
        _filter: &PhysicsQueryFilter,
        _out: &mut Vec<PhysicsShapeCastHit>,
    ) {
    }

    fn shape_overlap(
        &self,
        _query: &PhysicsShapeOverlapQuery,
        _filter: &PhysicsQueryFilter,
        _out: &mut Vec<PhysicsShapeOverlapHit>,
    ) {
    }

    fn drain_events(&mut self, out: &mut PhysicsEventBuffer) {
        out.contacts.append(&mut self.events.contacts);
        out.triggers.append(&mut self.events.triggers);
    }
}

unsafe fn read_native_body_state(
    body_interface: *mut joltc_sys::JPC_BodyInterface,
    record: &mut BodyRecord,
) {
    let mut position = rvec3(record.desc.body.transform.translation);
    let mut rotation = quat(record.desc.body.transform.rotation);
    unsafe {
        JPC_BodyInterface_GetPositionAndRotation(
            body_interface,
            record.native_id,
            &mut position,
            &mut rotation,
        );
        record.desc.body.linear_velocity = zircon_vec3(JPC_BodyInterface_GetLinearVelocity(
            body_interface,
            record.native_id,
        ));
        record.desc.body.angular_velocity = zircon_vec3(JPC_BodyInterface_GetAngularVelocity(
            body_interface,
            record.native_id,
        ));
    }
    record.desc.body.transform.translation = zircon_translation(position);
    record.desc.body.transform.rotation = zircon_quat(rotation);
    record.desc.collider.transform = record.desc.body.transform;
}

impl Drop for JoltPhysicsBackend {
    fn drop(&mut self) {
        let body_ids = self
            .bodies
            .iter()
            .map(|(_, record)| record.native_id)
            .collect::<Vec<_>>();
        for body_id in body_ids {
            self.destroy_native_body(body_id);
        }
        for (_, shape) in self.shapes.iter() {
            unsafe { JPC_Shape_Release(shape.native) };
        }
    }
}

fn jolt_shape_dimensions_are_supported(shape: &PhysicsColliderShape) -> bool {
    match shape {
        PhysicsColliderShape::Box { half_extents } => {
            half_extents.iter().all(|extent| *extent > 0.0)
        }
        PhysicsColliderShape::Sphere { .. }
        | PhysicsColliderShape::Capsule { .. }
        | PhysicsColliderShape::Cylinder { .. }
        | PhysicsColliderShape::ConvexHull { .. }
        | PhysicsColliderShape::Compound { .. } => true,
        PhysicsColliderShape::TriangleMesh { .. } | PhysicsColliderShape::HeightField { .. } => {
            true
        }
    }
}

fn shape_requires_static_body(shape: &PhysicsColliderShape) -> bool {
    match shape {
        PhysicsColliderShape::TriangleMesh { .. } | PhysicsColliderShape::HeightField { .. } => {
            true
        }
        PhysicsColliderShape::Compound { children } => children
            .iter()
            .any(|(_, child)| shape_requires_static_body(child)),
        PhysicsColliderShape::Box { .. }
        | PhysicsColliderShape::Sphere { .. }
        | PhysicsColliderShape::Capsule { .. }
        | PhysicsColliderShape::Cylinder { .. }
        | PhysicsColliderShape::ConvexHull { .. } => false,
    }
}
