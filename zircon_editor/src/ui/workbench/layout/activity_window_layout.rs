use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::{PaneConstraintOverride, ShellRegionId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::{
    ActivityDrawerLayout, ActivityDrawerSlot, ActivityWindowHostMode, ActivityWindowId,
    DocumentNode,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityWindowLayout {
    pub window_id: ActivityWindowId,
    pub descriptor_id: ViewDescriptorId,
    pub host_mode: ActivityWindowHostMode,
    pub activity_drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    pub content_workspace: DocumentNode,
    #[serde(default)]
    pub region_overrides: BTreeMap<ShellRegionId, PaneConstraintOverride>,
    #[serde(default)]
    pub view_overrides: BTreeMap<ViewInstanceId, PaneConstraintOverride>,
}
