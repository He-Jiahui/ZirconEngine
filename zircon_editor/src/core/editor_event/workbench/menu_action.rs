use serde::{Deserialize, Serialize};
use zircon_runtime::scene::components::NodeKind;

use crate::core::play::PlayKind;

use super::{ConsoleMessageFilter, ConsoleSourceFilter, ViewDescriptorId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 菜单标识归一化后的宿主操作；执行层再分流到项目、Play、布局、选择、视图等拥有者。
pub enum MenuAction {
    OpenProject,
    OpenScene,
    CreateScene,
    SaveProject,
    SaveAllDocuments,
    CloseProject,
    SaveLayout,
    ResetLayout,
    ClearConsole,
    SetConsoleMessageFilter(ConsoleMessageFilter),
    SetConsoleSourceFilter(ConsoleSourceFilter),
    SelectPlayMode(PlayKind),
    EnterPlayMode,
    KeepPlayChanges,
    ExitPlayMode,
    Undo,
    Redo,
    CreateNode(NodeKind),
    DeleteSelected,
    OpenView(ViewDescriptorId),
}
