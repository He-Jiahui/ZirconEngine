use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct SceneNodeData {
    pub id: SharedString,
    pub name: SharedString,
    pub depth: i32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct HierarchyPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub hierarchy_nodes: ModelRc<SceneNodeData>,
}
