use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalSceneSystem {
    ApplyDeferred,
    HierarchyValidity,
    ActiveHierarchy,
    WorldTransform,
    NodeCache,
    RenderExtractPrepare,
}
