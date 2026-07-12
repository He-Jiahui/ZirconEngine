use zircon_runtime::core::framework::physics::{PhysicsJointSyncState, PhysicsJointType};
use zircon_runtime::core::framework::scene::physics::PhysicsJointDrive;
use zircon_runtime::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointSpring {
    pub stiffness: Real,
    pub damping: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisConstraint {
    pub limit: Option<[Real; 2]>,
    pub drive: Option<PhysicsJointDrive>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JointParams {
    Fixed,
    Distance {
        min: Real,
        max: Real,
        spring: Option<JointSpring>,
    },
    Hinge {
        axis: [Real; 3],
        limit: Option<[Real; 2]>,
        motor: Option<PhysicsJointDrive>,
    },
    Slider {
        axis: [Real; 3],
        limit: Option<[Real; 2]>,
        motor: Option<PhysicsJointDrive>,
    },
    ConeTwist {
        axis: [Real; 3],
        swing_limit: [Real; 2],
        twist_limit: Real,
        motor: Option<PhysicsJointDrive>,
    },
    Generic6Dof {
        axis: [Real; 3],
        linear: [AxisConstraint; 3],
        angular: [AxisConstraint; 3],
    },
}

impl JointParams {
    pub fn from_joint_sync(joint: &PhysicsJointSyncState) -> Self {
        let constraint = &joint.constraint;
        match joint.kind {
            PhysicsJointType::Fixed => Self::Fixed,
            PhysicsJointType::Distance => {
                let [min, max] = joint
                    .limits
                    .or(constraint.linear_limits[0])
                    .unwrap_or([0.0, 0.0]);
                Self::Distance {
                    min,
                    max,
                    spring: drive_is_active(constraint.linear_drives[0]).then_some(JointSpring {
                        stiffness: constraint.linear_drives[0].stiffness,
                        damping: constraint.linear_drives[0].damping,
                    }),
                }
            }
            PhysicsJointType::Hinge => Self::Hinge {
                axis: joint.axis,
                limit: joint.limits.or(constraint.angular_limits[0]),
                motor: active_drive(constraint.angular_drives[0]),
            },
            PhysicsJointType::Slider => Self::Slider {
                axis: joint.axis,
                limit: joint.limits.or(constraint.linear_limits[0]),
                motor: active_drive(constraint.linear_drives[0]),
            },
            PhysicsJointType::ConeTwist => {
                let swing_x = symmetric_extent(constraint.angular_limits[0]);
                let swing_y = symmetric_extent(constraint.angular_limits[1]);
                let twist = symmetric_extent(joint.limits.or(constraint.angular_limits[2]));
                Self::ConeTwist {
                    axis: joint.axis,
                    swing_limit: [swing_x, swing_y],
                    twist_limit: twist,
                    motor: active_drive(constraint.angular_drives[2]),
                }
            }
            PhysicsJointType::Generic6Dof => Self::Generic6Dof {
                axis: joint.axis,
                linear: std::array::from_fn(|axis| AxisConstraint {
                    limit: constraint.linear_limits[axis],
                    drive: active_drive(constraint.linear_drives[axis]),
                }),
                angular: std::array::from_fn(|axis| AxisConstraint {
                    limit: constraint.angular_limits[axis],
                    drive: active_drive(constraint.angular_drives[axis]),
                }),
            },
        }
    }

    pub fn axis(&self) -> [Real; 3] {
        match self {
            Self::Fixed | Self::Distance { .. } => [0.0, 1.0, 0.0],
            Self::Hinge { axis, .. }
            | Self::Slider { axis, .. }
            | Self::ConeTwist { axis, .. }
            | Self::Generic6Dof { axis, .. } => *axis,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Fixed => true,
            Self::Distance { min, max, spring } => {
                min.is_finite()
                    && max.is_finite()
                    && *min >= 0.0
                    && min <= max
                    && spring.is_none_or(|spring| {
                        spring.stiffness.is_finite()
                            && spring.stiffness >= 0.0
                            && spring.damping.is_finite()
                            && spring.damping >= 0.0
                    })
            }
            Self::Hinge { axis, limit, motor } | Self::Slider { axis, limit, motor } => {
                axis_is_valid(*axis) && limit_is_valid(*limit) && drive_is_valid(*motor)
            }
            Self::ConeTwist {
                axis,
                swing_limit,
                twist_limit,
                motor,
            } => {
                axis_is_valid(*axis)
                    && swing_limit
                        .iter()
                        .all(|value| value.is_finite() && *value >= 0.0)
                    && twist_limit.is_finite()
                    && *twist_limit >= 0.0
                    && drive_is_valid(*motor)
            }
            Self::Generic6Dof {
                axis,
                linear,
                angular,
            } => {
                axis_is_valid(*axis)
                    && linear
                        .iter()
                        .chain(angular)
                        .all(|axis| limit_is_valid(axis.limit) && drive_is_valid(axis.drive))
            }
        }
    }
}

fn axis_is_valid(axis: [Real; 3]) -> bool {
    axis.iter().all(|value| value.is_finite())
        && axis.iter().map(|value| value * value).sum::<Real>() > Real::EPSILON
}

fn limit_is_valid(limit: Option<[Real; 2]>) -> bool {
    limit.is_none_or(|[min, max]| min.is_finite() && max.is_finite() && min <= max)
}

fn drive_is_valid(drive: Option<PhysicsJointDrive>) -> bool {
    drive.is_none_or(|drive| {
        [
            drive.target_position,
            drive.target_velocity,
            drive.stiffness,
            drive.damping,
            drive.max_force,
        ]
        .into_iter()
        .all(Real::is_finite)
            && drive.stiffness >= 0.0
            && drive.damping >= 0.0
            && drive.max_force >= 0.0
    })
}

fn active_drive(drive: PhysicsJointDrive) -> Option<PhysicsJointDrive> {
    drive_is_active(drive).then_some(drive)
}

fn drive_is_active(drive: PhysicsJointDrive) -> bool {
    drive != PhysicsJointDrive::default()
}

fn symmetric_extent(limit: Option<[Real; 2]>) -> Real {
    limit
        .map(|[min, max]| min.abs().max(max.abs()))
        .unwrap_or(0.0)
}
