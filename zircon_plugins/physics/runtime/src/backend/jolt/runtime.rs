use std::fmt;

use joltc_sys::{
    JPC_BodyCreationSettings, JPC_BodyID, JPC_BodyInterface_ActivateBody,
    JPC_BodyInterface_AddBody, JPC_BodyInterface_AddForce, JPC_BodyInterface_AddImpulse,
    JPC_BodyInterface_CreateBody, JPC_BodyInterface_DestroyBody,
    JPC_BodyInterface_GetAngularVelocity, JPC_BodyInterface_GetLinearVelocity,
    JPC_BodyInterface_GetPositionAndRotation, JPC_BodyInterface_IsActive,
    JPC_BodyInterface_RemoveBody, JPC_BodyInterface_SetAngularVelocity,
    JPC_BodyInterface_SetLinearVelocity, JPC_BodyInterface_SetMotionType,
    JPC_BodyInterface_SetPositionAndRotation, JPC_Body_GetID, JPC_PhysicsSystem_GetBodyInterface,
    JPC_Shape, JPC_Shape_Release, JPC_ACTIVATION_ACTIVATE, JPC_ACTIVATION_DONT_ACTIVATE,
    JPC_ALLOWED_DOFS_ALL, JPC_PHYSICS_UPDATE_ERROR_NONE,
};
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsQueryFilter,
        PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsSettings, PhysicsShapeCastHit,
        PhysicsShapeCastQuery, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::Real;

use crate::backend::handle_pool::HandlePool;
use crate::backend::validation::{body_desc_is_valid, material_is_valid, shape_is_valid};
use crate::backend::{
    BodyCommand, BodyDesc, BodyHandle, ConstraintDesc, ConstraintHandle, PhysicsBackend,
    PhysicsBackendError, PhysicsBackendObjectKind, PhysicsEventBuffer, ShapeHandle,
};

use super::conversion::{
    create_shape, motion_type, quat, rvec3, vec3, zircon_quat, zircon_translation, zircon_vec3,
};
use super::layers::{OBJECT_LAYER_MOVING, OBJECT_LAYER_NON_MOVING};
use super::native_world::NativeWorld;

const BACKEND_NAME: &str = "jolt";

pub struct JoltPhysicsBackend {
    native: NativeWorld,
    shapes: HandlePool<ShapeRecord, ShapeHandle>,
    bodies: HandlePool<BodyRecord, BodyHandle>,
}

struct ShapeRecord {
    native: *mut JPC_Shape,
    shape: PhysicsColliderShape,
    material: PhysicsMaterialMetadata,
}

#[derive(Clone)]
struct BodyRecord {
    native_id: JPC_BodyID,
    desc: BodyDesc,
}

unsafe impl Send for JoltPhysicsBackend {}

impl JoltPhysicsBackend {
    pub fn new(_settings: PhysicsSettings) -> Result<Self, PhysicsBackendError> {
        Ok(Self {
            native: NativeWorld::new()?,
            shapes: HandlePool::default(),
            bodies: HandlePool::default(),
        })
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
}

impl fmt::Debug for JoltPhysicsBackend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JoltPhysicsBackend")
            .field("native", &self.native)
            .field("shape_count", &self.shapes.iter().count())
            .field("body_count", &self.bodies.iter().count())
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
        let native = unsafe { create_shape(shape)? };
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
        if shape.shape != desc.collider.shape || !body_desc_is_valid(desc) {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Body,
                detail: "body state must be finite and reference the created collider shape"
                    .to_string(),
            });
        }
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
            AllowDynamicOrKinematic: desc.body.body_type != PhysicsBodyType::Static,
            IsSensor: desc.collider.sensor,
            AllowSleeping: desc.body.can_sleep,
            Friction: shape.material.dynamic_friction,
            Restitution: shape.material.restitution,
            LinearDamping: desc.body.linear_damping,
            AngularDamping: desc.body.angular_damping,
            GravityFactor: desc.body.gravity_scale,
            Shape: shape.native,
            ..JPC_BodyCreationSettings::default()
        };
        let body_interface = self.body_interface();
        let native_body = unsafe { JPC_BodyInterface_CreateBody(body_interface, &native_settings) };
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
        match self.bodies.insert(BodyRecord {
            native_id,
            desc: desc.clone(),
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
        Err(PhysicsBackendError::Unsupported {
            backend: BACKEND_NAME,
            operation: "create_constraint",
            detail: "Jolt constraints start in Plugins 03 M4",
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
        Err(Self::invalid_handle(
            PhysicsBackendObjectKind::Constraint,
            constraint.raw(),
        ))
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
        let body_interface = self.body_interface();
        for command in commands {
            let handle = command.body();
            let record = self.bodies.get_mut(handle).ok_or_else(|| {
                Self::invalid_handle(PhysicsBackendObjectKind::Body, handle.raw())
            })?;
            unsafe {
                let activation = if record.desc.body.body_type == PhysicsBodyType::Static {
                    JPC_ACTIVATION_DONT_ACTIVATE
                } else {
                    JPC_ACTIVATION_ACTIVATE
                };
                match *command {
                    BodyCommand::SetLinearVelocity { velocity, .. } => {
                        JPC_BodyInterface_SetLinearVelocity(
                            body_interface,
                            record.native_id,
                            vec3(velocity),
                        );
                        record.desc.body.linear_velocity = velocity;
                    }
                    BodyCommand::SetAngularVelocity { velocity, .. } => {
                        JPC_BodyInterface_SetAngularVelocity(
                            body_interface,
                            record.native_id,
                            vec3(velocity),
                        );
                        record.desc.body.angular_velocity = velocity;
                    }
                    BodyCommand::ApplyForce { force, .. } => {
                        JPC_BodyInterface_AddForce(body_interface, record.native_id, vec3(force));
                    }
                    BodyCommand::ApplyImpulse { impulse, .. } => {
                        JPC_BodyInterface_AddImpulse(
                            body_interface,
                            record.native_id,
                            vec3(impulse),
                        );
                    }
                    BodyCommand::Teleport { transform, .. } => {
                        JPC_BodyInterface_SetPositionAndRotation(
                            body_interface,
                            record.native_id,
                            rvec3(transform.translation),
                            quat(transform.rotation),
                            activation,
                        );
                        record.desc.body.transform = transform;
                        record.desc.collider.transform = transform;
                    }
                    BodyCommand::SetBodyType { body_type, .. } => {
                        JPC_BodyInterface_SetMotionType(
                            body_interface,
                            record.native_id,
                            motion_type(body_type),
                            JPC_ACTIVATION_ACTIVATE,
                        );
                        record.desc.body.body_type = body_type;
                    }
                }
                if record.desc.body.body_type != PhysicsBodyType::Static {
                    JPC_BodyInterface_ActivateBody(body_interface, record.native_id);
                }
            }
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
        Ok(())
    }

    fn read_active_states(&mut self, out: &mut Vec<(BodyHandle, PhysicsBodySyncState)>) {
        let body_interface = self.body_interface();
        for (handle, record) in self.bodies.iter_mut() {
            let active = unsafe { JPC_BodyInterface_IsActive(body_interface, record.native_id) };
            if !active {
                continue;
            }
            let mut position = rvec3(record.desc.body.transform.translation);
            let mut rotation = quat(record.desc.body.transform.rotation);
            unsafe {
                JPC_BodyInterface_GetPositionAndRotation(
                    body_interface,
                    record.native_id,
                    &mut position,
                    &mut rotation,
                );
                record.desc.body.linear_velocity = zircon_vec3(
                    JPC_BodyInterface_GetLinearVelocity(body_interface, record.native_id),
                );
                record.desc.body.angular_velocity = zircon_vec3(
                    JPC_BodyInterface_GetAngularVelocity(body_interface, record.native_id),
                );
            }
            record.desc.body.transform.translation = zircon_translation(position);
            record.desc.body.transform.rotation = zircon_quat(rotation);
            record.desc.collider.transform = record.desc.body.transform;
            out.push((handle, record.desc.body.clone()));
        }
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

    fn drain_events(&mut self, _out: &mut PhysicsEventBuffer) {}
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
        PhysicsColliderShape::Sphere { .. } | PhysicsColliderShape::Capsule { .. } => true,
    }
}
