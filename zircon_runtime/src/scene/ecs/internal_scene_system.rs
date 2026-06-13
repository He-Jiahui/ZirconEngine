use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalSceneSystem {
    ApplyDeferred,
    UpdateEvents,
    HierarchyValidity,
    ActiveHierarchy,
    WorldTransform,
    NodeCache,
    RenderExtractPrepare,
}
