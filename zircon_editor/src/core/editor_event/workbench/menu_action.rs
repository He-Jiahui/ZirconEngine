use serde::{Deserialize, Serialize};
use zircon_runtime::scene::components::NodeKind;

use crate::core::play::PlayKind;

use super::{ConsoleMessageFilter, ViewDescriptorId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuAction {
    OpenProject,
    OpenScene,
    CreateScene,
    SaveProject,
    CloseProject,
    SaveLayout,
    ResetLayout,
    ClearConsole,
    SetConsoleMessageFilter(ConsoleMessageFilter),
    SelectPlayMode(PlayKind),
    EnterPlayMode,
    ExitPlayMode,
    Undo,
    Redo,
    CreateNode(NodeKind),
    DeleteSelected,
    OpenView(ViewDescriptorId),
}
