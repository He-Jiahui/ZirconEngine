use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::scene::viewport::SceneViewportWorkspaceSessionSnapshot;
use crate::ui::workbench::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 项目辅助UI恢复材料；格式解码不证明实例身份唯一，恢复端还须核验注册与布局约束。
pub struct ProjectEditorWorkspace {
    pub workbench: WorkbenchLayout,
    pub open_view_instances: Vec<ViewInstance>,
    pub focused_view: Option<ViewInstanceId>,
    pub active_drawers: Vec<ActivityDrawerSlot>,
    #[serde(default)]
    pub scene_viewport_sessions: BTreeMap<ViewInstanceId, SceneViewportWorkspaceSessionSnapshot>,
}
