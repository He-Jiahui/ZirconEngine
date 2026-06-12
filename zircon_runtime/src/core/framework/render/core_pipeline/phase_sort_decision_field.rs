use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderPhaseSortDecisionField {
    PhaseOrder,
    RenderQueue,
    MaterialQueue,
    OrderInLayer,
    UiZIndex,
    OrderedDepthKey,
    EntityTieBreakerKey,
    EntityTieBreaker,
}
