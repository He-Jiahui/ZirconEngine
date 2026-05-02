use zircon_runtime_interface::ui::event_ui::UiActionDescriptor;

use crate::ui::workbench::snapshot::{ViewContentKind, ViewTabSnapshot};

use super::asset_actions::asset_actions;
use super::common_actions::common_tab_actions;
use super::inspector_actions::inspector_actions;
use super::viewport_actions::viewport_actions;

pub(crate) fn activity_actions_for_tab(tab: &ViewTabSnapshot) -> Vec<UiActionDescriptor> {
    if tab.placeholder {
        return Vec::new();
    }

    let mut actions = common_tab_actions();
    match tab.content_kind {
        ViewContentKind::Inspector => actions.extend(inspector_actions()),
        ViewContentKind::Assets => actions.extend(asset_actions()),
        ViewContentKind::Scene | ViewContentKind::Game => actions.extend(viewport_actions()),
        _ => {}
    }
    actions
}
