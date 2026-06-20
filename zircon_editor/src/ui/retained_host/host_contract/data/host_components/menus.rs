use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;
use super::common::{FrameRect, HostChromeControlFrameData, HostChromeTabData, TabData};

#[derive(Clone, Default)]
pub(crate) struct HostMenuChromeItemData {
    pub label: SharedString,
    pub shortcut: SharedString,
    pub action_id: SharedString,
    pub enabled: bool,
    pub children: ModelRc<HostMenuChromeItemData>,
}

#[derive(Clone, Default)]
pub(crate) struct HostMenuChromeMenuData {
    pub label: SharedString,
    pub popup_width_px: f32,
    pub popup_height_px: f32,
    pub popup_nodes: ModelRc<TemplatePaneNodeData>,
    pub items: ModelRc<HostMenuChromeItemData>,
}

#[derive(Clone, Default)]
pub(crate) struct HostMenuChromeData {
    pub outer_margin_px: f32,
    pub top_bar_height_px: f32,
    pub template_nodes: ModelRc<TemplatePaneNodeData>,
    pub menu_frames: ModelRc<HostChromeControlFrameData>,
    pub save_project_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub delete_enabled: bool,
    pub preset_names: ModelRc<SharedString>,
    pub active_preset_name: SharedString,
    pub resolved_preset_name: SharedString,
    pub menus: ModelRc<HostMenuChromeMenuData>,
}

#[derive(Clone, Default)]
pub(crate) struct HostPageChromeData {
    pub top_bar_height_px: f32,
    pub host_bar_height_px: f32,
    pub template_nodes: ModelRc<TemplatePaneNodeData>,
    pub tab_row_frame: FrameRect,
    pub project_path_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub project_path: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct HostStatusBarData {
    pub status_bar_frame: FrameRect,
    pub template_nodes: ModelRc<TemplatePaneNodeData>,
    pub status_primary: SharedString,
    pub status_secondary: SharedString,
    pub viewport_label: SharedString,
}
