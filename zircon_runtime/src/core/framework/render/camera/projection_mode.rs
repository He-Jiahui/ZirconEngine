use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

impl Default for ProjectionMode {
    fn default() -> Self {
        Self::Perspective
    }
}
