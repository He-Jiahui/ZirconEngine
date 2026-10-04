use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewHost;

use super::{SplitAxis, SplitPlacement, WorkspaceTarget};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 拖放目标声明；解析不修改布局，宿主须验证后转为布局命令。
pub enum DropTarget {
    Host(ViewHost),
    Split {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
    },
    NewFloatingWindow,
}
