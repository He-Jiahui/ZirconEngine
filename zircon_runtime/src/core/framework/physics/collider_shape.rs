use serde::{Deserialize, Serialize};

use crate::core::math::Real;
use crate::core::math::Transform;
use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicsColliderShape {
    Box {
        half_extents: [Real; 3],
    },
    Sphere {
        radius: Real,
    },
    Capsule {
        radius: Real,
        half_height: Real,
    },
    Cylinder {
        radius: Real,
        half_height: Real,
    },
    ConvexHull {
        points: Vec<[Real; 3]>,
    },
    TriangleMesh {
        mesh: AssetReference,
    },
    HeightField {
        resolution: [u32; 2],
        heights: AssetReference,
    },
    Compound {
        children: Vec<(Transform, Box<PhysicsColliderShape>)>,
    },
}
