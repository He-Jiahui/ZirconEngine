use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::asset::AssetCreationMenuGeneration;
use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::document_tab_model::DocumentTabModel;
use super::document_workspace_model::DocumentWorkspaceModel;
use super::drawer_ring_model::DrawerRingModel;
use super::floating_window_model::FloatingWindowModel;
use super::main_host_strip_view_model::MainHostStripViewModel;
use super::status_bar_model::StatusBarModel;
use super::tool_window_stack_model::ToolWindowStackModel;
use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry, EditorKeymap, MenuBarModel};
use crate::core::editor_message::PlayStateKind;

/// Run controls use the same controller-derived snapshot and predicates as menus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunControlsModel {
    pub state: PlayStateKind,
    pub enter_enabled: bool,
    pub stop_enabled: bool,
}

impl Default for RunControlsModel {
    fn default() -> Self {
        Self {
            state: PlayStateKind::Edit,
            enter_enabled: false,
            stop_enabled: false,
        }
    }
}

impl RunControlsModel {
    pub fn from_commands(registry: &EditorCommandRegistry, context: &CommandEvalCtx) -> Self {
        let enabled = |id: &str| {
            registry.command(id).is_some_and(|command| command.is_enabled(context))
        };
        Self {
            state: context.play_state(),
            enter_enabled: enabled("runtime.play_mode.enter"),
            stop_enabled: enabled("runtime.play_mode.exit"),
        }
    }
}

#[derive(Clone, Debug)]
/// 同一轮chrome、命令评估及注册环境的呈现投影；宿主布局和反射共用，实际操作仍经命令边界。
pub struct WorkbenchViewModel {
    pub run_controls: RunControlsModel,
    pub asset_creation_menu: Arc<AssetCreationMenuGeneration>,
    pub keymap: EditorKeymap,
    pub menu_bar: MenuBarModel,
    pub host_strip: MainHostStripViewModel,
    pub drawer_ring: DrawerRingModel,
    pub tool_windows: BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>,
    pub document_tabs: Vec<DocumentTabModel>,
    pub floating_windows: Vec<FloatingWindowModel>,
    pub document: DocumentWorkspaceModel,
    pub status_bar: StatusBarModel,
}
