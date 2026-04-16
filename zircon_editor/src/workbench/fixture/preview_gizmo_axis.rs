use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PreviewGizmoAxis {
    X,
    Y,
    Z,
}
