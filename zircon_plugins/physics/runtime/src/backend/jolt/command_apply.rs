use joltc_sys::{
    JPC_BodyInterface, JPC_BodyInterface_ActivateBody, JPC_BodyInterface_AddForce,
    JPC_BodyInterface_AddImpulse, JPC_BodyInterface_SetAngularVelocity,
    JPC_BodyInterface_SetLinearVelocity, JPC_BodyInterface_SetMotionQuality,
    JPC_BodyInterface_SetMotionType, JPC_BodyInterface_SetObjectLayer,
    JPC_BodyInterface_SetPositionAndRotation, JPC_Body_SetAllowSleeping, JPC_ACTIVATION_ACTIVATE,
    JPC_ACTIVATION_DONT_ACTIVATE, JPC_MOTION_QUALITY_DISCRETE, JPC_MOTION_QUALITY_LINEAR_CAST,
};
use zircon_runtime::core::framework::{physics::PhysicsBodyType, scene::physics::PhysicsCcdMode};

use crate::backend::BodyCommand;

use super::conversion::{motion_type, quat, rvec3, vec3};
use super::layers::{OBJECT_LAYER_MOVING, OBJECT_LAYER_NON_MOVING};
use super::runtime::BodyRecord;

pub(super) unsafe fn apply_body_command(
    body_interface: *mut JPC_BodyInterface,
    record: &mut BodyRecord,
    command: BodyCommand,
) {
    let activation = if record.desc.body.body_type == PhysicsBodyType::Static {
        JPC_ACTIVATION_DONT_ACTIVATE
    } else {
        JPC_ACTIVATION_ACTIVATE
    };
    match command {
        BodyCommand::SetLinearVelocity { velocity, .. } => {
            JPC_BodyInterface_SetLinearVelocity(body_interface, record.native_id, vec3(velocity));
            record.desc.body.linear_velocity = velocity;
        }
        BodyCommand::SetAngularVelocity { velocity, .. } => {
            JPC_BodyInterface_SetAngularVelocity(body_interface, record.native_id, vec3(velocity));
            record.desc.body.angular_velocity = velocity;
        }
        BodyCommand::ApplyForce { force, .. } => {
            JPC_BodyInterface_AddForce(body_interface, record.native_id, vec3(force));
        }
        BodyCommand::ApplyImpulse { impulse, .. } => {
            JPC_BodyInterface_AddImpulse(body_interface, record.native_id, vec3(impulse));
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
            JPC_BodyInterface_SetObjectLayer(
                body_interface,
                record.native_id,
                if body_type == PhysicsBodyType::Static {
                    OBJECT_LAYER_NON_MOVING
                } else {
                    OBJECT_LAYER_MOVING
                },
            );
            JPC_BodyInterface_SetMotionType(
                body_interface,
                record.native_id,
                motion_type(body_type),
                JPC_ACTIVATION_ACTIVATE,
            );
            record.desc.body.body_type = body_type;
        }
        BodyCommand::SetCcdMode { mode, .. } => {
            JPC_BodyInterface_SetMotionQuality(
                body_interface,
                record.native_id,
                match mode {
                    PhysicsCcdMode::Disabled => JPC_MOTION_QUALITY_DISCRETE,
                    PhysicsCcdMode::LinearCast => JPC_MOTION_QUALITY_LINEAR_CAST,
                },
            );
            record.desc.body.ccd_mode = mode;
        }
        BodyCommand::SetSleepPolicy { policy, .. } => {
            JPC_Body_SetAllowSleeping(record.native, policy.allows_sleep());
            record.desc.body.sleep_policy = policy;
        }
    }
    if record.desc.body.body_type != PhysicsBodyType::Static {
        JPC_BodyInterface_ActivateBody(body_interface, record.native_id);
    }
}

pub(super) unsafe fn apply_projected_body_state(
    body_interface: *mut JPC_BodyInterface,
    record: &mut BodyRecord,
) {
    let activation = if record.desc.body.body_type == PhysicsBodyType::Static {
        JPC_ACTIVATION_DONT_ACTIVATE
    } else {
        JPC_ACTIVATION_ACTIVATE
    };
    JPC_BodyInterface_SetPositionAndRotation(
        body_interface,
        record.native_id,
        rvec3(record.desc.body.transform.translation),
        quat(record.desc.body.transform.rotation),
        activation,
    );
    JPC_BodyInterface_SetLinearVelocity(
        body_interface,
        record.native_id,
        vec3(record.desc.body.linear_velocity),
    );
    JPC_BodyInterface_SetAngularVelocity(
        body_interface,
        record.native_id,
        vec3(record.desc.body.angular_velocity),
    );
}
