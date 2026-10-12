use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderPhaseSortDecisionField {
    PhaseOrder,
    CameraOrder,
    Queue,
    Domain,
    TieBreakerKey,
    EntityTieBreaker,
}
