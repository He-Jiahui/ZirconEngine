use serde::{Deserialize, Serialize};
use zircon_runtime::scene::components::NodeKind;

use super::ViewDescriptorId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuAction {
    OpenProject,
    OpenScene,
    CreateScene,
    SaveProject,
    SaveLayout,
    ResetLayout,
    Undo,
    Redo,
    CreateNode(NodeKind),
    DeleteSelected,
    OpenView(ViewDescriptorId),
}
