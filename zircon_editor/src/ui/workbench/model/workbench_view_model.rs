use std::collections::BTreeMap;

use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::document_tab_model::DocumentTabModel;
use super::document_workspace_model::DocumentWorkspaceModel;
use super::drawer_ring_model::DrawerRingModel;
use super::floating_window_model::FloatingWindowModel;
use super::main_host_strip_view_model::MainHostStripViewModel;
use super::menu_bar_model::MenuBarModel;
use super::status_bar_model::StatusBarModel;
use super::tool_window_stack_model::ToolWindowStackModel;

#[derive(Clone, Debug)]
pub struct WorkbenchViewModel {
    pub menu_bar: MenuBarModel,
    pub host_strip: MainHostStripViewModel,
    pub drawer_ring: DrawerRingModel,
    pub tool_windows: BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>,
    pub document_tabs: Vec<DocumentTabModel>,
    pub floating_windows: Vec<FloatingWindowModel>,
    pub document: DocumentWorkspaceModel,
    pub status_bar: StatusBarModel,
}
