use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 默认抽屉归属和交互意图；布局构造器解释这些声明，运行时命令仍需许可校验。
pub struct JetBrainsShellPreset {
    pub drawers: Vec<JetBrainsDrawerPreset>,
    pub tab_behavior: JetBrainsTabBehavior,
    pub floating_window_behavior: JetBrainsFloatingWindowBehavior,
}

impl JetBrainsShellPreset {
    pub fn new(drawers: impl IntoIterator<Item = JetBrainsDrawerPreset>) -> Self {
        Self {
            drawers: drawers.into_iter().collect(),
            tab_behavior: JetBrainsTabBehavior::default(),
            floating_window_behavior: JetBrainsFloatingWindowBehavior::default(),
        }
    }

    pub fn drawer(&self, slot: ActivityDrawerSlot) -> Option<&JetBrainsDrawerPreset> {
        self.drawers.iter().find(|drawer| drawer.slot == slot)
    }

    pub fn drawer_for_view(&self, view_id: &str) -> Option<&JetBrainsDrawerPreset> {
        self.drawers
            .iter()
            .find(|drawer| drawer.visible_views.iter().any(|view| view == view_id))
    }

    pub fn drawer_slot_for_view(&self, view_id: &str) -> Option<ActivityDrawerSlot> {
        self.drawer_for_view(view_id).map(|drawer| drawer.slot)
    }

    pub fn default_mode_for_slot(&self, slot: ActivityDrawerSlot) -> Option<ActivityDrawerMode> {
        self.drawer(slot).map(|drawer| drawer.default_mode)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 某语义抽屉的初始视图和保存意图；visible_views是描述符清单。
pub struct JetBrainsDrawerPreset {
    pub slot: ActivityDrawerSlot,
    pub label: String,
    pub default_mode: ActivityDrawerMode,
    pub visible_views: Vec<String>,
    pub allows_detach: bool,
    pub allows_attach: bool,
    pub collapse_to_activity_bar: bool,
    pub persist_extent: bool,
    pub persist_selection: bool,
}

impl JetBrainsDrawerPreset {
    pub fn new(
        slot: ActivityDrawerSlot,
        label: impl Into<String>,
        visible_views: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        Self {
            slot,
            label: label.into(),
            default_mode: ActivityDrawerMode::Pinned,
            visible_views: visible_views.into_iter().map(str::to_string).collect(),
            allows_detach: true,
            allows_attach: true,
            collapse_to_activity_bar: true,
            persist_extent: true,
            persist_selection: true,
        }
    }

    pub fn with_default_mode(mut self, mode: ActivityDrawerMode) -> Self {
        self.default_mode = mode;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 标签交互设计要求，实际callback和停靠命令应共同落实。
pub struct JetBrainsTabBehavior {
    pub reorder_tabs: bool,
    pub activate_on_drop: bool,
    pub close_documents_with_middle_click: bool,
    pub keep_tool_tabs_close_guarded: bool,
}

impl Default for JetBrainsTabBehavior {
    fn default() -> Self {
        Self {
            reorder_tabs: true,
            activate_on_drop: true,
            close_documents_with_middle_click: true,
            keep_tool_tabs_close_guarded: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 分离与恢复的设计要求；原生窗口和持久化由宿主生命周期处理。
pub struct JetBrainsFloatingWindowBehavior {
    pub detach_to_native_window: bool,
    pub attach_to_original_drawer: bool,
    pub focus_floating_window_on_detach: bool,
    pub persist_floating_geometry: bool,
    pub restore_hidden_drawers_on_attach: bool,
}

impl Default for JetBrainsFloatingWindowBehavior {
    fn default() -> Self {
        Self {
            detach_to_native_window: true,
            attach_to_original_drawer: true,
            focus_floating_window_on_detach: true,
            persist_floating_geometry: true,
            restore_hidden_drawers_on_attach: true,
        }
    }
}
