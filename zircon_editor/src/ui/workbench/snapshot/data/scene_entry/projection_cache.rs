use std::collections::BTreeSet;

use zircon_runtime::scene::{NodeId, WorldInspectionArtifact};

use super::SceneEntries;

/// Snapshot projection owner retained by `EditorState`.
/// Its output is a direct view of the runtime-owned immutable hierarchy allocation.
#[derive(Debug, Default)]
pub(crate) struct SceneEntryProjectionCache;

impl SceneEntryProjectionCache {
    /// 以当前artifact复用层级分配并叠加本轮selection；此owner不缓存过期world行。
    pub(crate) fn project(
        &self,
        artifact: &WorldInspectionArtifact,
        selected: &BTreeSet<NodeId>,
    ) -> SceneEntries {
        SceneEntries::from_artifact(artifact, selected.iter().copied())
    }
}
