use std::collections::BTreeSet;
use std::ops::Deref;
use std::sync::Arc;

use zircon_runtime::scene::{NodeId, WorldInspectionArtifact, WorldInspectionHierarchyRow};

/// Synthetic hierarchy input used by preview and test fixtures.
#[derive(Clone, Debug)]
/// fixture合成层级输入，不携带权威父关系或inspection代次。
pub struct SceneEntry {
    pub id: NodeId,
    pub name: String,
    pub depth: usize,
}

/// Immutable hierarchy rows plus the editor-owned selection overlay for one UI snapshot.
#[derive(Clone, Debug)]
/// runtime层级代次加editor选中覆盖；共享行分配不代表选中状态也相同。
pub struct SceneEntries {
    entries: Arc<[WorldInspectionHierarchyRow]>,
    selected: Arc<BTreeSet<NodeId>>,
    inspection_generation: Option<u64>,
}

impl SceneEntries {
    /// 预览/测试构造路径，inspection代次为空，不能当权威reflow输入。
    pub fn from_entries(
        entries: impl IntoIterator<Item = SceneEntry>,
        selected: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        let entries = entries
            .into_iter()
            .map(|entry| WorldInspectionHierarchyRow {
                entity: entry.id,
                parent: None,
                depth: entry.depth as u32,
                display_name: entry.name,
                kind: "Entity".to_string(),
                subtree_hash: 0,
                active_in_hierarchy: true,
                has_children: false,
            })
            .collect::<Vec<_>>();
        Self {
            entries: entries.into(),
            selected: Arc::new(selected.into_iter().collect()),
            inspection_generation: None,
        }
    }

    pub(crate) fn from_hierarchy_rows_at_generation(
        entries: impl Into<Arc<[WorldInspectionHierarchyRow]>>,
        selected: impl IntoIterator<Item = NodeId>,
        inspection_generation: u64,
    ) -> Self {
        Self {
            entries: entries.into(),
            selected: Arc::new(selected.into_iter().collect()),
            inspection_generation: Some(inspection_generation),
        }
    }

    /// 正常authoring发布共享runtime层级分配并记录artifact代次。
    pub(crate) fn from_artifact(
        artifact: &WorldInspectionArtifact,
        selected: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        Self {
            entries: artifact.hierarchy_rows_arc(),
            selected: Arc::new(selected.into_iter().collect()),
            inspection_generation: Some(artifact.generation()),
        }
    }

    /// 同代次展示过滤仅替换行；调用方须维护原generation与替换行一致。
    pub(crate) fn with_hierarchy_rows(
        &self,
        entries: impl Into<Arc<[WorldInspectionHierarchyRow]>>,
    ) -> Self {
        Self {
            entries: entries.into(),
            selected: self.selected.clone(),
            inspection_generation: self.inspection_generation,
        }
    }

    pub(crate) const fn inspection_generation(&self) -> Option<u64> {
        self.inspection_generation
    }

    pub(crate) fn selected_entities(&self) -> &BTreeSet<NodeId> {
        self.selected.as_ref()
    }

    pub fn is_selected(&self, node_id: NodeId) -> bool {
        self.selected.contains(&node_id)
    }

    /// Clones the runtime-owned immutable hierarchy allocation used by this snapshot.
    pub fn hierarchy_rows_arc(&self) -> Arc<[WorldInspectionHierarchyRow]> {
        self.entries.clone()
    }

    /// Reports whether two snapshots share their generation-derived hierarchy projection.
    pub fn shares_hierarchy_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entries, &other.entries)
    }
}

impl Default for SceneEntries {
    fn default() -> Self {
        Self::from_entries(std::iter::empty(), [])
    }
}

impl From<Vec<SceneEntry>> for SceneEntries {
    fn from(entries: Vec<SceneEntry>) -> Self {
        Self::from_entries(entries, [])
    }
}

impl Deref for SceneEntries {
    type Target = [WorldInspectionHierarchyRow];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
