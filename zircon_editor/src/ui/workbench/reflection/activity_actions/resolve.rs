use zircon_runtime_interface::ui::event_ui::UiActionDescriptor;

use crate::ui::workbench::snapshot::{ViewContentKind, ViewTabSnapshot};

use super::asset_actions::{asset_actions, ASSET_ACTION_COUNT};
use super::common_actions::{common_tab_actions, COMMON_TAB_ACTION_COUNT};
use super::inspector_actions::{inspector_actions, INSPECTOR_ACTION_COUNT};
use super::viewport_actions::{viewport_actions, VIEWPORT_ACTION_COUNT};

pub(crate) fn activity_actions_for_tab(tab: &ViewTabSnapshot) -> Vec<UiActionDescriptor> {
    if tab.placeholder {
        return Vec::new();
    }

    let specialized_action_count = match tab.content_kind {
        ViewContentKind::Inspector => INSPECTOR_ACTION_COUNT,
        ViewContentKind::Assets => ASSET_ACTION_COUNT,
        ViewContentKind::Scene | ViewContentKind::Game => VIEWPORT_ACTION_COUNT,
        _ => 0,
    };
    let action_capacity = COMMON_TAB_ACTION_COUNT.saturating_add(specialized_action_count);
    let mut actions = Vec::with_capacity(action_capacity);
    actions.extend(common_tab_actions());
    match tab.content_kind {
        ViewContentKind::Inspector => actions.extend(inspector_actions()),
        ViewContentKind::Assets => actions.extend(asset_actions()),
        ViewContentKind::Scene | ViewContentKind::Game => actions.extend(viewport_actions()),
        _ => {}
    }
    actions
}

#[cfg(test)]
#[path = "tests/resolve_optimization_tests.rs"]
mod optimization_tests;
