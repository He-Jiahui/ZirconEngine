use serde::{Deserialize, Serialize};

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum PhysicsMassProperties {
    Explicit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inertia_tensor: Option<[[Real; 3]; 3]>,
    },
    AutoFromShape {
        density: Real,
    },
}

impl Default for PhysicsMassProperties {
    fn default() -> Self {
        Self::Explicit {
            inertia_tensor: None,
        }
    }
}

impl PhysicsMassProperties {
    pub fn is_valid(self) -> bool {
        match self {
            Self::AutoFromShape { density } => density.is_finite() && density > 0.0,
            Self::Explicit {
                inertia_tensor: None,
            } => true,
            Self::Explicit {
                inertia_tensor: Some(tensor),
            } => inertia_tensor_is_positive_definite(tensor),
        }
    }
}

fn inertia_tensor_is_positive_definite(tensor: [[Real; 3]; 3]) -> bool {
    const SYMMETRY_EPSILON: Real = 1.0e-5;
    if tensor.iter().flatten().any(|value| !value.is_finite()) {
        return false;
    }
    if (tensor[0][1] - tensor[1][0]).abs() > SYMMETRY_EPSILON
        || (tensor[0][2] - tensor[2][0]).abs() > SYMMETRY_EPSILON
        || (tensor[1][2] - tensor[2][1]).abs() > SYMMETRY_EPSILON
    {
        return false;
    }
    let leading_minor_2 = tensor[0][0] * tensor[1][1] - tensor[0][1] * tensor[1][0];
    let determinant = tensor[0][0] * (tensor[1][1] * tensor[2][2] - tensor[1][2] * tensor[2][1])
        - tensor[0][1] * (tensor[1][0] * tensor[2][2] - tensor[1][2] * tensor[2][0])
        + tensor[0][2] * (tensor[1][0] * tensor[2][1] - tensor[1][1] * tensor[2][0]);
    tensor[0][0] > 0.0 && leading_minor_2 > 0.0 && determinant > 0.0
}
