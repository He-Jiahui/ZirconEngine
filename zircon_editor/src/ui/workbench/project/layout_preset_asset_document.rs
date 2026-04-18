use serde::{Deserialize, Serialize};

use crate::layout::WorkbenchLayout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::ui::workbench::project) struct LayoutPresetAssetDocument {
    pub(in crate::ui::workbench::project) format_version: u32,
    pub(in crate::ui::workbench::project) preset_name: String,
    pub(in crate::ui::workbench::project) workbench: WorkbenchLayout,
}
