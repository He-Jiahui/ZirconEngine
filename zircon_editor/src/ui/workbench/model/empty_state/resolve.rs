use crate::snapshot::{EditorChromeSnapshot, ViewContentKind, ViewTabSnapshot};

use super::super::pane_empty_state_model::PaneEmptyStateModel;
use super::console::console_empty_state;
use super::hierarchy::hierarchy_empty_state;
use super::placeholder::placeholder_empty_state;
use super::project::project_or_assets_empty_state;
use super::scene::scene_empty_state;

pub(crate) fn empty_state_for_tab(
    tab: &ViewTabSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Option<PaneEmptyStateModel> {
    match tab.content_kind {
        ViewContentKind::Welcome => None,
        ViewContentKind::Project | ViewContentKind::Assets => project_or_assets_empty_state(chrome),
        ViewContentKind::Hierarchy => hierarchy_empty_state(chrome),
        ViewContentKind::Scene => scene_empty_state(chrome),
        ViewContentKind::Inspector if chrome.inspector.is_none() => Some(PaneEmptyStateModel {
            title: "Nothing selected".to_string(),
            body: "Select an item in Hierarchy or Scene to inspect it.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Console => Some(console_empty_state(chrome)),
        ViewContentKind::Placeholder => Some(placeholder_empty_state()),
        _ => None,
    }
}
