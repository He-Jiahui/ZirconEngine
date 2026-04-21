use crate::core::editor_event::MenuAction;
use crate::ui::workbench::event::menu_action_binding;

use super::super::pane_action_model::PaneActionModel;

pub(super) fn open_project_action() -> PaneActionModel {
    PaneActionModel {
        label: "Open Project".to_string(),
        binding: Some(menu_action_binding(&MenuAction::OpenProject)),
        prominent: true,
    }
}

pub(super) fn open_scene_action() -> PaneActionModel {
    PaneActionModel {
        label: "Open Scene".to_string(),
        binding: Some(menu_action_binding(&MenuAction::OpenScene)),
        prominent: true,
    }
}

pub(super) fn create_scene_action() -> PaneActionModel {
    PaneActionModel {
        label: "Create Scene".to_string(),
        binding: Some(menu_action_binding(&MenuAction::CreateScene)),
        prominent: false,
    }
}
