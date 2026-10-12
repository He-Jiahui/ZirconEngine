#[path = "declarations.rs"]
mod declarations;
#[path = "dirty_regions.rs"]
mod dirty_regions;
#[path = "synchronize.rs"]
mod synchronize;

pub(in crate::hybrid_gi) use declarations::{
    HybridGiMeshSdfMaterialFlags, HybridGiMeshSdfObject, HybridGiMeshSdfObjectFlags,
    HybridGiMeshSdfSceneState, HybridGiMeshSdfSyncReport,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
