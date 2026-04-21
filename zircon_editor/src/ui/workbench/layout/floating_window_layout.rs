use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::view::ViewInstanceId;

use super::{DocumentNode, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatingWindowLayout {
    pub window_id: MainPageId,
    pub title: String,
    pub workspace: DocumentNode,
    pub focused_view: Option<ViewInstanceId>,
    #[serde(default)]
    pub frame: ShellFrame,
}
