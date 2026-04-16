use serde::{Deserialize, Serialize};

use crate::layout::WorkbenchLayout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::workbench::project) struct LayoutPresetAssetDocument {
    pub(in crate::workbench::project) format_version: u32,
    pub(in crate::workbench::project) preset_name: String,
    pub(in crate::workbench::project) workbench: WorkbenchLayout,
}
