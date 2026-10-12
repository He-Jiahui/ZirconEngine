use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct InspectorPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub info: SharedString,
    pub inspector_name: SharedString,
    pub inspector_parent: SharedString,
    pub inspector_x: SharedString,
    pub inspector_y: SharedString,
    pub inspector_z: SharedString,
    pub delete_enabled: bool,
}
