use std::f32::consts::PI;

use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::framework::scene::physics::PhysicsMassProperties;
use zircon_runtime::core::math::Real;

use super::{PhysicsBackendError, PhysicsBackendObjectKind};

const INERTIA_RATIO_EPSILON: Real = 1.0e-4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResolvedBodyMass {
    pub mass: Real,
    pub density: Real,
    pub inertia_multiplier: Real,
}

pub(crate) fn resolve_body_mass(
    shape: &PhysicsColliderShape,
    authored_mass: Real,
    properties: PhysicsMassProperties,
) -> Result<ResolvedBodyMass, PhysicsBackendError> {
    if !properties.is_valid() {
        return Err(invalid_mass("mass properties must be finite and positive"));
    }
    let volume = shape_volume(shape)
        .filter(|volume| volume.is_finite() && *volume > 0.0)
        .ok_or_else(|| PhysicsBackendError::Unsupported {
            backend: "physics",
            operation: "resolve_mass_properties",
            detail: "mass resolution requires a supported collider with finite positive volume",
        })?;
    let (mass, density, inertia_tensor) = match properties {
        PhysicsMassProperties::AutoFromShape { density } => (volume * density, density, None),
        PhysicsMassProperties::Explicit { inertia_tensor } => {
            if !authored_mass.is_finite() || authored_mass <= 0.0 {
                return Err(invalid_mass("explicit mass must be finite and positive"));
            }
            (authored_mass, authored_mass / volume, inertia_tensor)
        }
    };
    let inertia_multiplier = match inertia_tensor {
        None => 1.0,
        Some(requested) => explicit_inertia_multiplier(shape, mass, requested)?,
    };
    Ok(ResolvedBodyMass {
        mass,
        density,
        inertia_multiplier,
    })
}

fn shape_volume(shape: &PhysicsColliderShape) -> Option<Real> {
    match shape {
        PhysicsColliderShape::Box { half_extents } => {
            Some(8.0 * half_extents[0] * half_extents[1] * half_extents[2])
        }
        PhysicsColliderShape::Sphere { radius } => Some((4.0 / 3.0) * PI * radius.powi(3)),
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => Some(2.0 * PI * radius.powi(2) * half_height + (4.0 / 3.0) * PI * radius.powi(3)),
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => Some(2.0 * PI * radius.powi(2) * half_height),
        PhysicsColliderShape::Compound { children } => children
            .iter()
            .try_fold(0.0, |sum, (_, child)| Some(sum + shape_volume(child)?)),
        PhysicsColliderShape::ConvexHull { .. }
        | PhysicsColliderShape::TriangleMesh { .. }
        | PhysicsColliderShape::HeightField { .. } => None,
    }
}

fn explicit_inertia_multiplier(
    shape: &PhysicsColliderShape,
    mass: Real,
    requested: [[Real; 3]; 3],
) -> Result<Real, PhysicsBackendError> {
    let base =
        analytic_inertia_diagonal(shape, mass).ok_or_else(|| PhysicsBackendError::Unsupported {
            backend: "physics",
            operation: "explicit_inertia_tensor",
            detail: "explicit inertia tensors currently require a primitive collider",
        })?;
    if requested[0][1].abs() > INERTIA_RATIO_EPSILON
        || requested[0][2].abs() > INERTIA_RATIO_EPSILON
        || requested[1][0].abs() > INERTIA_RATIO_EPSILON
        || requested[1][2].abs() > INERTIA_RATIO_EPSILON
        || requested[2][0].abs() > INERTIA_RATIO_EPSILON
        || requested[2][1].abs() > INERTIA_RATIO_EPSILON
    {
        return Err(PhysicsBackendError::Unsupported {
            backend: "jolt",
            operation: "explicit_inertia_tensor",
            detail: "the current JoltC binding exposes a scalar inertia multiplier, not rotated principal axes",
        });
    }
    let ratios = [
        requested[0][0] / base[0],
        requested[1][1] / base[1],
        requested[2][2] / base[2],
    ];
    if ratios
        .iter()
        .any(|ratio| !ratio.is_finite() || *ratio <= 0.0)
        || (ratios[0] - ratios[1]).abs() > INERTIA_RATIO_EPSILON
        || (ratios[0] - ratios[2]).abs() > INERTIA_RATIO_EPSILON
    {
        return Err(PhysicsBackendError::Unsupported {
            backend: "jolt",
            operation: "explicit_inertia_tensor",
            detail: "the requested inertia tensor is not a uniform scale of the collider inertia",
        });
    }
    Ok(ratios[0])
}

fn analytic_inertia_diagonal(shape: &PhysicsColliderShape, mass: Real) -> Option<[Real; 3]> {
    match shape {
        PhysicsColliderShape::Box { half_extents } => {
            let size = half_extents.map(|extent| extent * 2.0);
            Some([
                mass * (size[1].powi(2) + size[2].powi(2)) / 12.0,
                mass * (size[0].powi(2) + size[2].powi(2)) / 12.0,
                mass * (size[0].powi(2) + size[1].powi(2)) / 12.0,
            ])
        }
        PhysicsColliderShape::Sphere { radius } => Some([0.4 * mass * radius.powi(2); 3]),
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => {
            let height = half_height * 2.0;
            Some([
                mass * (3.0 * radius.powi(2) + height.powi(2)) / 12.0,
                0.5 * mass * radius.powi(2),
                mass * (3.0 * radius.powi(2) + height.powi(2)) / 12.0,
            ])
        }
        PhysicsColliderShape::Capsule { .. }
        | PhysicsColliderShape::ConvexHull { .. }
        | PhysicsColliderShape::TriangleMesh { .. }
        | PhysicsColliderShape::HeightField { .. }
        | PhysicsColliderShape::Compound { .. } => None,
    }
}

fn invalid_mass(detail: &str) -> PhysicsBackendError {
    PhysicsBackendError::InvalidDescriptor {
        kind: PhysicsBackendObjectKind::Body,
        detail: detail.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_box() -> PhysicsColliderShape {
        PhysicsColliderShape::Box {
            half_extents: [0.5; 3],
        }
    }

    #[test]
    fn auto_mass_uses_shape_volume_and_density() {
        let resolved = resolve_body_mass(
            &unit_box(),
            99.0,
            PhysicsMassProperties::AutoFromShape { density: 2.5 },
        )
        .expect("unit box supports automatic mass resolution");

        assert_eq!(resolved.mass, 2.5);
        assert_eq!(resolved.density, 2.5);
        assert_eq!(resolved.inertia_multiplier, 1.0);
    }

    #[test]
    fn explicit_uniform_inertia_scale_maps_to_jolt_multiplier() {
        let resolved = resolve_body_mass(
            &unit_box(),
            2.0,
            PhysicsMassProperties::Explicit {
                inertia_tensor: Some([
                    [2.0 / 3.0, 0.0, 0.0],
                    [0.0, 2.0 / 3.0, 0.0],
                    [0.0, 0.0, 2.0 / 3.0],
                ]),
            },
        )
        .expect("uniform primitive inertia scale is supported");

        assert!((resolved.inertia_multiplier - 2.0).abs() <= INERTIA_RATIO_EPSILON);
    }

    #[test]
    fn zero_volume_shape_rejects_mass_resolution() {
        let error = resolve_body_mass(
            &PhysicsColliderShape::Box {
                half_extents: [0.0, 0.5, 0.5],
            },
            1.0,
            PhysicsMassProperties::AutoFromShape { density: 1.0 },
        )
        .expect_err("zero-volume colliders cannot define mass");

        assert!(matches!(
            error,
            PhysicsBackendError::Unsupported {
                operation: "resolve_mass_properties",
                ..
            }
        ));
    }
}
