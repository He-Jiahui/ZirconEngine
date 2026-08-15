mod declarations;
mod dirty_regions;
mod synchronize;

pub(in crate::hybrid_gi) use declarations::{
    HybridGiMeshSdfMaterialFlags, HybridGiMeshSdfObject, HybridGiMeshSdfObjectFlags,
    HybridGiMeshSdfSceneState, HybridGiMeshSdfSyncReport,
};

#[cfg(test)]
mod tests;
