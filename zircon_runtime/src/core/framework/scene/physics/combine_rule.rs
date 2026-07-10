use serde::{Deserialize, Serialize};

/// Authored policy for combining two colliding materials' scalar properties.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsCombineRule {
    #[default]
    Average,
    Minimum,
    Maximum,
    Multiply,
}
